use std::sync::Arc;

use serde::Serialize;

use crate::repositories::pokemon::Repository;

use crate::domain::fetch_pokemon;

use super::status_code::Status;

#[derive(Serialize)]
pub struct Response {
    number: u16,
    name: String,
    types: Vec<String>,
}

pub fn serve(repo: Arc<dyn Repository>, number: u16) -> rouille::Response {
    let req = fetch_pokemon::Request::new(number);
    match fetch_pokemon::execute(repo, req) {
        Ok(pokemon) => rouille::Response::json(&Response {
            number: pokemon.number,
            name: pokemon.name,
            types: pokemon.types,
        }),
        Err(fetch_pokemon::Error::BadRequest) => rouille::Response::from(Status::BadRequest),
        Err(fetch_pokemon::Error::NotFound) => rouille::Response::from(Status::NotFound),
        Err(fetch_pokemon::Error::Unknown) => rouille::Response::from(Status::InternalServerError),
    }
}
