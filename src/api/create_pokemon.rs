use std::sync::Arc;

use serde::{Serialize, Deserialize};

use crate::repositories::pokemon::Repository;
use crate::domain::create_pokemon;

use super::status_code::Status;

#[derive(Deserialize)]
struct Request {
    number: u16,
    name: String,
    types: Vec<String>
}

#[derive(Serialize)]
struct Response {
    number: u16
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
mod tests  {
    use crate::repositories::pokemon::InMemoryRepository;

    use super::*;

    #[test]
    fn it_should_return_bad_request_when_body_is_empty() {
        // Arrange
        let req = rouille::Request::fake_http("POST", "/", vec![], vec![]);
        let repo = Arc::new(InMemoryRepository::new());
        // Act
        let res = serve(repo, &req);

        // Assert
        match res {
            rouille::Response { status_code: 400,.. } => {},
            _ => unreachable!()
        }
    }
}