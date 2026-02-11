use self::{cache::Cache, settings::Settings, windows::Windows};
use egui::{Context, Id};
use serde::{Deserialize, Serialize};

pub(crate) const ID_SOURCE: &str = "Pane";

/// Configuration state
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct State {
    // #[serde(skip)]
    pub(crate) cache: Cache,
    #[serde(skip)]
    pub(crate) event: Event,
    pub(crate) settings: Settings,
    pub(crate) windows: Windows,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            cache: Cache::new(),
            event: Event::new(),
            settings: Settings::new(),
            windows: Windows::new(),
        }
    }
}

impl State {
    pub(crate) fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|data| data.get_persisted_mut_or_insert_with(id, Self::new).clone())
    }

    pub(crate) fn remove(self, ctx: &Context, id: Id) {
        ctx.data_mut(|data| {
            data.remove::<Self>(id);
        });
    }

    pub(crate) fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|data| {
            data.insert_persisted(id, self);
        });
    }
}

/// Event
#[derive(Clone, Copy, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Event {
    pub(crate) reset_table_state: bool,
}

impl Event {
    pub(crate) fn new() -> Self {
        Self {
            reset_table_state: false,
        }
    }
}

pub(crate) mod cache;
pub(crate) mod settings;
pub(crate) mod windows;
