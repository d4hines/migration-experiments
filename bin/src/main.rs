use v2::{A, B, C};
fn main() {
    let state = A {
        field1: B {
            field1: 0.0,
            field2: C {
                field1: true,
                field2: true,
                field3: Some(99.9),
            },
        },
    };
    println!("{:#?}", state);
}
