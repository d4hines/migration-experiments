pub trait MigrateInto<T> {
    fn migrate(self) -> T;
}

// Implement self-to-self migration for simple types
macro_rules! impl_migrate_into_self {
    ($($t:ty),+) => {
        $(
            impl MigrateInto<$t> for $t {
                fn migrate(self) -> $t {
                    self
                }
            }
        )+
    };
}

impl_migrate_into_self!(f64, i64, u64, bool, Option<u64>, Vec<bool>);
