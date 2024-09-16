//! The alpha crate implements the next version of our types, which
//! will be made availalbe on the next release. All development work
//! happens on this crate. Each release, the dev making the release
//! transfers the changes on alpha to a new version, and resets alpha
//! to the "identity migration".

use migration_utils::MigrateInto;

pub struct A {
    pub field1: B,
    pub field2: f64,
}

pub struct B {
    pub field1: f64,
    pub field2: C,
}

pub struct C {
    pub field1: bool,
    pub field2: bool,
    pub field4: Vec<bool>,
}

// This gets bumped up on every release.
use v2 as prev;

impl MigrateInto<A> for prev::A {
    fn migrate(self) -> A {
        A {
            field1: self.field1.migrate(),
            field2: f64::default(),
        }
    }
}

impl MigrateInto<B> for prev::B {
    fn migrate(self) -> B {
        B {
            field1: f64::default(),
            field2: self.field2.migrate(),
        }
    }
}

impl MigrateInto<C> for prev::C {
    fn migrate(self) -> C {
        C {
            field1: self.field1,
            field2: self.field2,
            field4: vec![false],
        }
    }
}
