#![allow(clippy::new_ret_no_self)]

pub mod prelude;

use crate::error::{Error, Result};
use std::marker::PhantomData;
use teloxide::types::UserId;
use turso::{Builder as TBuilder, Database as TDatabase};

// Marker for builder (by order)
pub struct Initializing;
pub struct Boostrapping;

#[derive(Debug, Clone)]
pub struct Database(TDatabase);

#[derive(Debug)]
pub struct Builder<State> {
    database: Option<TDatabase>,
    _initialized: PhantomData<State>,
}

impl Builder<Initializing> {
    pub async fn connect(self, path: &str) -> Result<Builder<Boostrapping>> {
        let instance = TBuilder::new_local(path)
            .build()
            .await
            .map_err(Error::DatabaseError)?;

        Ok(Builder {
            database: Some(instance),
            _initialized: PhantomData,
        })
    }
}

impl Builder<Boostrapping> {
    pub async fn bootstrap(self) -> Result<Self> {
        // Users tables (* means unique)
        // *Id -> Integer
        // Admin -> Bool
        self.database
            .as_ref()
            .ok_or(Error::NoDatabaseInstance)?
            .connect()?
            .execute(
                r"CREATE TABLE IF NOT EXISTS users (
                    id INTEGER NOT NULL,
                    admin BOOLEAN
                )",
                (),
            )
            .await?;

        // Post states (* means unique)
        // *Id -> Message ID
        // Chat -> Chat ID
        // Selection -> Comma seperated Chat IDs
        self.database
            .as_ref()
            .ok_or(Error::NoDatabaseInstance)?
            .connect()?
            .execute(
                r"CREATE TABLE IF NOT EXISTS messages (
                    id INTEGER NOT NULL,
                    chat INTEGER,
                    selection TEXT
                )",
                (),
            )
            .await?;

        Ok(Self {
            database: self.database,
            _initialized: PhantomData,
        })
    }

    pub fn build(self) -> Result<Database> {
        Ok(Database(self.database.ok_or(Error::NoDatabaseInstance)?))
    }
}

impl Database {
    pub fn new() -> Builder<Initializing> {
        Builder {
            database: None,
            _initialized: PhantomData,
        }
    }

    pub async fn add_user(&self, id: UserId) -> Result<()> {
        let mut statement = self
            .0
            .connect()?
            .prepare("INSERT INTO users (id, admin) VALUES (?1, ?2)")
            .await?;

        statement.execute([id.0, 0]).await?;

        Ok(())
    }

    pub async fn get_admins(&self) -> Result<()> {
        Ok(())
    }
}
