use iter::Iter;

#[repr(u8)]
#[derive(Debug, Iter)]
enum Foo {
    A,
    B,
}

#[repr(i32)]
#[derive(Debug, Iter)]
#[allow(unused)]
enum Bar {
    A,
    B,
    C,
    D,
    E,
    F,
    Meow,
}

fn main() {
    for x in Foo::iter() {
        dbg!(x);
    }

    println!();

    for x in Bar::iter() {
        dbg!(x);
    }

    println!("Hello, world!");
}
