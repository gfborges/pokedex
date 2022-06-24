use crate::repositories::pokemon::{DeleteError, Repository};
use std::sync::Arc;

use super::entities::PokemonNumber;

#[derive(Debug)]
pub enum Error {
    Unknown,
    BadRequest,
    NotFound,
}

pub struct Request {
    pub number: u16,
}

pub fn execute(repo: Arc<dyn Repository>, req: Request) -> Result<(), Error> {
    match PokemonNumber::try_from(req.number) {
        Ok(number) => match repo.delete(number) {
            Ok(_) => Ok(()),
            Err(DeleteError::NotFound) => Err(Error::NotFound),
            Err(DeleteError::Unknown) => Err(Error::Unknown),
        },
        Err(_) => Err(Error::BadRequest),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::domain::entities::{PokemonName, PokemonTypes};
    use crate::repositories::inmemory_pokemon::InMemoryRepository;

    #[test]
    fn it_should_return_unknown_error_when_unexpected_error_happens() {
        let repo = Arc::new(InMemoryRepository::new().with_error());

        let req = Request { number: 25 };
        let res = execute(repo, req);

        assert!(matches!(res, Err(Error::Unknown)))
    }

    #[test]
    fn it_should_return_bad_request_when_number_is_invalid() {
        let repo = Arc::new(InMemoryRepository::new());

        let req = Request { number: 0 };
        let res = execute(repo, req);

        assert!(matches!(res, Err(Error::BadRequest)));
    }

    #[test]
    fn it_should_return_not_found_when_repo_does_not_find_pokemon() {
        let repo = Arc::new(InMemoryRepository::new());

        let req = Request { number: 1 };
        let res = execute(repo, req);

        assert!(matches!(res, Err(Error::NotFound)));
    }

    #[test]
    fn it_should_return_ok_when_pokemon_is_deleted() {
        let repo = Arc::new(InMemoryRepository::new());
        repo.insert(
            PokemonNumber::vulpix(),
            PokemonName::vulpix(),
            PokemonTypes::vulpix(),
        )
        .expect("error inserting vulpix");
        repo.insert(
            PokemonNumber::pikachu(),
            PokemonName::pikachu(),
            PokemonTypes::pikachu(),
        )
        .expect("error inserting pikachu");

        let req = Request { number: 25 };
        let res = execute(repo.clone(), req).expect("error while deleting pikachu");

        let pokemons = repo.fetch_all().expect("error on fetch all pokemons");

        assert!(matches!(res, ()));
        assert_eq!(pokemons.len(), 1);
        assert_eq!(pokemons[0].number, PokemonNumber::vulpix());
    }
}
