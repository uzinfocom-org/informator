#![allow(clippy::new_ret_no_self)]

pub mod models;
pub mod schema;

use crate::error::{Error, Result};
use diesel::{
    ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection,
    SqliteExpressionMethods,
    r2d2::{ConnectionManager, Pool},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use models::*;
use r2d2::PooledConnection;
use std::marker::PhantomData;
use teloxide::types::{ChatId, UserId};

// Aliases
pub type Pooling = Pool<ConnectionManager<SqliteConnection>>;
pub type Pooled = PooledConnection<ConnectionManager<SqliteConnection>>;

// Statics
pub const DEFACTO: &[u64] = &[7598454972];
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

// Marker for builder (by order)
pub struct Initializing; // first ensure db exists or can be connected
pub struct Migrating; // then make sure we are catching up with migrations
pub struct Finalizing; // finally, ready to get started!

#[derive(Debug, Clone)]
pub struct Storage {
    database: Pool<ConnectionManager<SqliteConnection>>,
}

#[derive(Debug, Clone)]
pub struct Builder<State> {
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
            database: None,
            _initialized: PhantomData,
        }
    }
}

impl Builder<Initializing> {
    pub async fn connect(
        self,
        path: Option<&str>,
    ) -> std::result::Result<Builder<Migrating>, Error> {
        path.ok_or(Error::NoDatabaseUrl)
            .and_then(|p| Ok(ConnectionManager::<SqliteConnection>::new(p)))
            .and_then(|c| {
                Pool::builder()
                    .max_size(10)
                    .build(c)
                    .map_err(Error::PoolingError)
            })
            .and_then(|p| Builder {
                database: {},
                _initialized: PhantomData,
            })
    }
}

impl Builder<Migrating> {
    pub async fn migrate(self) -> Result<Builder<Finalizing>> {
        self.database
            .as_ref()
            .ok_or(Error::NoDatabaseInstance)?
            .get()?
            .run_pending_migrations(MIGRATIONS)
            .map(|_| Builder {
                database: self.database,
                _initialized: PhantomData,
            })
            .map_err(|_| Error::MigrationError)
    }
}

impl Builder<Finalizing> {
    pub fn build(self) -> Result<Storage> {
        Ok(Storage {
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

    pub fn admins(&self) -> Result<Vec<User>> {
        use schema::users::dsl::*;
        users
            .filter(admin.is(true))
            .select(User::as_select())
            .load(&mut self.database.get()?)
            .map_err(Error::DatabaseError)
    }

    pub fn is_admin(&self, _user: UserId) -> Result<bool> {
        use schema::users::dsl::*;
        Ok(users
            .filter(id.eq(_user.0 as i64))
            .filter(admin.eq(true))
            .select(User::as_select())
            .load(&mut self.database.get()?)
            .map_err(Error::DatabaseError)?
            .iter()
            .map(UserId::from)
            .collect::<Vec<UserId>>()
            .contains(&_user))
    }

    pub fn admin_ids(&self) -> Result<Vec<UserId>> {
        Ok(self
            .admins()?
            .iter()
            .map(|user| user.id)
            .map(|n| UserId(n.to_owned() as u64))
            .collect::<Vec<UserId>>())
    }

    pub fn admin_chats(&self) -> Result<Vec<ChatId>> {
        Ok(self
            .admins()?
            .iter()
            .map(|user| user.chat)
            .filter(|ch| ch.is_some())
            .collect::<Option<Vec<i64>>>()
            .ok_or(Error::WeirdChatId)?
            .iter()
            .map(|n| ChatId(n.to_owned()))
            .collect::<Vec<ChatId>>())
    }

    pub fn exists(&self, _user: UserId) -> Result<bool> {
        use schema::users::dsl::*;
        Ok(users
            .filter(id.eq(_user.0 as i64))
            .select(User::as_select())
            .load(&mut self.database.get()?)
            .map_err(Error::DatabaseError)?
            .iter()
            .map(UserId::from)
            .collect::<Vec<UserId>>()
            .contains(&_user))
    }

    pub fn add_user(&mut self, _user: UserId, _chat: ChatId) -> Result<User> {
        use schema::users::{self};
        match self.exists(_user)? {
            true => Err(Error::ReturningUser),
            false => diesel::insert_into(users::table)
                .values(User {
                    id: _user.0 as i64,
                    chat: Some(_chat.0),
                    admin: DEFACTO.contains(&_user.0),
                    language: "en".to_string(),
                })
                .returning(User::as_returning())
                .get_result(&mut self.database.get()?)
                .map_err(Error::DatabaseError),
        }
    }

    pub fn add_admin(&mut self, _user: UserId) -> Result<()> {
        use schema::users::dsl::*;
        match self.is_admin(_user)? {
            true => Err(Error::ProxyError(
                "User already is in admins list!".to_string(),
            )),
            false => diesel::update(users)
                .filter(id.lt(_user.0 as i64))
                .set(admin.eq(true))
                .execute(&mut self.database.get()?)
                .map_err(Error::DatabaseError)
                .map(|_| ()),
        }
    }
}
