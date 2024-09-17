use quote::{format_ident, quote};
use regex::Regex;
use std::path::PathBuf;
use std::{fs, path::Path};
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};
use syn::{parse_quote, Fields, ItemEnum, ItemImpl, ItemStruct, Type};

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// New version to use
    #[arg(short, long)]
    migration_version: String,

    /// Path to version to copy
    #[arg(short, long)]
    alpha_path: String,

    /// Path to the new crate
    #[arg(short, long)]
    output_path: String,

    /// The name of the previous migration crate  e.g. `v3`
    #[arg(short, long)]
    prev_version: String,
}

struct ItemVisitor<'ast> {
    structs: Vec<&'ast ItemStruct>,
    enums: Vec<&'ast ItemEnum>,
}

impl<'ast> Visit<'ast> for ItemVisitor<'ast> {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        self.structs.push(node);
        visit::visit_item_struct(self, node);
    }
    fn visit_item_enum(&mut self, i: &'ast ItemEnum) {
        self.enums.push(i);
        visit::visit_item_enum(self, i);
    }
}
struct MigrationReset<'ast> {
    structs: Vec<&'ast ItemStruct>,
    enums: Vec<&'ast ItemEnum>,
}

fn generate_migrate_into(item_struct: &ItemStruct) -> ItemImpl {
    let struct_name = &item_struct.ident;
    let fields = &item_struct.fields;

    let field_assignments = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! { #field_name: self.#field_name.migrate() }
    });

    parse_quote! {
        impl MigrateInto<#struct_name> for prev::#struct_name {
            fn migrate(self) -> #struct_name {
                #struct_name {
                    #(#field_assignments,)*
                }
            }
        }
    }
}

fn generate_migrate_into_enum(item_enum: &ItemEnum) -> ItemImpl {
    let enum_name = &item_enum.ident;

    let variant_matches = item_enum.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        match &variant.fields {
            Fields::Named(named_fields) => {
                let field_names = named_fields.named.iter().map(|f| &f.ident);
                let field_assignments = field_names.clone();
                quote! {
                    prev::#enum_name::#variant_name { #(#field_names),* } =>
                        #enum_name::#variant_name { #(#field_assignments: #field_assignments.migrate()),* }
                }
            },
            Fields::Unnamed(unnamed_fields) => {
                let field_names: Vec<_> = (0..unnamed_fields.unnamed.len())
                    .map(|i| format_ident!("field{}", i))
                    .collect();
                quote! {
                    prev::#enum_name::#variant_name(#(#field_names),*) =>
                        #enum_name::#variant_name(#(#field_names.migrate()),*)
                }
            },
            Fields::Unit => {
                quote! {
                    prev::#enum_name::#variant_name => #enum_name::#variant_name
                }
            },
        }
    });

    parse_quote! {
        impl MigrateInto<#enum_name> for prev::#enum_name {
            fn migrate(self) -> #enum_name {
                match self {
                    #(#variant_matches,)*
                }
            }
        }
    }
}

impl<'ast> VisitMut for MigrationReset<'ast> {
    fn visit_item_impl_mut(&mut self, node: &mut syn::ItemImpl) {
        if let Some(ident) = node.trait_.as_ref().and_then(|(_, trait_path, _)| {
            trait_path
                .segments
                .last()
                .filter(|seg| seg.ident.to_string() == "MigrateInto")
                .and_then(|segment| {
                    if let syn::PathArguments::AngleBracketed(angle_bracketed_generic_arguments) =
                        &segment.arguments
                    {
                        angle_bracketed_generic_arguments
                            .args
                            .first()
                            .and_then(|x| {
                                if let syn::GenericArgument::Type(Type::Path(path)) = x {
                                    path.path.get_ident().map(|i| i.to_string())
                                } else {
                                    None
                                }
                            })
                    } else {
                        None
                    }
                })
        }) {
            println!("Processing type: {}", ident);
            if let Some(struct_) = self.structs.iter().find(|s| s.ident.to_string() == ident) {
                let updated = generate_migrate_into(&struct_);
                *node = updated;
            }
            if let Some(enum_) = self.enums.iter().find(|e| e.ident.to_string() == ident) {
                let updated = generate_migrate_into_enum(&enum_);
                *node = updated;
            }
        }
        visit_mut::visit_item_impl_mut(self, node);
    }
}

fn reset_migration_for_file(file_path: &Path, version: &str) -> Result<(), std::io::Error> {
    let code = fs::read_to_string(&file_path).unwrap();

    let syntax_tree_read = syn::parse_file(&code).unwrap();
    let mut syntax_tree = syn::parse_file(&code).unwrap();
    let mut visitor = ItemVisitor {
        structs: Vec::new(),
        enums: Vec::new(),
    };
    visitor.visit_file(&syntax_tree_read);
    let mut visitor = MigrationReset {
        structs: visitor.structs.clone(),
        enums: visitor.enums.clone(),
    };
    visitor.visit_file_mut(&mut syntax_tree);
    let code = prettyplease::unparse(&syntax_tree);
    // replace the string 'use \w as prev;' with 'user {version} as prev';
    let re = Regex::new(r"use .* as prev;").unwrap();
    let code = re
        .replace(&code, &format!("use {} as prev;", version))
        .to_string();
    fs::write(file_path, code)
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn relative_path(from_path: &Path, to_path: &Path) -> String {
    // Get the common ancestor (root) between the two paths
    let common_root = from_path
        .ancestors()
        .find(|ancestor| to_path.starts_with(ancestor));

    if let Some(common_root) = common_root {
        // The number of directories to go up from the `from_path` to the common root
        let mut upward_steps = PathBuf::new();
        for _ in common_root.components().count()..from_path.components().count() {
            upward_steps.push("..");
        }

        // The path from the common root to the destination (to_path)
        let remaining_to_path = to_path.strip_prefix(common_root).unwrap();

        // Combine the upward steps with the remaining path
        upward_steps
            .join(remaining_to_path)
            .to_string_lossy()
            .to_string()
    } else {
        // In case there is no common root, just return the `to_path` as is
        to_path.to_string_lossy().to_string()
    }
}

/// This tool creates promotes the current "alpha" migration crate to a numbered
/// migration crate and resets the alpha crate via the following steps:
/// - Copies the current alpha into a new folder name given by --output-path
/// - Renames the package.name to the vaue given by --migration-version
/// - Adds the newly created crate as a dependency to the alpha crate
/// - Removes the previous crate dependency from the alpha crate (given by --prev-version)
/// - Resets the migration code of each file the alpha crate using [reset_migration_for_file]
/// - replaces the string "use {version} as prev;" in each file;
fn main() {
    let Args {
        migration_version,
        alpha_path,
        output_path,
        prev_version,
    } = clap::Parser::parse();
    let alpha_path = Path::new(&alpha_path);
    let output_path = Path::new(&output_path);
    // Copy the current alpha into a new folder
    fs::create_dir_all(&output_path).unwrap();
    for entry in fs::read_dir(&alpha_path).unwrap() {
        let entry = entry.unwrap();
        let target_path = output_path.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            fs::create_dir_all(&target_path).unwrap();
            copy_dir_all(entry.path(), target_path).unwrap();
        } else {
            fs::copy(entry.path(), target_path).unwrap();
        }
    }

    // Rename the package.name in Cargo.toml
    let cargo_toml_path = output_path.join("Cargo.toml");
    let mut cargo_toml: toml::Value = fs::read_to_string(&cargo_toml_path)
        .unwrap()
        .parse()
        .unwrap();
    cargo_toml["package"]["name"] = toml::Value::String(migration_version.clone());
    fs::write(&cargo_toml_path, toml::to_string(&cargo_toml).unwrap()).unwrap();

    // Add the newly created crate as a dependency to the alpha crate
    let alpha_cargo_toml_path = alpha_path.join("Cargo.toml");
    let mut alpha_cargo_toml: toml::Value = fs::read_to_string(&alpha_cargo_toml_path)
        .unwrap()
        .parse()
        .unwrap();
    let dependencies = &mut alpha_cargo_toml["dependencies"].as_table_mut().unwrap();
    dependencies.insert(
        migration_version.clone(),
        toml::Value::Table({
            let mut table = toml::Table::new();
            table.insert(
                "path".to_string(),
                toml::Value::String(relative_path(alpha_path, output_path)),
            );
            table
        }),
    );

    // Remove the previous crate dependency from the alpha crate
    if let Some(deps) = alpha_cargo_toml
        .get_mut("dependencies")
        .and_then(|d| d.as_table_mut())
    {
        deps.remove(&prev_version);
    }
    fs::write(
        &alpha_cargo_toml_path,
        toml::to_string(&alpha_cargo_toml).unwrap(),
    )
    .unwrap();

    // Reset the migration code of each file in the alpha crate
    let path = Path::new(&alpha_path).join("src");
    for entry in walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        println!(
            "Resetting migration code for file {}",
            entry.path().display()
        );
        reset_migration_for_file(&entry.path(), &migration_version).unwrap();
    }
}
