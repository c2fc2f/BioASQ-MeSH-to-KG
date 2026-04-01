use std::{collections::HashSet, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize};

fn parse_from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let s: String = String::deserialize(deserializer)?;
    s.parse::<T>().map_err(serde::de::Error::custom)
}

fn parse_from_opt_str<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let s: Option<String> = Option::<String>::deserialize(deserializer)?;
    if s.is_none() {
        return Ok(None);
    }
    s.unwrap()
        .parse::<T>()
        .map(|s| Some(s))
        .map_err(serde::de::Error::custom)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BioASQEntry {
    #[serde(rename = "abstractText")]
    pub(crate) r#abstract: String,

    pub(crate) journal: String,

    #[serde(rename = "meshMajor")]
    pub(crate) mesh: HashSet<String>,

    #[serde(deserialize_with = "parse_from_str")]
    pub(crate) pmid: u32,

    pub(crate) title: String,

    #[serde(deserialize_with = "parse_from_opt_str")]
    pub(crate) year: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BioASQDataset {
    pub(crate) articles: Vec<BioASQEntry>,
}
