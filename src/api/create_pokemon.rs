use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::domain::create_pokemon;
use crate::repositories::pokemon::Repository;

use super::status_code::Status;

#[derive(Deserialize, Serialize)]
struct Request {
    number: u16,
    name: String,
    types: Vec<String>,
}

#[derive(Serialize)]
struct Response {
    number: u16,
}

pub fn serve(repo: Arc<dyn Repository>, req: &rouille::Request) -> rouille::Response {
    let req = match rouille::input::json_input::<Request>(req) {
        Ok(req) => create_pokemon::Request {
            number: req.number,
            name: req.name,
            types: req.types,
        },
        _ => return rouille::Response::from(Status::BadRequest),
    };

    let res = create_pokemon::execute(repo, req);

    match res {
        create_pokemon::Response::Ok(number) => rouille::Response::json(&Response { number }),
        create_pokemon::Response::Conflict => rouille::Response::from(Status::Conflict),
        create_pokemon::Response::BadRequest => rouille::Response::from(Status::BadRequest),
        create_pokemon::Response::Error => rouille::Response::from(Status::InternalServerError),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        domain::entities::{Pokemon, PokemonName, PokemonNumber, PokemonTypes},
        repositories::pokemon::InMemoryRepository,
    };

    use super::*;

    fn request(body: Option<Request>) -> rouille::Request {
        match body {
            Some(body) => {
                let data = serde_json::to_string(&body).unwrap().into_bytes();
                let headers = vec![("Content-Type".to_owned(), "application/json".to_owned())];
                rouille::Request::fake_http("POST", "/", headers, data)
            }
            None => rouille::Request::fake_http("POST", "/", vec![], vec![]),
        }
    }

    #[test]
    fn it_should_return_bad_request_when_body_is_empty() {
        // Arrange
        let req = request(None);
        let repo = Arc::new(InMemoryRepository::new());

        // Act
        let res = serve(repo, &req);

        // Assert
        assert_eq!(res.status_code, 400);
    }

    #[test]
    fn it_should_return_ok_when_body_is_valid() {
        // Arrange
        let req = Request {
            number: 20,
            name: String::from("Vulpix"),
            types: vec![String::from("Fire")],
        };
        let req = request(Some(req));
        let repo = Arc::new(InMemoryRepository::new());

        // Act
        let res = serve(repo, &req);

        // Assert
        assert_eq!(res.status_code, 200);
    }

    #[test]
    fn it_should_return_server_error_when_repo_error_happens() {
        // Arrange
        let req = Request {
            number: 20,
            name: String::from("Electabuzz"),
            types: vec![String::from("Electric")],
        };
        let req = request(Some(req));
        let repo = Arc::new(InMemoryRepository::new().with_error());

        // Act
        let res = serve(repo, &req);

        // Assert
        assert_eq!(res.status_code, 500);
    }

    #[test]
    fn it_should_return_conflic_when_number_exists() {
        // Arrange
        let req = Request {
            number: 20,
            name: String::from("Electabuzz"),
            types: vec![String::from("Electric")],
        };
        let req = request(Some(req));
        let repo = Arc::new(InMemoryRepository::new().with_error());
        repo.insert(
            PokemonNumber::try_from(20).unwrap(),
            PokemonName::try_from("Pikachu".to_owned()).unwrap(),
            PokemonTypes::try_from(vec!["Electric".to_owned()]).unwrap(),
        );
        // Act
        let res = serve(repo, &req);

        // Assert
        assert_eq!(res.status_code, 409);
    }
}
