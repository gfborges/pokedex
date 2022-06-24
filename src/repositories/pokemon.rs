use crate::domain::entities::{Pokemon, PokemonName, PokemonNumber, PokemonTypes};

#[derive(Debug)]
pub enum InsertError {
    Conflict,
    Unknown,
}

#[derive(Debug)]
pub enum FetchAllError {
    Unknown,
}

#[derive(Debug)]
pub enum FetchOneError {
    Unknown,
    NotFound,
}

#[derive(Debug)]
pub enum DeleteError {
    Unknown,
    NotFound,
}

pub trait Repository: Send + Sync {
    fn insert(
        &self,
        number: PokemonNumber,
        name: PokemonName,
        types: PokemonTypes,
    ) -> Result<Pokemon, InsertError>;
    fn fetch_all(&self) -> Result<Vec<Pokemon>, FetchAllError>;
    fn fetch_one(&self, number: PokemonNumber) -> Result<Pokemon, FetchOneError>;
    fn delete(&self, number: PokemonNumber) -> Result<(), DeleteError>;
}
