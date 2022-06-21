use std::sync::Mutex;

use crate::domain::entities::{Pokemon, PokemonName, PokemonNumber, PokemonTypes};

pub enum Insert {
    Ok(PokemonNumber),
    Conflict,
    Error
}
pub trait Repository: Send + Sync {
    fn insert(&self, number: PokemonNumber, name: PokemonName, types: PokemonTypes) -> Insert;
}

pub struct InMemoryRepository {
    error: bool,
    pokemons: Mutex<Vec<Pokemon>>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self {
            pokemons: Mutex::new(vec![]),
            error: false,
        }
    }

    #[cfg(test)]
    pub fn with_error(self) -> Self {
        Self {
            error: true,
            ..self
        }
    }
}

impl Repository for InMemoryRepository {
    fn insert(&self, number: PokemonNumber, name: PokemonName, types: PokemonTypes) -> Insert {
        if self.error {
            return Insert::Error;
        }
        println!("no error");
        let mut pokemons = match self.pokemons.lock() {
            Ok(lock) => lock,
            _ => return Insert::Error, 
        };
        println!("no mutex error");
        if pokemons.iter().any(|pokemon| pokemon.number == number) {
            return Insert::Conflict;
        }
        println!("no conflict ");

        let number_clone = number.clone();
        pokemons.push(Pokemon::new(number, name, types));
        Insert::Ok(number_clone)
    }
}
