use std::sync::Arc;

use crate::repositories::pokemon::{FetchOneError, Repository};

use super::entities::PokemonNumber;

#[derive(Debug)]
pub enum Error {
    Unknown,
    BadRequest,
    NotFound,
}

pub struct Request {
    number: u16,
}

impl Request {
    pub fn new(number: u16) -> Self {
        Self {
            number,
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub number: u16,
    pub name: String,
    pub types: Vec<String>,
}

pub fn execute(repo: Arc<dyn Repository>, req: Request) -> Result<Response, Error> {
    match PokemonNumber::try_from(req.number) {
        Ok(number) => match repo.fetch_one(number) {
            Ok(pokemon) => Ok(Response {
                number: u16::from(pokemon.number),
                name: String::from(pokemon.name),
                types: Vec::<String>::from(pokemon.types),
            }),
            Err(FetchOneError::NotFound) => Err(Error::NotFound),
            Err(FetchOneError::Unknow) => Err(Error::Unknown),
        },
        Err(_) => Err(Error::BadRequest),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{PokemonName, PokemonNumber, PokemonTypes};
    use crate::repositories::pokemon::InMemoryRepository;

    #[test]
    fn it_should_return_unknow_error_when_an_unexpected_error_happens() {
        let repo = Arc::new(InMemoryRepository::new().with_error());

        let req = Request::new(25);
        let res = execute(repo, req);

        assert!(matches!(res, Err(Error::Unknown)))
    }

    #[test]
    fn it_should_return_bad_request_when_request_is_invalid() {
        let repo = Arc::new(InMemoryRepository::new());

        let req = Request::new(0);
        let res = execute(repo, req);

        assert!(matches!(res, Err(Error::BadRequest)));
    }

    #[test]
    fn it_should_return_not_found_when_request_repo_does_not_contain_pokemon() {
        let repo = Arc::new(InMemoryRepository::new());

        let req = Request::new(25);
        let res = execute(repo, req);

        assert!(matches!(res, Err(Error::NotFound)));
    }

    #[test]
    fn it_should_return_pokemon_otherwise() {
        let repo = Arc::new(InMemoryRepository::new());
        repo.insert(
            PokemonNumber::pikachu(),
            PokemonName::pikachu(),
            PokemonTypes::pikachu(),
        )
        .expect("error inserting pikachu");

        let req = Request::new(25);
        let res = execute(repo, req).expect("error on execute");

        assert_eq!(res.number, 25);
        assert_eq!(res.name, "Pikachu");
        assert_eq!(res.types, vec!["Electric"]);
    }
}
