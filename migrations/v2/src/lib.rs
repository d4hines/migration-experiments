use migration_utils::MigrateInto;

#[derive(Debug)]
pub struct A {
    pub field1: B,
}

#[derive(Debug)]
pub struct B {
    pub field1: f64,
    pub field2: C,
}

#[derive(Debug)]
pub struct C {
    pub field1: bool,
    pub field2: bool,
    pub field3: Option<f64>,
}

use v1 as prev;

impl MigrateInto<A> for prev::A {
    fn migrate(self) -> A {
        A {
            field1: self.field1.migrate(),
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
            field1: self.field1.migrate(),
            field2: self.field2.migrate(),
            field3: Option::default(),
        }
    }
}
