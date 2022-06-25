use serde::Deserialize;

use super::pokemon::{DeleteError, FetchAllError, FetchOneError, InsertError, Repository};
use crate::domain::entities::{Pokemon, PokemonName, PokemonNumber, PokemonTypes};

pub struct AirtableRepository {
    url: String,
    auth_header: String,
}

#[derive(Deserialize)]
struct AirtableJson {
    records: Vec<AirtableRecord>,
}

#[derive(Deserialize)]
struct AirtableRecord {
    id: String,
    fields: AirtableFields,
}

#[derive(Deserialize)]
struct AirtableFields {
    pub number: u16,
    pub name: String,
    pub types: Vec<String>,
}

impl AirtableRepository {
    pub fn try_new(apikey: &str, workspace_id: &str) -> Result<Self, ()> {
        let url = format!("https://api.airtable.com/v0/{}/pokemons", workspace_id);
        let auth_header = format!("Bearer {}", apikey);

        if let Err(_) = ureq::get(&url).set("Authorization", &auth_header).call() {
            return Err(());
        }

        Ok(Self { url, auth_header })
    }

    fn fetch_pokemon_rows(&self, number: Option<u16>) -> Result<AirtableJson, ()> {
        let url = match number {
            Some(number) => format!("{}?filterByFormula=number%3D{}", self.url, number),
            None => format!("{}?sort%5B0%5D%5Bfield%5D=number", self.url),
        };

        let res = match ureq::get(&url)
            .set("Authorization", &self.auth_header)
            .call()
        {
            Ok(res) => res,
            Err(e) => {
                println!("error calling airtable: {e}");
                return Err(());
            }
        };

        match res.into_json() {
            Ok(json) => Ok(json),
            Err(e) => {
                println!("error deserializing json: {e}");
                Err(())
            }
        }
    }
}

#[cfg(test)]
impl AirtableRepository {
    pub fn new_test(url: &str, apikey: &str) -> Self {
        let auth_header = format!("Bearer {apikey}");
        Self {
            url: url.to_owned(),
            auth_header,
        }
    }
}

impl Repository for AirtableRepository {
    fn insert(
        &self,
        number: PokemonNumber,
        name: PokemonName,
        types: PokemonTypes,
    ) -> Result<Pokemon, InsertError> {
        let json = match self.fetch_pokemon_rows(Some(u16::from(number.clone()))) {
            Ok(json) => json,
            _ => return Err(InsertError::Unknown),
        };

        if !json.records.is_empty() {
            return Err(InsertError::Conflict);
        }

        let body = ureq::json!({
            "records": [{
                "fields": {
                    "number": u16::from(number.clone()),
                    "name": String::from(name.clone()),
                    "types": Vec::<String>::from(types.clone()),
                },
            }],
        });

        if let Err(e) = ureq::post(&self.url)
            .set("Authorization", &self.auth_header)
            .send_json(body)
        {
            println!("error inserting pokemon({:?}) on airtable: {e}", number);
            return Err(InsertError::Unknown);
        }

        Ok(Pokemon::new(number, name, types))
    }

    fn fetch_all(&self) -> Result<Vec<Pokemon>, FetchAllError> {
        let json = match self.fetch_pokemon_rows(None) {
            Ok(json) => json,
            Err(_) => return Err(FetchAllError::Unknown),
        };

        let mut pokemons = Vec::with_capacity(json.records.len());
        for record in json.records {
            match (
                PokemonNumber::try_from(record.fields.number),
                PokemonName::try_from(record.fields.name),
                PokemonTypes::try_from(record.fields.types),
            ) {
                (Ok(number), Ok(name), Ok(types)) => {
                    pokemons.push(Pokemon::new(number, name, types))
                }
                _ => {
                    println!("error parsing pokemon({})", record.fields.number);
                    return Err(FetchAllError::Unknown);
                }
            }
        }

        Ok(pokemons)
    }

    fn fetch_one(&self, number: PokemonNumber) -> Result<Pokemon, FetchOneError> {
        let number = u16::from(number);
        let mut json = match self.fetch_pokemon_rows(Some(number)) {
            Ok(json) => json,
            Err(_) => return Err(FetchOneError::Unknown),
        };

        if json.records.is_empty() {
            return Err(FetchOneError::NotFound);
        }

        let fields = json.records.remove(0).fields;
        match (
            PokemonNumber::try_from(fields.number),
            PokemonName::try_from(fields.name),
            PokemonTypes::try_from(fields.types),
        ) {
            (Ok(number), Ok(name), Ok(types)) => Ok(Pokemon::new(number, name, types)),
            _ => Err(FetchOneError::Unknown),
        }
    }

    fn delete(&self, number: PokemonNumber) -> Result<(), DeleteError> {
        let mut json = match self.fetch_pokemon_rows(Some(u16::from(number.clone()))) {
            Ok(json) => json,
            _ => return Err(DeleteError::Unknown),
        };

        if json.records.is_empty() {
            return Err(DeleteError::NotFound);
        }

        let record = json.records.remove(0);
        let path = format!("{}/{}", self.url, record.id);
        let req = ureq::delete(&path)
            .set("Authorization", &self.auth_header)
            .call();

        if let Err(e) = req {
            println!("error deleting pokemon({:?}) on airtable: {e}", number);
            return Err(DeleteError::Unknown);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::PokemonNumber;
    use httpmock::prelude;
    use serde_json::json;

    const APIKEY: &str = "TEST-KEY";

    #[test]
    fn it_should_create_repository_with_url_and_apikey() {
        let server = prelude::MockServer::start();
        let url = server.url("/test/api");
        let repo = AirtableRepository::new_test(url.as_str(), APIKEY);

        assert_eq!(repo.url, server.url("/test/api"));
        assert_eq!(repo.auth_header, "Bearer TEST-KEY");
    }

    #[test]
    fn it_should_fail_to_delete_when_pokemon_does_not_exist() {
        let server = prelude::MockServer::start();
        let url = server.url("/test/api");
        let repo = AirtableRepository::new_test(url.as_str(), APIKEY);

        let pokedex_mock = server.mock(|when, then| {
            when.method(prelude::GET).path("/test/api");
            then.status(200).json_body(json!({"records": []}));
        });

        let err = repo
            .delete(PokemonNumber::pikachu())
            .expect_err("should have returned error on delete");

        pokedex_mock.assert();
        assert!(matches!(err, DeleteError::NotFound));
    }

    #[test]
    fn it_should_fail_to_delete_when_json_is_incorrect() {
        let server = prelude::MockServer::start();
        let url = server.url("/test/api");
        let repo = AirtableRepository::new_test(url.as_str(), APIKEY);

        let pokedex_mock = server.mock(|when, then| {
            when.method(prelude::GET).path("/test/api");
            then.status(200).json_body(json!({"records": [{
                "id": "ID",
                "fields": {
                    "number": ""
                }
            }]}));
        });

        let err = repo
            .delete(PokemonNumber::pikachu())
            .expect_err("should have returned error on delete");

        pokedex_mock.assert();
        assert!(matches!(err, DeleteError::Unknown));
    }

    #[test]
    fn it_should_fail_to_delete_when_pokemon_fetching_fails() {
        let server = prelude::MockServer::start();
        let url = server.url("/test/api");
        let repo = AirtableRepository::new_test(url.as_str(), APIKEY);

        let pokedex_mock = server.mock(|when, then| {
            when.method(prelude::GET).path("/test/api");
            then.status(500);
        });

        let err = repo
            .delete(PokemonNumber::pikachu())
            .expect_err("should have returned error on delete");

        pokedex_mock.assert();
        assert!(matches!(err, DeleteError::Unknown));
    }

    #[test]
    fn it_should_fail_to_delete_when_delete_request_fails() {
        let server = prelude::MockServer::start();
        let url = server.url("/test/api");
        let repo = AirtableRepository::new_test(url.as_str(), APIKEY);

        let get_route = server.mock(|when, then| {
            when.method(prelude::GET).path("/test/api");
            then.status(200).json_body(json!(
            {"records": [{
                "id":"ID",
                "fields": {
                    "number": 25u16,
                    "name": "pikachu",
                    "types": ["Electric"]
                }
            }]}));
        });

        let delete_route = server.mock(|when, then| {
            when.method(prelude::DELETE).path("/test/api/ID");
            then.status(500);
        });

        let err = repo
            .delete(PokemonNumber::pikachu())
            .expect_err("should have returned error on delete");

        assert!(matches!(err, DeleteError::Unknown));
        assert_eq!(get_route.hits(), 1);
        assert_eq!(delete_route.hits(), 1);
    }

    #[test]
    fn it_should_delete_otherwise() {
        let server = prelude::MockServer::start();
        let url = server.url("/test/api");
        let repo = AirtableRepository::new_test(url.as_str(), APIKEY);

        let get_route = server.mock(|when, then| {
            when.method(prelude::GET).path("/test/api");
            then.status(200).json_body(json!(
            {"records": [{
                "id":"ID",
                "fields": {
                    "number": 25u16,
                    "name": "pikachu",
                    "types": ["Electric"]
                }
            }]}));
        });

        let delete_route = server.mock(|when, then| {
            when.method(prelude::DELETE).path("/test/api/ID");
            then.status(200);
        });

        let res = repo
            .delete(PokemonNumber::pikachu());

        assert!(res.is_ok());
        assert_eq!(get_route.hits(), 1);
        assert_eq!(delete_route.hits(), 1);
    }
    
}
