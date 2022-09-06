use chrono::prelude::*;
use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub const CHECKVIST_DATE_FORMAT: &str = "%Y/%m/%d %H:%M:%S %z";

pub trait CheckvistClient {
    fn get_lists(&self) -> Result<Vec<Checklist>, CheckvistError>;
    fn get_list(&self, list_id: u32) -> Result<Checklist, CheckvistError>;
    fn add_list(&self, list_name: &str) -> Result<Checklist, CheckvistError> ;
    fn get_tasks(&self, list_id: u32) -> Result<Vec<Task>, CheckvistError> ;
    fn add_task(&self, list_id: u32, task: &Task) -> Result<Task, CheckvistError> ;
}

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
// only need PartialEq for test, but this doesn't work
// because: integration tests build differently?
// #[cfg_attr(all(test), derive(PartialEq))]
pub struct Checklist {
    pub id: u32,
    pub name: String,
    // Serde does offer a with=mod to do both, but I couldn't get it to pass type checking
    #[serde(deserialize_with = "de_checkvist_date")]
    #[serde(serialize_with = "se_checkvist_date")]
    pub updated_at: DateTime<FixedOffset>,
    pub task_count: u16,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
// TODO - REFACTOR: add updated_at
pub struct Task {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    pub content: String,
    pub position: u16,
}

// Checkvist doesn't use a standard date format, so we custom de/ser
fn de_checkvist_date<'de, D>(de: D) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    // see https://serde.rs/custom-date-format.html
    let s = String::deserialize(de)?;
    let formatted =
        DateTime::parse_from_str(&s, CHECKVIST_DATE_FORMAT).map_err(serde::de::Error::custom)?;

    Ok(formatted)
}

fn se_checkvist_date<S>(list: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format!("{}", list.format(CHECKVIST_DATE_FORMAT));
    serializer.serialize_str(&s)
}

#[derive(Debug)]
pub enum CheckvistError {
    UnknownError { message: String },
    NetworkError(ureq::Error),
    // used by serde_json for decoding errors
    IoError(std::io::Error),
    TokenRefreshFailedError,
    // TODO - REFACTOR: should we really depend on rusqlite here?
    SqliteError(rusqlite::Error)
}

impl fmt::Display for CheckvistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::IoError(ref err) => write!(f, "{:?}", err),
            Self::NetworkError(ref err) => write!(f, "{:?}", err),
            Self::UnknownError { ref message } => write!(f, "{}", message),
            Self::TokenRefreshFailedError => write!(f, "Could not refresh token"),
            Self::SqliteError(ref err) => write!(f, "{:?}", err),
        }
    }
}

impl std::error::Error for CheckvistError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::IoError(ref err) => Some(err),
            Self::NetworkError(ref err) => Some(err),
            Self::UnknownError { message: _ } => None,
            Self::TokenRefreshFailedError => None,
            Self::SqliteError(ref err) => Some(err)
        }
    }
}

impl From<ureq::Error> for CheckvistError {
    fn from(err: ureq::Error) -> Self {
        CheckvistError::NetworkError(err)
    }
}

impl From<std::io::Error> for CheckvistError {
    fn from(err: std::io::Error) -> Self {
        CheckvistError::IoError(err)
    }
}

impl From<rusqlite::Error> for CheckvistError {
    fn from(err: rusqlite::Error) -> Self {
        CheckvistError::SqliteError(err)
    }
}
