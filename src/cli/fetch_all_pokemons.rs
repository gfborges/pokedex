use std::sync::Arc;

use crate::{repositories::pokemon::Repository};
use crate::domain::fetch_all_pokemons;


pub fn run(repo: Arc<dyn Repository>) {
    match fetch_all_pokemons::execute(repo) {
        Ok(res) => res.into_iter().for_each(|p| {
            println!(
                "{:?}",
                p
            );
        }),
        Err(fetch_all_pokemons::Error::Unknown) => println!("An unknown error occurred"),
    };
}