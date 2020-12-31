#[macro_use]
extern crate lazy_static;
extern crate rocket;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::io::Error;

pub use card::Card;
pub use review::Review;

mod card;
mod review;
mod repo;
pub mod cors;
pub mod error;

#[derive(Debug)]
pub enum GitError {
    Generic(String),
}

impl From<std::io::Error> for GitError {
    fn from(err: Error) -> Self {
        GitError::Generic(err.to_string())
    }
}

pub fn test_repo() -> Result<(), GitError> {
    let mut repo = repo::Repo::new("https://github.com/openalcoholics/drinking-game-cards".to_string(), "dgc".to_string(), "v2".to_string())?;

    repo.reset()?;
    repo.checkout("feature/i18n".to_string())?;

    Ok(())
}
