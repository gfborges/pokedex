use dialoguer::{theme::ColorfulTheme, Select, Input, MultiSelect};
use std::sync::Arc;

use crate::repositories::pokemon::Repository;

mod create_pokemon;
mod fetch_all_pokemons;
mod fetch_pokemon;
mod delete_pokemon;

pub fn run(repo: Arc<dyn Repository>) {
    let choices = [
        "Fetch all Pokemons",
        "Fetch a Pokemon",
        "Create a Pokemon",
        "Delete a Pokemon",
        "Exit",
    ];
    loop {
        let prompt = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Make your choice")
            .items(&choices)
            .default(0)
            .interact();

        let index = match prompt {
            Ok(i) => i,
            Err(_) => continue
        };

        match index {
            0 => fetch_all_pokemons::run(repo.clone()),
            1 => fetch_pokemon::run(repo.clone()),
            2 => create_pokemon::run(repo.clone()),
            3 => delete_pokemon::run(repo.clone()),
            4 => break,
            _ => continue,
        }
    }
}


pub fn prompt_number() -> Result<u16, ()> {
    match Input::new().with_prompt("Pokemon number").interact_text() {
        Ok(number) => Ok(number),
        _ => Err(()),
    }
}

pub fn prompt_name() -> Result<String, ()> {
    match Input::new().with_prompt("Pokemon name").interact_text() {
        Ok(name) => Ok(name),
        _ => Err(()),
    }
}

pub fn prompt_types() -> Result<Vec<String>, ()> {
    let types = ["Electric", "Fire"];
    match MultiSelect::new()
        .with_prompt("Pokemon types")
        .items(&types)
        .interact()
    {
        Ok(indexes) => Ok(indexes
            .into_iter()
            .map(|index| String::from(types[index]))
            .collect::<Vec<String>>()),
        _ => Err(()),
    }
}
