use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use teloxide::types::UserId;

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = super::schema::posts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Post {
    pub id: i32,
    pub chat: i64,
    pub selection: String,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = super::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: i64,
    pub chat: Option<i64>,
    pub admin: bool,
    pub language: String,
}

impl From<User> for UserId {
    fn from(value: User) -> Self {
        UserId(value.id as u64)
    }
}

impl From<&User> for UserId {
    fn from(value: &User) -> Self {
        UserId(value.id as u64)
    }
}
