fn main() {
    divan::main();
}

#[divan::bench]
fn foo() -> i32 {
    divan::black_box(1) + divan::black_box(2)
}
