use std::sync::Arc;

use repositories::pokemon::InMemoryRepository;

mod api;
pub mod domain;
pub mod repositories;

#[macro_use]
extern crate rouille;
extern crate serde;

fn main() {
    let repo = InMemoryRepository::new();
    api::serve("127.0.0.1:8000", Arc::new(repo));
}
