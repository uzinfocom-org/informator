use std::{env::VarError, num::ParseIntError};
use thiserror::Error;

/// Normal fucking Result
pub type Result<T> = std::result::Result<T, Error>;

/// Expression of insanity
pub type Insane = Box<dyn std::error::Error + Send + Sync>;

/// Sanity Destroyer 3000
pub type Insult<T> = std::result::Result<T, Insane>;

pub struct ErrorStruct(Option<String>);

impl From<VarError> for ErrorStruct {
    fn from(value: VarError) -> Self {
        Self(Some(value.to_string()))
    }
}

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
    #[error("this chat_id seems very weird")]
    WeirdChatId,
    #[error("is this really a valid db path?")]
    InvalidPath,
    #[error("this user seems to be already in database.")]
    ReturningUser,
    #[error("{0}")] // passing string for a reason
    ProxyError(String),

    /// Don't you even dare to touch this one!!!
    /// Must be avoided at any cost for sanity preservation.
    #[error("baba yaga is bullshitting somewhere: {0}")]
    Unknown(String),
}

// To pass the Insult's criterias
unsafe impl Send for Error {}
unsafe impl Sync for Error {}
