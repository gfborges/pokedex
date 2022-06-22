use std::sync::Mutex;

use crate::domain::entities::{Pokemon, PokemonName, PokemonNumber, PokemonTypes};

#[derive(Debug)]
pub enum InsertError {
    Conflict,
    Unknown
}

#[derive(Debug)]
pub enum FetchAllError {
    Unknown
}

pub trait Repository: Send + Sync {
    fn insert(&self, number: PokemonNumber, name: PokemonName, types: PokemonTypes) -> Result<Pokemon, InsertError>;
    fn fetch_all(&self) -> Result<Vec<Pokemon>, FetchAllError>;
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
    fn insert(&self, number: PokemonNumber, name: PokemonName, types: PokemonTypes) -> Result<Pokemon, InsertError> {
        if self.error {
            return Err(InsertError::Unknown);
        }
        let mut pokemons = match self.pokemons.lock() {
            Ok(lock) => lock,
            _ => return Err(InsertError::Unknown), 
        };
        if pokemons.iter().any(|pokemon| pokemon.number == number) {
            return Err(InsertError::Conflict);
        }
        let pokemon = Pokemon::new(number, name, types);
        pokemons.push(pokemon.clone());
        Ok(pokemon)
    }

    fn fetch_all(&self) -> Result<Vec<Pokemon>, FetchAllError> {
        if self.error {
            return Err(FetchAllError::Unknown);
        }

        let mut pokemons = match self.pokemons.lock() {
            Ok(lock) => lock.to_vec(),
            Err(_) => return Err(FetchAllError::Unknown),
        };

        pokemons.sort_by(|a, b| a.number.cmp(&b.number));
        Ok(pokemons)
    }
}
