use std::{env::VarError, num::ParseIntError};
use thiserror::Error;

/// Normal fucking Result
pub type Result<T> = std::result::Result<T, Error>;

/// Expression of insanity
pub type Insane = Box<dyn std::error::Error + Send + Sync>;

/// Sanity Destroyer 3000
pub type Insult<T> = std::result::Result<T, Insane>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Are you SURE that database is initialized?!")]
    NoDatabaseInstance,
    #[error("database url where?! {0}")]
    NoDatabaseUrl(#[from] VarError),
    #[error("trouble with db connection: {0}")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("trouble while creating pool of connections: {0}")]
    PoolingError(#[from] r2d2::Error),
    #[error("trouble while migrating migrations.")]
    MigrationError,
    #[error("trouble with parsing number: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("is this really a valid db path?")]
    InvalidPath,
}

// To pass the Insult's criterias
unsafe impl Send for Error {}
unsafe impl Sync for Error {}
