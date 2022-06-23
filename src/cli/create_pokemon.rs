use std::sync::Arc;

use crate::{repositories::pokemon::Repository};
use crate::domain::create_pokemon;

use super::{prompt_number, prompt_name, prompt_types};


pub fn run(repo: Arc<dyn Repository>) {
    let number = prompt_number();
    let name = prompt_name();
    let types = prompt_types();


    let req = match (number, name, types) {
        (Ok(number), Ok(name), Ok(types)) => create_pokemon::Request {
            number,
            name,
            types,
        },
        _ => {
            println!("An error occurred during the prompt");
            return;
        }
    };

    match create_pokemon::execute(repo, req) {
        Ok(res) => println!(
            "{:?}",
            res
        ),
        Err(create_pokemon::Error::BadRequest) => println!("The request is invalid"),
        Err(create_pokemon::Error::Conflict) => println!("The Pokemon already exists"),
        Err(create_pokemon::Error::Unknown) => println!("An unknown error occurred"),
    };
}