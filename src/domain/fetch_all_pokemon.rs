use std::sync::Arc;

use crate::repositories::pokemon::Repository;

#[derive(Debug)]
pub enum Error {
    Unknown,
}

pub struct Response {
    pub number: u16,
    pub name: String,
    pub types: Vec<String>,
}

pub fn execute(repo: Arc<dyn Repository>) -> Result<Vec<Response>, Error> {
    match repo.fetch_all() {
        Ok(pokemons) => Ok(
            pokemons.into_iter().map(|pokemon| Response {
                number: u16::from(pokemon.number),
                name: String::from(pokemon.name),
                types: Vec::<String>::from(pokemon.types)
            }).collect()
        ),
        Err(_) => Err(Error::Unknown)
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        domain::entities::{PokemonName, PokemonNumber, PokemonTypes},
        repositories::pokemon::{InMemoryRepository, Repository},
    };

    use super::*;

    #[test]
    fn it_should_return_an_error_when_an_unexpected_error_happens() {
        let repo = Arc::new(InMemoryRepository::new().with_error());

        let res = execute(repo);

        assert!(matches!(res, Err(Error::Unknown)), "didn't returned error");
    }

    #[test]
    fn it_should_return_all_pokemon_in_repository() {
        let repo = Arc::new(InMemoryRepository::new());
        repo.insert(
            PokemonNumber::pikachu(),
            PokemonName::pikachu(),
            PokemonTypes::pikachu(),
        )
        .expect("error inserting pikachu");
        repo.insert(
            PokemonNumber::vulpix(),
            PokemonName::vulpix(),
            PokemonTypes::vulpix(),
        )
        .expect("error inserting vulpix");

        let res = execute(repo).expect("execute returned an error");

        assert_eq!(res[0].number, 25);
        assert_eq!(res[0].name, "Pikachu".to_owned());
        assert_eq!(res[0].types, vec!["Electric".to_owned()]);

        assert_eq!(res[1].number, 37);
        assert_eq!(res[1].name, "Vulpix".to_owned());
        assert_eq!(res[1].types, vec!["Fire".to_owned()]);
    }
}
