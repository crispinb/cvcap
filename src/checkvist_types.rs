use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub const CHECKVIST_DATE_FORMAT: &str = "%Y/%m/%d %H:%M:%S %z";

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

// Checkvist doesn't use a standard date format, so we custom de/ser
fn de_checkvist_date<'de, D>(de: D) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    // see https://serde.rs/custom-date-format.html
    let s = String::deserialize(de)?;
    dbg!(&s);
    let formatted = DateTime::parse_from_str(&s, CHECKVIST_DATE_FORMAT).map_err(serde::de::Error::custom)?;

    Ok(formatted)
}

fn se_checkvist_date<S>(list: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
where S: Serializer {

    let s = format!("{}", list.format(CHECKVIST_DATE_FORMAT));
    serializer.serialize_str(&s)
}