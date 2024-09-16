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
}
