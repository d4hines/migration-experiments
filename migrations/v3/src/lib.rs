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
    pub field1: Option<u64>,
    pub field2: bool,
    pub field4: Vec<bool>,
}

pub enum D {
    E1 { field1: f64, field2: bool },
    E2(f64),
}

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
            field1: if self.field1 { Some(99) } else { None },
            field2: self.field2,
            field4: vec![false],
        }
    }
}
