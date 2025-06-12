pub mod client;
mod model;
pub mod pool;
mod worker;

use std::error::Error;

use thiserror::Error;
pub use worker::WorkerConfig;

#[derive(Error, Debug)]
pub enum ActorSqlError {
    #[error("BuilderSqlError")]
    BuilderSqlError,
    #[error("Literal Error {0}")]
    LiteralError(String),
    #[error("Custom Sqlite Error {0}")]
    CustomRusqliteError(rusqlite::Error),
    #[error("Sqlite Error {0}")]
    RusqliteBuildError(String),
    #[error("Actor Error {0}")]
    ActorError(Box<dyn Error + Send + Sync>),
}

type Result<T> = std::result::Result<T, ActorSqlError>;
type EResult = Result<()>;

pub use rusqlite::types::Value as RsValue;
pub use model::ActorSqliteRow;

impl From<rusqlite::Error> for ActorSqlError {
    fn from(value: rusqlite::Error) -> Self {
        Self::CustomRusqliteError(value)
    }
}

impl<'a> From<&'a str> for ActorSqlError {
    fn from(value: &'a str) -> Self {
        Self::LiteralError(value.to_owned())
    }
}

impl<T: 'static + Send + Sync> From<flume::SendError<T>> for ActorSqlError {
    fn from(value: flume::SendError<T>) -> Self {
        Self::ActorError(Box::new(value))
    }
}

impl From<flume::RecvError> for ActorSqlError {
    fn from(value: flume::RecvError) -> Self {
        Self::ActorError(Box::new(value))
    }
}

impl<T: 'static + Send + Sync> From<oneshot::SendError<T>> for ActorSqlError {
    fn from(value: oneshot::SendError<T>) -> Self {
        Self::ActorError(Box::new(value))
    }
}

impl From<oneshot::RecvError> for ActorSqlError {
    fn from(value: oneshot::RecvError) -> Self {
        Self::ActorError(Box::new(value))
    }
}
