use std::sync::Arc;

use serde::Serialize;

use crate::repositories::pokemon::Repository;

use crate::domain::fetch_all_pokemons;

use super::status_code::Status;

#[derive(Serialize)]
pub struct Response {
    number: u16,
    name: String,
    types: Vec<String>,
}

pub fn serve(repo: Arc<dyn Repository>) -> rouille::Response {
    match fetch_all_pokemons::execute(repo) {
        Ok(pokemons) => rouille::Response::json(&pokemons.into_iter().map(|pokemon| Response {
            number: pokemon.number,
            name: pokemon.name,
            types: pokemon.types,
        }).collect::<Vec<Response>>()),
        Err(fetch_all_pokemons::Error::Unknown) => {
            rouille::Response::from(Status::InternalServerError)
        }
    }
}
