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

#[derive(Queryable, Selectable, Serialize, Deserialize, Insertable)]
#[diesel(table_name = super::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: i64,
    pub admin: bool,
    pub language: String,
}

impl User {
    pub fn to_telegram_id(&self) -> UserId {
        UserId(self.id as u64)
    }
}
