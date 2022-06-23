use std::sync::{Mutex, MutexGuard};

use rusqlite::{params, params_from_iter, Connection, OpenFlags};
use serde::Deserialize;

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

pub struct InMemoryRepository {
    error: bool,
    pokemons: Mutex<Vec<Pokemon>>,
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

pub struct SqliteRepository {
    conn: Mutex<Connection>,
}

impl SqliteRepository {
    pub fn try_new(path: &str) -> Result<Self, ()> {
        let conn = match Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE) {
            Ok(conn) => conn,
            Err(_) => return Err(()),
        };
        match conn.execute("pragma foreign_keys =1", []) {
            Ok(_) => Ok(Self {
                conn: Mutex::new(conn),
            }),
            Err(_) => Err(()),
        }
    }

    fn fetch_pokemon_rows(
        lock: &MutexGuard<'_, Connection>,
        number: Option<u16>,
    ) -> Result<Vec<(u16, String)>, ()> {
        let (query, params) = match number {
            Some(number) => (
                "select number, name from pokemons where number = ?",
                vec![number],
            ),
            None => ("select number, name from pokemons", vec![]),
        };

        let mut stmt = match lock.prepare(query) {
            Ok(s) => s,
            Err(_) => return Err(()),
        };

        let mut rows = match stmt.query(params_from_iter(params)) {
            Ok(rows) => rows,
            Err(_) => return Err(()),
        };

        let mut pokemon_rows = vec![];

        while let Ok(Some(row)) = rows.next() {
            match (row.get::<usize, u16>(0), row.get::<usize, String>(1)) {
                (Ok(number), Ok(name)) => pokemon_rows.push((number, name)),
                _ => return Err(()),
            }
        }
        Ok(pokemon_rows)
    }

    fn fetch_type_rows(lock: &MutexGuard<'_, Connection>, number: u16) -> Result<Vec<String>, ()> {
        let mut stmt = match lock.prepare("select name from types where pokemon_number = ?") {
            Ok(s) => s,
            Err(_) => return Err(()),
        };

        let mut rows = match stmt.query([number]) {
            Ok(rows) => rows,
            _ => return Err(()),
        };

        let mut type_rows = vec![];
        while let Ok(Some(row)) = rows.next() {
            match row.get::<usize, String>(0) {
                Ok(name) => type_rows.push(name),
                Err(_) => return Err(()),
            }
        }
        Ok(type_rows)
    }
}

impl Repository for SqliteRepository {
    fn insert(
        &self,
        number: PokemonNumber,
        name: PokemonName,
        types: PokemonTypes,
    ) -> Result<Pokemon, InsertError> {
        let mut lock = match self.conn.lock() {
            Ok(lock) => lock,
            Err(_) => return Err(InsertError::Unknown),
        };
        let transaction = match lock.transaction() {
            Ok(t) => t,
            Err(e) => {
                println!("error while starting transaction: {e}");
                return Err(InsertError::Unknown);
            }
        };

        match transaction.execute(
            "insert into pokemons values (?, ?)",
            params![u16::from(number.clone()), String::from(name.clone())],
        ) {
            Ok(_) => {}
            Err(rusqlite::Error::SqliteFailure(_, Some(msg))) => {
                if msg == "UNIQUE constraint failed: pokemon.number" {
                    return Err(InsertError::Conflict);
                }
                return Err(InsertError::Unknown);
            }
            Err(e) => {
                println!("error while inserting pokemon: {e}");
                return Err(InsertError::Unknown);
            }
        }

        for tipe in Vec::from(types.clone()) {
            if let Err(e) = transaction.execute(
                "insert into types values(?, ?)",
                params![u16::from(number.clone()), tipe],
            ) {
                println!("error in inserting type: {e}");
                return Err(InsertError::Unknown);
            }
        }

        match transaction.commit() {
            Ok(_) => Ok(Pokemon::new(number, name, types)),
            Err(e) => {
                println!("error while commiting transaction: {e}");
                Err(InsertError::Unknown)
            }
        }
    }

    fn fetch_all(&self) -> Result<Vec<Pokemon>, FetchAllError> {
        let lock = match self.conn.lock() {
            Ok(lock) => lock,
            Err(_) => return Err(FetchAllError::Unknown),
        };

        let pokemon_rows = match Self::fetch_pokemon_rows(&lock, None) {
            Ok(rows) => rows,
            Err(_) => return Err(FetchAllError::Unknown),
        };

        let mut pokemons = Vec::with_capacity(pokemon_rows.len());
        for pokemon_row in pokemon_rows {
            let type_rows = match Self::fetch_type_rows(&lock, pokemon_row.0) {
                Ok(rows) => rows,
                Err(_) => return Err(FetchAllError::Unknown),
            };
            let pokemon = match (
                PokemonNumber::try_from(pokemon_row.0),
                PokemonName::try_from(pokemon_row.1),
                PokemonTypes::try_from(type_rows),
            ) {
                (Ok(number), Ok(name), Ok(types)) => Pokemon::new(number, name, types),
                _ => return Err(FetchAllError::Unknown),
            };

            pokemons.push(pokemon);
        }

        Ok(pokemons)
    }

    fn fetch_one(&self, number: PokemonNumber) -> Result<Pokemon, FetchOneError> {
        let lock = match self.conn.lock() {
            Ok(lock) => lock,
            Err(_) => return Err(FetchOneError::Unknown),
        };

        let number = u16::from(number);
        let mut pokemon_rows = match Self::fetch_pokemon_rows(&lock, Some(number)) {
            Ok(rows) => rows,
            Err(_) => return Err(FetchOneError::Unknown),
        };
        if pokemon_rows.is_empty() {
            return Err(FetchOneError::NotFound);
        }
        let pokemon_row = pokemon_rows.remove(0);

        let type_rows = match Self::fetch_type_rows(&lock, number) {
            Ok(rows) => rows,
            Err(_) => return Err(FetchOneError::Unknown),
        };

        match (
            PokemonNumber::try_from(pokemon_row.0),
            PokemonName::try_from(pokemon_row.1),
            PokemonTypes::try_from(type_rows),
        ) {
            (Ok(number), Ok(name), Ok(types)) => Ok(Pokemon::new(number, name, types)),
            _ => Err(FetchOneError::Unknown),
        }
    }

    fn delete(&self, number: PokemonNumber) -> Result<(), DeleteError> {
        let lock = match self.conn.lock() {
            Ok(lock) => lock,
            Err(_) => return Err(DeleteError::Unknown),
        };

        match lock.execute(
            "delete from pokemons where number = ?",
            params![u16::from(number)],
        ) {
            Ok(0) => Err(DeleteError::NotFound),
            Ok(_) => Ok(()),
            _ => Err(DeleteError::Unknown),
        }
    }
}

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

        match res.into_json::<AirtableJson>() {
            Ok(json) => Ok(json),
            Err(e) => {
                println!("error deserializing json: {e}");
                Err(())
            }
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
                _ => return Err(FetchAllError::Unknown),
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

        if !json.records.is_empty() {
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
