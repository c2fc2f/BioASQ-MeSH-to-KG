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

fn escaped_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?
        .replace('"', "\"\"")
        .replace("\\n", " ")
        .replace('\n', " ")
        .trim()
        .to_owned();

    Ok(s)
}

fn escaped_string_set<'de, D>(
    deserializer: D,
) -> Result<HashSet<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct EscapedStringVisitor;

    impl<'de> serde::de::Visitor<'de> for EscapedStringVisitor {
        type Value = HashSet<String>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a sequence of strings")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut set: HashSet<String> = HashSet::new();
            while let Some(s) = seq.next_element::<String>()? {
                set.insert(
                    s.replace('"', "\"\"")
                        .replace("\\n", " ")
                        .replace('\n', " ")
                        .trim()
                        .to_owned(),
                );
            }
            Ok(set)
        }
    }

    deserializer.deserialize_seq(EscapedStringVisitor)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BioASQEntry {
    #[serde(rename = "abstractText", deserialize_with = "escaped_string")]
    pub(crate) r#abstract: String,

    #[serde(deserialize_with = "escaped_string")]
    pub(crate) journal: String,

    #[serde(rename = "meshMajor", deserialize_with = "escaped_string_set")]
    pub(crate) mesh: HashSet<String>,

    #[serde(deserialize_with = "parse_from_str")]
    pub(crate) pmid: u32,

    #[serde(deserialize_with = "escaped_string")]
    pub(crate) title: String,

    #[serde(deserialize_with = "parse_from_opt_str")]
    pub(crate) year: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BioASQDataset {
    pub(crate) articles: Vec<BioASQEntry>,
}
