mod create_pokemon;
mod fetch_all_pokemons;
mod health;
mod status_code;

use std::sync::Arc;

use status_code::Status;

use crate::repositories::pokemon::Repository;

pub fn serve(addr: &str, repo: Arc<dyn Repository>) {
    rouille::start_server(addr, move |req| {
        router!(req,
        (GET) (/health) => {
            health::serve()
        },
        (POST) (/) => {
            create_pokemon::serve(repo.clone(), req)
        },
        (GET) (/) => {
            fetch_all_pokemons::serve(repo.clone())
        },
        _ => {
            rouille::Response::from(Status::NotFound)
        })
    })
}
