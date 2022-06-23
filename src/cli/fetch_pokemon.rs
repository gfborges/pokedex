use std::sync::Arc;

use crate::domain::fetch_pokemon;
use crate::repositories::pokemon::Repository;

use super::prompt_number;


#[derive(Debug)]
struct Response {
    number: u16,
    name: String,
    types: Vec<String>,
}


pub fn run(repo: Arc<dyn Repository>) {
    let number = match prompt_number() {
        Ok(n) => n,
        Err(_) => {
            println!("An error occurred during prompt!");
            return;
        },
    };

    let req = fetch_pokemon::Request::new(number);
    match fetch_pokemon::execute(repo, req) {
        Ok(res) => println!(
            "{:?}",
            Response {
                number: res.number,
                name: res.name,
                types: res.types,
            }
        ),
        Err(fetch_pokemon::Error::Unknown) => println!("An unknown error uccurred"),
        Err(fetch_pokemon::Error::BadRequest) => println!("Invalid request"),
        Err(fetch_pokemon::Error::NotFound) => println!("Pokemon not found"),
    }
}