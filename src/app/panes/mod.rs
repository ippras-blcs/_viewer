use crate::{
    app::mqtt::{
        TOPIC_ATUC, TOPIC_DDOC_C1, TOPIC_DDOC_C2, TOPIC_DDOC_T1, TOPIC_DDOC_T2, TOPIC_DDOC_V1,
        TOPIC_DDOC_V2, TOPIC_DTEC,
    },
    utils::hashed::HashedMetaDataFrame,
};
use egui::{Ui, Vec2, WidgetText, vec2};
use egui_tiles::{TileId, UiResponse};
use serde::{Deserialize, Serialize};

const ID_SOURCE: &str = "Pane";
const MARGIN: Vec2 = vec2(4.0, 2.0);

/// Pane
#[derive(Deserialize, Serialize)]
pub(crate) enum Pane {
    Temperature(temperature::Pane),
    Turbidity(turbidity::Pane),
}

impl Pane {
    pub(crate) fn temperature(frames: Vec<HashedMetaDataFrame>) -> Self {
        Self::Temperature(temperature::Pane::new(frames))
    }

    pub(crate) fn turbidity(frames: Vec<HashedMetaDataFrame>) -> Self {
        Self::Turbidity(turbidity::Pane::new(frames))
    }

    pub(crate) const fn kind(&self) -> Kind {
        match self {
            Self::Temperature(_) => Kind::Dtec,
            Self::Turbidity(_) => Kind::Atuc,
        }
    }

    pub(crate) fn title(&self) -> String {
        match self {
            Self::Temperature(pane) => pane.title(),
            Self::Turbidity(pane) => pane.title(),
        }
    }

    // pub(crate) const fn topic(&self) -> Option<&str> {
    //     if self.is_real_time() {
    //         Some(self.kind.topic())
    //     } else {
    //         None
    //     }
    // }

    // pub(crate) const fn is_real_time(&self) -> bool {
    //     // self.source.is_none()
    //     false
    // }
}

/// Behavior
#[derive(Debug)]
pub(crate) struct Behavior {
    pub(crate) close: Option<TileId>,
}

impl egui_tiles::Behavior<Pane> for Behavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> WidgetText {
        pane.title().to_string().into()
    }

    fn pane_ui(&mut self, ui: &mut Ui, tile_id: TileId, pane: &mut Pane) -> UiResponse {
        match pane {
            Pane::Temperature(pane) => pane.ui(ui, self, tile_id),
            Pane::Turbidity(pane) => pane.ui(ui, self, tile_id),
        }
    }
}

/// Kind
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) enum Kind {
    Atuc,
    Ddoc(Ddoc),
    Dtec,
}

/// Digital Disolved Oxygen Controller
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) enum Ddoc {
    C1,
    C2,
    T1,
    T2,
    V1,
    V2,
}

impl Kind {
    pub(crate) const fn topic(&self) -> &str {
        match self {
            Kind::Atuc => TOPIC_ATUC,
            Kind::Ddoc(Ddoc::C1) => TOPIC_DDOC_C1,
            Kind::Ddoc(Ddoc::C2) => TOPIC_DDOC_C2,
            Kind::Ddoc(Ddoc::T1) => TOPIC_DDOC_T1,
            Kind::Ddoc(Ddoc::T2) => TOPIC_DDOC_T2,
            Kind::Ddoc(Ddoc::V1) => TOPIC_DDOC_V1,
            Kind::Ddoc(Ddoc::V2) => TOPIC_DDOC_V2,
            Kind::Dtec => TOPIC_DTEC,
        }
    }
}

//     pub(crate) const ATUC: Self = Self::new(Kind::Atuc);
//     pub(crate) const DDOC_C1: Self = Self::new(Kind::Ddoc(Ddoc::C1));
//     pub(crate) const DDOC_C2: Self = Self::new(Kind::Ddoc(Ddoc::C2));
//     pub(crate) const DDOC_T1: Self = Self::new(Kind::Ddoc(Ddoc::T1));
//     pub(crate) const DDOC_T2: Self = Self::new(Kind::Ddoc(Ddoc::T2));
//     pub(crate) const DDOC_V1: Self = Self::new(Kind::Ddoc(Ddoc::V1));
//     pub(crate) const DDOC_V2: Self = Self::new(Kind::Ddoc(Ddoc::V2));
//     pub(crate) const DTEC: Self = Self::new(Kind::Dtec);

pub(crate) mod temperature;
pub(crate) mod turbidity;

// pub(crate) mod plot;
// pub(crate) mod table;
// pub(crate) mod view;
