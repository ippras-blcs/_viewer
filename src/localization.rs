use egui::{RichText, WidgetText};
use fluent::{concurrent::FluentBundle, FluentResource};
use fluent_content::Content;
use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
    sync::{Arc, LazyLock, RwLock},
};
use tracing::{enabled, error, Level};
use unic_langid::{langid, LanguageIdentifier};

pub(crate) macro lowercase($key:literal) {
    LOCALIZATION
        .read()
        .unwrap()
        .0
        .content($key)
        .unwrap_or_else(|| $key.to_uppercase())
}

pub(crate) macro titlecase($key:literal) {
    match LOCALIZATION.read().unwrap().0.content($key) {
        Some(content) => {
            let mut chars = content.chars();
            chars
                .next()
                .map(char::to_uppercase)
                .into_iter()
                .flatten()
                .chain(chars)
                .collect()
        }
        None => $key.to_uppercase(),
    }
}

macro source($path:literal) {
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), $path))
}

const EN: LanguageIdentifier = langid!("en");
const RU: LanguageIdentifier = langid!("ru");

pub(crate) static LOCALIZATION: LazyLock<RwLock<Localization>> =
    LazyLock::new(|| RwLock::new(Localization::new(Locale::En)));

/// Localization
#[derive(Clone)]
pub(crate) struct Localization(pub(crate) Arc<FluentBundle<FluentResource>>);

impl Localization {
    pub(crate) fn new(locale: Locale) -> Self {
        let mut bundle = FluentBundle::new_concurrent(vec![locale.into()]);
        for &source in sources(locale) {
            let resource = match FluentResource::try_new(source.to_owned()) {
                Ok(resource) => resource,
                Err((resource, errors)) => {
                    if enabled!(Level::WARN) {
                        for error in errors {
                            error!(%error);
                        }
                    }
                    resource
                }
            };
            if let Err(errors) = bundle.add_resource(resource) {
                if enabled!(Level::WARN) {
                    for error in errors {
                        error!(%error);
                    }
                }
            }
        }
        Localization(Arc::new(bundle))
    }

    pub(crate) fn locale(&self) -> Locale {
        match self.0.locales[0] {
            EN => Locale::En,
            RU => Locale::Ru,
            _ => unreachable!(),
        }
    }
}

/// Locale
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum Locale {
    #[default]
    En,
    Ru,
}

impl Locale {
    pub(crate) const fn text(&self) -> &str {
        match self {
            Self::En => "🇺🇸",
            Self::Ru => "🇷🇺",
        }
    }
}

impl From<Locale> for LanguageIdentifier {
    fn from(value: Locale) -> Self {
        match value {
            Locale::En => EN,
            Locale::Ru => RU,
        }
    }
}

const fn sources(locale: Locale) -> &'static [&'static str] {
    match locale {
        Locale::En => &[
            source!("/ftl/en/properties.ftl"),
            source!("/ftl/en/pane_settings.ftl"),
        ],
        Locale::Ru => &[
            source!("/ftl/ru/properties.ftl"),
            source!("/ftl/ru/pane_settings.ftl"),
        ],
    }
}
