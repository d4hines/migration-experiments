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
        todo!()
    }
}
use v3 as prev;
impl MigrateInto<A> for prev::A {
    fn migrate(self) -> A {
        todo!()
    }
}
impl MigrateInto<B> for prev::B {
    fn migrate(self) -> B {
        todo!()
    }
}

impl MigrateInto<C> for prev::C {
    fn migrate(self) -> C {
        todo!()
    }
}
