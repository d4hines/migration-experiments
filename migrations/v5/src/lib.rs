//! The alpha crate implements the next version of our types, which
//! will be made availalbe on the next release. All development work
//! happens on this crate. Each release, the dev making the release
//! transfers the changes on alpha to a new version, and resets alpha
//! to the "identity migration".
use migration_utils::MigrateInto;
#[derive(Debug)]
pub struct A {
    pub field1: B,
    pub field2: f64,
}
#[derive(Debug)]
pub struct B {
    pub field1: f64,
    pub field2: C,
    pub field3: u64,
}
#[derive(Debug)]
pub struct C {
    pub field1: Option<u64>,
    pub field2: bool,
    pub field4: Vec<bool>,
}
pub enum D {
    E1 { field1: f64, field2: bool },
    E2(f64),
}
impl MigrateInto<D> for prev::D {
    fn migrate(self) -> D {
        match self {
            prev::D::E1 { field1, field2 } => D::E1 {
                field1: field1.migrate(),
                field2: field2.migrate(),
            },
            prev::D::E2(field0) => D::E2(field0.migrate()),
        }
    }
}
use v4 as prev;
impl MigrateInto<A> for prev::A {
    fn migrate(self) -> A {
        todo!()
    }
}
impl MigrateInto<B> for prev::B {
    fn migrate(self) -> B {
        B {
            field1: self.field1.migrate(),
            field2: self.field2.migrate(),
            field3: 99,
        }
    }
}
impl MigrateInto<C> for prev::C {
    fn migrate(self) -> C {
        C {
            field1: self.field1.migrate(),
            field2: self.field2.migrate(),
            field4: self.field4.migrate(),
        }
    }
}
