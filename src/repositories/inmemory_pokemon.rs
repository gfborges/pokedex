
use crate::domain::entities::PokemonTypes;
use crate::domain::entities::PokemonName;
use crate::domain::entities::PokemonNumber;

use super::pokemon::DeleteError;
use super::pokemon::FetchAllError;
use super::pokemon::FetchOneError;
use super::pokemon::InsertError;
use super::pokemon::Repository;

use crate::domain::entities::Pokemon;

use std::sync::Mutex;

pub struct InMemoryRepository {
    pub(crate) error: bool,
    pub(crate) pokemons: Mutex<Vec<Pokemon>>,
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
    fn insert(
        &self,
        number: PokemonNumber,
        name: PokemonName,
        types: PokemonTypes,
    ) -> Result<Pokemon, InsertError> {
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

    fn fetch_one(&self, number: PokemonNumber) -> Result<Pokemon, FetchOneError> {
        if self.error {
            return Err(FetchOneError::Unknown);
        }

        let pokemons = match self.pokemons.lock() {
            Ok(lock) => lock.to_vec(),
            Err(_) => return Err(FetchOneError::Unknown),
        };

        match pokemons.iter().find(|p| p.number == number) {
            Some(pokemon) => Ok(pokemon.clone()),
            None => Err(FetchOneError::NotFound),
        }
    }

    fn delete(&self, number: PokemonNumber) -> Result<(), DeleteError> {
        if self.error {
            return Err(DeleteError::Unknown);
        }
        let mut pokemons = match self.pokemons.lock() {
            Ok(lock) => lock,
            Err(_) => return Err(DeleteError::Unknown),
        };

        let index = match pokemons.iter().position(|p| p.number == number) {
            Some(index) => index,
            None => return Err(DeleteError::NotFound),
        };
        pokemons.remove(index);
        Ok(())
    }
}
