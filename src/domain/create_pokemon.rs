use std::sync::Arc;

use crate::domain::entities::{PokemonName, PokemonNumber, PokemonTypes};
use crate::repositories::pokemon::{InsertError, Repository};

pub struct Request {
    pub number: u16,
    pub name: String,
    pub types: Vec<String>,
}

#[derive(Debug)]
pub struct Response {
    pub number: u16,
    pub name: String,
    pub types: Vec<String>,
}

pub enum Error {
    BadRequest,
    Conflict,
    Unknown,
}

pub fn execute(repo: Arc<dyn Repository>, req: Request) -> Result<Response, Error> {
    match (
        PokemonNumber::try_from(req.number),
        PokemonName::try_from(req.name),
        PokemonTypes::try_from(req.types),
    ) {
        (Ok(number), Ok(name), Ok(types)) => match repo.insert(number, name.clone(), types.clone())
        {
            Ok(pokemon) => Ok(Response {
                number: u16::from(pokemon.number),
                name: String::from(pokemon.name),
                types: Vec::<String>::from(pokemon.types),
            }),
            Err(InsertError::Conflict) => Err(Error::Conflict),
            _ => Err(Error::Unknown),
        },
        _ => Err(Error::BadRequest),
    }
}

#[cfg(test)]
mod tests {

    use crate::repositories::inmemory_pokemon::InMemoryRepository;

    use super::*;

    #[test]
    fn it_should_return_the_pokemon_otherwise() {
        let repo = Arc::new(InMemoryRepository::new());
        let req = Request {
            number: 25,
            name: String::from("Pikachu"),
            types: vec![String::from("Electric")],
        };

        let res = execute(repo, req);

        match res {
            Ok(res) => {
                assert_eq!(res.number, 25);
                assert_eq!(res.name, "Pikachu".to_owned());
                assert_eq!(res.types, vec!["Electric".to_owned()]);
            }
            _ => unreachable!("execute returned an error"),
        }
    }

    #[test]
    fn it_should_return_a_bad_request_error_when_request_is_invalid() {
        let repo = Arc::new(InMemoryRepository::new());
        let req = Request {
            number: 25,
            name: String::from(""),
            types: vec![String::from("Electric")],
        };

        let res = execute(repo, req);

        match res {
            Err(Error::BadRequest) => {}
            _ => unreachable!(),
        };
    }

    #[test]
    fn it_should_return_a_conflict_error_when_pokemon_already_exists() {
        let repo = Arc::new(InMemoryRepository::new());
        let number = PokemonNumber::pikachu();
        let name = PokemonName::pikachu();
        let types = PokemonTypes::pikachu();
        repo.insert(number, name, types).ok();

        let req = Request {
            number: 25,
            name: String::from("Charmander"),
            types: vec![String::from("Fire")],
        };
        let res = execute(repo, req);

        assert!(matches!(res, Err(Error::Conflict)), "execute didn't return conflict");
    }

    #[test]
    fn it_should_return_an_error_when_an_unexpected_error_happens() {
        let repo = Arc::new(InMemoryRepository::new().with_error());
        let number = 25;
        let req = Request {
            number,
            name: String::from("Pikachu"),
            types: vec![String::from("Electric")],
        };

        let res = execute(repo, req);

        match res {
            Err(Error::Unknown) => {}
            _ => unreachable!(),
        }
    }
}
