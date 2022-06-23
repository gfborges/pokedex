use std::sync::Arc;

mod api;
mod cli;
pub mod domain;
pub mod repositories;

#[macro_use]
extern crate rouille;
#[macro_use]
extern crate clap;
extern crate serde;

use clap::{App, Arg};
use repositories::pokemon::{InMemoryRepository, Repository, SqliteRepository};

fn main() {
    
    let matches = App::new(crate_name!())
    .version(crate_version!())
    .author(crate_authors!())
    .arg(Arg::with_name("cli").long("cli").help("Runs in CLI mode"))
    .arg(Arg::with_name("sqlite").long("sqlite").value_name("PATH"))
    .get_matches();

    let repo = build_repo(matches.value_of("sqlite"));

    match matches.occurrences_of("cli") {
        0 => api::serve("localhost:8000", repo),
        _ => cli::run(repo),
    }
}

fn build_repo(sqlite_path:  Option<&str>) -> Arc<dyn Repository> {
    if let Some(path) = sqlite_path {
        let repo = SqliteRepository::try_new(path).expect("Erro while creating sqlite repo");
        return Arc::new(repo);
    }
    let repo = Arc::new(InMemoryRepository::new());
    repo
}
