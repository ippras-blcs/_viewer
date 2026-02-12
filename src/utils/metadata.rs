use std::collections::HashMap;

use crate::{
    r#const::{IDENTIFIER, TYPE},
    utils::hashed::HashedMetaDataFrame,
};
use chrono::{DateTime, Local, NaiveDate};
use itertools::Itertools as _;
use metadata::{AUTHORS, DATE, DEFAULT_VERSION, DESCRIPTION, Metadata, NAME, PARAMETERS, VERSION};

const DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct MetadataStruct {
    pub(crate) authors: Vec<String>,
    pub(crate) identifier: u64,
    pub(crate) name: String,
    pub(crate) date_time: DateTime<Local>,
    pub(crate) r#type: String,
}

impl From<MetadataStruct> for Metadata {
    fn from(value: MetadataStruct) -> Self {
        let mut metadata = Metadata::new();
        metadata.insert(AUTHORS.to_owned(), value.authors.join(";"));
        metadata.insert(IDENTIFIER.to_owned(), value.identifier.to_string());
        metadata.insert(NAME.to_owned(), value.name.to_owned());
        metadata.insert(DATE.to_owned(), value.date_time.to_string());
        metadata.insert(TYPE.to_owned(), value.r#type.to_owned());
        metadata
    }
}

pub fn join(frames: &[HashedMetaDataFrame]) -> Metadata {
    let mut meta = Metadata::default();
    meta.insert(AUTHORS.to_owned(), authors(frames));
    meta.insert(DATE.to_owned(), date(frames));
    meta.insert(DESCRIPTION.to_owned(), description(frames));
    meta.insert(NAME.to_owned(), name(frames));
    meta.insert(PARAMETERS.to_owned(), parameters(frames));
    meta.insert(VERSION.to_owned(), DEFAULT_VERSION.to_owned());
    meta
}

pub(crate) fn authors(frames: &[HashedMetaDataFrame]) -> String {
    frames
        .iter()
        .flat_map(|frame| frame.meta.get(AUTHORS).map(|authors| authors.split(";")))
        .flatten()
        .unique()
        .sorted()
        .join(";")
}

pub(crate) fn date(frames: &[HashedMetaDataFrame]) -> String {
    let mut date = None;
    for frame in frames {
        date = std::cmp::max(
            date,
            frame
                .meta
                .get(DATE)
                .and_then(|date| NaiveDate::parse_from_str(date, DATE_FORMAT).ok()),
        );
    }
    date.map_or_default(|date| date.to_string())
}

pub(crate) fn description(frames: &[HashedMetaDataFrame]) -> String {
    let descriptions = frames
        .iter()
        .flat_map(|frame| frame.meta.get(DESCRIPTION))
        .map(String::as_str)
        .collect();
    longest_common_prefix(descriptions).to_owned()
}

pub(crate) fn name(frames: &[HashedMetaDataFrame]) -> String {
    let names: Vec<_> = frames
        .iter()
        .filter_map(|frame| frame.meta.get(NAME))
        .map(String::as_str)
        .unique()
        .collect();
    match &*names {
        &[name] => name.to_owned(),
        names => format!("[{}]", names.join(", ")),
    }
}

// pub(crate) fn parameters(frames: &[HashedMetaDataFrame], parameters: &[String]) -> String {
//     frames
//         .iter()
//         .flat_map(|frame| {
//             frame
//                 .meta
//                 .get(PARAMETERS)
//                 .map(|parameter| parameter.split(";"))
//         })
//         .flatten()
//         .chain(parameters.iter().map(|parameter| parameter.deref()))
//         .unique()
//         .sorted()
//         .join(";")
// }
pub(crate) fn parameters(frames: &[HashedMetaDataFrame]) -> String {
    frames
        .iter()
        .flat_map(|frame| {
            frame
                .meta
                .get(PARAMETERS)
                .map(|parameter| parameter.split(";"))
        })
        .flatten()
        .unique()
        .sorted()
        .join(";")
}

// pub(crate) fn parse_parameters(string: &str) -> String {
//     string.split(";")
// }

pub fn longest_common_prefix(strings: Vec<&str>) -> &str {
    if strings.is_empty() {
        return "";
    }
    let mut prefix = strings[0];
    for string in strings {
        while !string.starts_with(prefix) {
            if prefix.is_empty() {
                return "";
            }
            prefix = prefix
                .trim_end_matches(|c| c != '\n')
                .trim_end_matches('\n');
        }
    }
    prefix
}

use nom::{
    IResult, Parser,
    bytes::complete::{is_not, tag, take_till, take_until, take_while},
    character::complete::char,
    combinator::{map, opt, rest},
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub struct Parsed<'a> {
    pub name: &'a str,
    pub parameters: HashMap<&'a str, &'a str>,
    pub version: Option<&'a str>,
    pub timestamp: Option<&'a str>,
}

// #[derive(Debug, PartialEq)]
// pub struct Parameter<'a> {
//     pub key: &'a str,
//     pub value: Option<&'a str>,
// }

pub fn parse(input: &str) -> IResult<&str, Metadata> {
    let mut metadata = Metadata::new();
    // Читаем имя
    let (input, name) = take_till(|c| c == '{' || c == '[' || c == '.')(input)?;
    metadata.insert(NAME.to_owned(), name.to_owned());
    // Читаем параметры (опционально)
    let (input, parameters) = opt(delimited(
        char('{'),
        separated_list0(
            char(';'),
            separated_pair(
                take_until("="),
                char('='),
                take_while(|c| c != ';' && c != '}'),
            ),
        ),
        char('}'),
    ))
    .parse(input)?;
    for (key, value) in parameters.into_iter().flatten() {
        metadata.insert(key.to_owned(), value.to_owned());
    }
    // Читаем версию (опционально)
    let (input, version) = opt(delimited(char('['), take_until("]"), char(']'))).parse(input)?;
    if let Some(version) = version {
        metadata.insert(VERSION.to_owned(), version.to_owned());
    }
    // Читаем дату (опционально)
    let (input, timestamp) = opt(preceded(tag("."), rest)).parse(input)?;
    if let Some(timestamp) = timestamp {
        metadata.insert(DATE.to_owned(), timestamp.to_owned());
    }
    Ok((input, metadata))
}
