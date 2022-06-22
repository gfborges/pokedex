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
    name: String,
    types: Vec<String>,
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
        Ok(res) => rouille::Response::json(&Response {
            number: res.number,
            name: res.name,
            types: res.types,
        }),
        Err(create_pokemon::Error::Conflict) => rouille::Response::from(Status::Conflict),
        Err(create_pokemon::Error::BadRequest) => rouille::Response::from(Status::BadRequest),
        Err(create_pokemon::Error::Unknown) => rouille::Response::from(Status::InternalServerError),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        domain::entities::{PokemonName, PokemonNumber, PokemonTypes},
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
            name: "Electabuzz".to_owned(),
            types: vec!["Electric".to_owned()],
        };
        let req = request(Some(req));
        let repo = Arc::new(InMemoryRepository::new().with_error());

        // Act
        let res = serve(repo, &req);

        // Assert
        assert_eq!(res.status_code, 500);
    }

    #[test]
    fn it_should_return_conflict_when_number_exists() {
        // Arrange
        let req = Request {
            number: 20,
            name: "Electabuzz".to_owned(),
            types: vec!["Electric".to_owned()],
        };
        let req = request(Some(req));
        let repo = Arc::new(InMemoryRepository::new());
        repo.insert(
            PokemonNumber::pikachu(),
            PokemonName::pikachu(),
            PokemonTypes::pikachu(),
        )
        .ok();

        // Act
        let res = serve(repo, &req);

        // Assert
        assert_eq!(res.status_code, 409);
    }
}
