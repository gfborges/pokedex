mod api;
pub mod domain;
pub mod repositories;

#[macro_use]
extern crate rouille;

fn main() {
    api::serve("127.0.0.1:8000");
    println!("Hello, world!");
}
