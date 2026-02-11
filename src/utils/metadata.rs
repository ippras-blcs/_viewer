use crate::utils::hashed::HashedMetaDataFrame;
use chrono::NaiveDate;
use itertools::Itertools as _;
use metadata::{AUTHORS, DATE, DEFAULT_VERSION, DESCRIPTION, Metadata, NAME, PARAMETERS, VERSION};

const DATE_FORMAT: &str = "%Y-%m-%d";

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
