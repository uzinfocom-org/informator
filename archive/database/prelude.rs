use crate::error::{Error, Result};
use teloxide::types::{ChatId, MessageId, UserId};
use turso::Row;

pub trait Extractable<T>: Sized {
    fn extract(value: T) -> Result<Self>;
}

#[derive(Debug)]
pub struct User {
    pub id: UserId,
    pub admin: bool,
}

impl Extractable<Row> for User {
    fn extract(value: Row) -> Result<Self> {
        Ok(Self {
            id: UserId(value.get(0).map_err(Error::DatabaseError)?),
            admin: value.get(1).map_err(Error::DatabaseError)?,
        })
    }
}

#[derive(Debug)]
pub struct Message {
    pub id: MessageId,
    pub chat: ChatId,
    pub selection: Vec<ChatId>,
}

impl Extractable<Row> for Message {
    fn extract(value: Row) -> Result<Self> {
        Ok(Self {
            id: MessageId(value.get(0).map_err(Error::DatabaseError)?),
            chat: ChatId(value.get(1).map_err(Error::DatabaseError)?),
            selection: value
                .get::<String>(2)
                .map_err(Error::DatabaseError)?
                .split(',')
                .map(|s| s.parse::<i64>())
                // Vec<Result<i64>> -> Result<Vec<i64>>
                .collect::<std::result::Result<Vec<i64>, _>>()?
                .iter()
                .map(|n| ChatId(n.to_owned()))
                .collect(),
        })
    }
}
