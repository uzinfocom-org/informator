#![allow(clippy::new_ret_no_self)]

pub mod models;
pub mod schema;

use crate::{
    error::{Error, Result},
    storage::schema::users,
};
use diesel::{
    Connection, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection, SqliteExpressionMethods,
    r2d2::{ConnectionManager, Pool},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use models::*;
use r2d2::PooledConnection;
use std::{env, marker::PhantomData};
use teloxide::types::UserId;

// Aliases
pub type Pooling = Pool<ConnectionManager<SqliteConnection>>;
pub type Pooled = PooledConnection<ConnectionManager<SqliteConnection>>;

// Statics
pub const DEFACTO: &[u64] = &[7598454972];
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

// Marker for builder (by order)
pub struct Initializing; // first ensure db exists or can be connected
pub struct Migrating; // then make sure we are catching up with migrations
pub struct Syncing; // if there are other states, sync with db for caching
pub struct Finalizing; // finally, ready to get started!

#[derive(Debug, Clone)]
pub struct Storage {
    admins: Vec<UserId>,
    database: Pool<ConnectionManager<SqliteConnection>>,
}

#[derive(Debug, Clone)]
pub struct Builder<State> {
    admins: Vec<UserId>,
    database: Option<Pooling>,
    _initialized: PhantomData<State>,
}

// To be able to do black magic through threading jobs
// Please, don't even think about wrapping storage to Arc!
unsafe impl Send for Storage {}
unsafe impl Sync for Storage {}

impl Default for Builder<Initializing> {
    fn default() -> Self {
        Self {
            admins: DEFACTO.iter().map(|u| UserId(u.to_owned())).collect(),
            database: None,
            _initialized: PhantomData,
        }
    }
}

impl Builder<Initializing> {
    pub async fn connect(self, path: Option<&str>) -> Result<Builder<Migrating>> {
        let path = match path {
            Some(p) => p,
            None => &env::var("DATABASE_URL").map_err(Error::NoDatabaseUrl)?,
        };

        let manager = ConnectionManager::<SqliteConnection>::new(path);
        let instance = Pool::builder()
            .max_size(10)
            .build(manager)
            .map_err(Error::PoolingError)?;

        Ok(Builder {
            admins: self.admins,
            database: Some(instance),
            _initialized: PhantomData,
        })
    }
}

impl Builder<Migrating> {
    pub async fn migrate(self) -> Result<Builder<Syncing>> {
        self.database
            .as_ref()
            .ok_or(Error::NoDatabaseInstance)?
            .get()?
            .run_pending_migrations(MIGRATIONS)
            .map_err(|_| Error::MigrationError)?;

        Ok(Builder {
            admins: self.admins,
            database: self.database,
            _initialized: PhantomData,
        })
    }
}

impl Builder<Syncing> {
    pub async fn sync(self) -> Result<Builder<Finalizing>> {
        let mut connection = self
            .database
            .as_ref()
            .ok_or(Error::NoDatabaseInstance)?
            .get()?;

        let admins: Vec<UserId> = connection
            .transaction(|c| {
                use schema::users::dsl::*;
                users
                    .filter(admin.is(true))
                    .select(User::as_select())
                    .load(c)
            })
            .map_err(Error::DatabaseError)?
            .iter()
            .to_owned()
            .map(|u| u.to_telegram_id())
            .collect();

        Ok(Builder {
            admins: [self.admins, admins].concat(),
            database: self.database,
            _initialized: PhantomData,
        })
    }
}

impl Builder<Finalizing> {
    pub fn build(self) -> Result<Storage> {
        Ok(Storage {
            admins: self.admins,
            database: self.database.ok_or(Error::NoDatabaseInstance)?,
        })
    }
}

impl Storage {
    pub fn new() -> Builder<Initializing> {
        Builder::<Initializing>::default()
    }

    pub fn conn(&self) -> Result<Pooled> {
        self.database.get().map_err(Error::PoolingError)
    }

    pub fn sync(&mut self) -> Result<&Vec<UserId>> {
        let admins: Vec<UserId> = self
            .conn()?
            .transaction(|c| {
                use schema::users::dsl::*;
                users
                    .filter(admin.is(true))
                    .select(User::as_select())
                    .load(c)
            })
            .map_err(Error::DatabaseError)?
            .iter()
            .to_owned()
            .map(|u| u.to_telegram_id())
            .collect();

        self.admins = [
            admins,
            DEFACTO.iter().map(|u| UserId(u.to_owned())).collect(),
        ]
        .concat();

        Ok(&self.admins)
    }

    pub fn add_admin(&mut self, user: UserId) -> Result<User> {
        self.sync()?;

        if DEFACTO.contains(&user.0) {
            return Err(Error::ProxyError(
                "User already is a DEFACTO admin!".to_string(),
            ));
        }

        if self.admins.contains(&user) {
            return Err(Error::ProxyError(
                "User already is in admins list!".to_string(),
            ));
        }

        self.conn()?
            .transaction(|c| {
                diesel::insert_into(users::table)
                    .values(User {
                        id: user.0 as i64,
                        admin: true,
                        language: "en".to_string(),
                    })
                    .returning(User::as_returning())
                    .get_result(c)
            })
            .map_err(Error::DatabaseError)
    }
}
