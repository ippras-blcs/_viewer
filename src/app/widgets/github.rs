use crate::{
    app::{ICON_SIZE, identifiers::DATA},
    utils::{HashedMetaDataFrame, spawn},
};
use anyhow::{Context as _, Error, Result};
use egui::{
    Context, Id, InnerResponse, IntoAtoms, PopupCloseBehavior, Response, RichText, ScrollArea, Ui,
    Widget,
    containers::menu::{MenuButton, MenuConfig},
    scroll_area::ScrollAreaOutput,
};
use egui_ext::{Doi as _, LabeledSeparator};
use egui_phosphor::regular::CLOUD_ARROW_DOWN;
use ehttp::{Request, fetch_async};
use std::borrow::Cow;
use tracing::{instrument, trace};
use url::Url;
use urlencoding::decode;

/// Github widget
pub struct Github;

impl Github {
    fn content(&mut self, ui: &mut Ui) {
        // IPPRAS
        ui.hyperlink_to(RichText::new("IPPRAS").heading(), "https://ippras.ru");
        ui.menu_button_with_scroll("Acer", |ui| {
            ui.heading("Acer Ginnala");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Acer/Acer ginnala[1].2025-07-08.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Acer/Acer ginnala[2].2025-07-08.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Acer/Acer ginnala[3].2025-07-08.utca.ron");
            ui.heading("Acer Pensylvanicum");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Acer/Acer pensylvanicum[1].2025-07-08.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Acer/Acer pensylvanicum[2].2025-07-08.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Acer/Acer pensylvanicum[3].2025-07-08.utca.ron");
            ui.heading("Acer Rubrum");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Acer/Acer rubrum[1].2025-07-09.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Acer/Acer rubrum[2].2025-07-09.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Acer/Acer rubrum[3].2025-07-09.utca.ron");
            ui.heading("Acer Spicatum");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Acer/Acer spicatum[1].2025-07-09.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Acer/Acer spicatum[2].2025-07-09.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Acer/Acer spicatum[3].2025-07-09.utca.ron");
            ui.heading("Acer Ukurunduense");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Acer/Acer ukurunduense[1].2025-07-08.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Acer/Acer ukurunduense[2].2025-07-08.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Acer/Acer ukurunduense[3].2025-07-08.utca.ron");
        });
        ui.menu_button_with_scroll("Catalpa", |ui| {
            ui.heading("Catalpa ovata");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Catalpa/Catalpa ovata{TL}.2025-11-26.utca.ron");
        });
        ui.menu_button_with_scroll("Helianthus annuus", |ui| {
            ui.heading("Helianthus annuus");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/Бузулук[1].2025-11-06.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/Бузулук[2].2025-11-06.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/Бузулук[3].2025-11-06.utca.ron");
            ui.separator();
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-2233[1].2025-10-29.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-2233[2].2025-10-29.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-2233[3].2025-10-29.utca.ron");
            ui.separator();
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-2699[1].2025-10-30.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-2699[2].2025-10-30.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-2699[3].2025-10-30.utca.ron");
            ui.separator();
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-2776[1].2025-11-01.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-2776[2].2025-11-01.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-2776[3].2025-11-01.utca.ron");
            ui.separator();
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-3110[1].2025-11-10.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-3110[2].2025-11-10.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-3110[3].2025-11-10.utca.ron");
            ui.separator();
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-3384[1].2025-10-31.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-3384[2].2025-10-31.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-3384[3].2025-10-31.utca.ron");
            ui.separator();
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-3599[1].2025-10-30.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-3599[2].2025-10-30.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-3599[3].2025-10-30.utca.ron");
            ui.separator();
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-3675[1].2025-10-31.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-3675[2].2025-10-31.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-3675[3].2025-10-31.utca.ron");
            ui.separator();
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-3714[1].2025-10-31.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-3714[2].2025-10-31.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/HelianthusAnnuus/К-3714[3].2025-10-31.utca.ron");
        });
        ui.menu_button_with_scroll("Microalgae", |ui| {
            ui.labeled_separator("2025");
            ui.heading("Chromochloris zofingiensis");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/C-108{-N}[1].2025-04-23.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/C-108{-N}[2].2025-04-23.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/C-108{-N}[3].2025-04-23.utca.ron");
            ui.heading("Neochlorella semenenkoi");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/C-1210{-N}[1].2025-04-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/C-1210{-N}[2].2025-04-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/C-1210{-N}[3].2025-04-24.utca.ron");
            ui.heading("Lobosphaera sp.");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/C-1540{-N}[1].2025-04-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/C-1540{-N}[2].2025-04-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/C-1540{-N}[3].2025-04-24.utca.ron");
            ui.heading("Coelastrella affinis");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/H-626{-N}.2025-04-24.utca.ron");
            ui.heading("Porphyridium purpureum");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/P-519{-N}[1].2025-04-23.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/P-519{-N}[2].2025-04-23.utca.ron");

            ui.labeled_separator("2023");
            ui.heading("Vischeria sp.");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/C-70{Control;SN-1,2(2,3)}.2023-10-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/C-70{Control;SN-2}.2023-10-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/C-70{H_2O_2;SN-1,2(2,3)}.2023-10-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/C-70{H_2O_2;SN-2}.2023-10-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/C-70{NaCl;SN-1,2(2,3)}.2023-10-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/C-70{NaCl;SN-2}.2023-10-24.utca.ron");
            ui.heading("Vischeria punctata");
            // _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/H-242{Control;SN-1,2(2,3)}.2023-10-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/H-242{Control;SN-2}.2023-10-24.utca.ron");
            ui.heading("Coelastrella affinis");
            // _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/H-626{Control;0day;SN-1,2(2,3)}.2023-10-22.utca.ron");
            // _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/H-626{Control;3day;SN-1,2(2,3)}.2023-10-22.utca.ron");
            // _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/H-626{Control;9day;SN-1,2(2,3)}.2023-10-22.utca.ron");
            // _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/H-626{-N;3day;SN-1,2(2,3)}.2023-10-22.utca.ron");
            // _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/H-626{-N;9day;SN-1,2(2,3)}.2023-10-22.utca.ron");
            // _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/H-626{Control;-Mg;3day;SN-1,2(2,3)}.2023-10-22.utca.ron");
            // _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/H-626{Control;-Mg;9day;SN-1,2(2,3)}.2023-10-22.utca.ron");

            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/H-626{-N;3day;SN-2}.2023-10-22.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/H-626{-N;9day;SN-2}.2023-10-22.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/H-626{Control;-Mg;3day;SN-2}.2023-10-22.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/H-626{Control;-Mg;9day;SN-2}.2023-10-22.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/H-626{Control;0day;SN-2}.2023-10-22.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/H-626{Control;3day;SN-2}.2023-10-22.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Microalgae/H-626{Control;9day;SN-2}.2023-10-22.utca.ron");
        });
        ui.menu_button_with_scroll("Lunaria rediviva", |ui| {
            ui.heading("Lunaria rediviva");
            // Petal
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/LunariaRediviva/Lunaria rediviva, petal[1].2024-05-16.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/LunariaRediviva/Lunaria rediviva, petal[2].2024-05-16.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/LunariaRediviva/Lunaria rediviva, petal[3].2024-05-16.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/LunariaRediviva/Lunaria rediviva, petal[4].2024-05-17.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/LunariaRediviva/Lunaria rediviva, petal[5].2024-05-17.utca.ron");
            // Seed
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/LunariaRediviva/Lunaria rediviva, seed, 0mm[1].2024-05-27.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/LunariaRediviva/Lunaria rediviva, seed, 0mm[2].2024-05-27.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/LunariaRediviva/Lunaria rediviva, seed, 0mm[3].2024-05-27.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/LunariaRediviva/Lunaria rediviva, seed, 1mm[1].2024-05-29.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/LunariaRediviva/Lunaria rediviva, seed, 1mm[2].2024-05-29.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/LunariaRediviva/Lunaria rediviva, seed, 1mm[3].2024-05-29.utca.ron");
        });
        ui.menu_button_with_scroll("Pinus cedrus", |ui| {
            ui.heading("Pinus cedrus");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/PinusCedrus/Pinus cedrus{SN-1,2(2,3)}.2023-05-19.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/PinusCedrus/Pinus cedrus{SN-2}.2023-05-19.utca.ron");
        });
        ui.menu_button_with_scroll("Polyscias", |ui| {
            ui.heading("Polyscias");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Polyscias/Polyscias{SN-1,2(2,3)}[1].2024-11-12.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Polyscias/Polyscias{SN-1,2(2,3)}[2].2024-11-12.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Polyscias/Polyscias{SN-1,2(2,3)}[3].2024-11-12.utca.ron");

            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Polyscias/Polyscias{SN-2}[1].2024-11-12.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Polyscias/Polyscias{SN-2}[2].2024-11-12.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Polyscias/Polyscias{SN-2}[3].2024-11-12.utca.ron");
        });
        ui.menu_button_with_scroll("Sidorov (2014)", |ui| {
            ui.doi("10.1007/s11746-014-2553-8");
            ui.heading("Subgenus Euonymus");
            ui.heading("Section Euonymus");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2014/Euonymus bungeanus.2014-06-19.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2014/Euonymus europaeus.2014-06-19.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2014/Euonymus hamiltonianus.2014-06-19.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2014/Euonymus phellomanus.2014-06-19.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2014/Euonymus semiexsertus.2014-06-19.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2014/Euonymus sieboldianus.2014-06-19.utca.ron");
            ui.heading("Section Melanocarya");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2014/Euonymus alatus.2014-06-19.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2014/Euonymus sacrosanctus.2014-06-19.utca.ron");
            ui.heading("Section Pseudovyenomus");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2014/Euonymus pauciflorus.2014-06-19.utca.ron");
            ui.heading("Subgenus Kalonymus");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2014/Euonymus latifolius.2014-06-19.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2014/Euonymus macropterus.2014-06-19.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2014/Euonymus maximowiczianus.2014-06-19.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2014/Euonymus sachalinensis.2014-06-19.utca.ron");
        });
        ui.menu_button_with_scroll("Sidorov (2025)", |ui| {
            ui.doi("10.3390/plants14040612");
            ui.heading("Lunaria Rediviva");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2025/Lunaria rediviva{TMSH;SN-1,2(2,3)}[1.1].2024-01-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2025/Lunaria rediviva{TMSH;SN-1,2(2,3)}[1.2].2024-01-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2025/Lunaria rediviva{TMSH;SN-1,2(2,3)}[1.3].2024-01-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2025/Lunaria rediviva{TMSH;SN-1,2(2,3)}[2.1].2024-01-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2025/Lunaria rediviva{TMSH;SN-1,2(2,3)}[2.2].2024-01-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2025/Lunaria rediviva{TMSH;SN-1,2(2,3)}[3.1].2024-01-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2025/Lunaria rediviva{TMSH;SN-1,2(2,3)}[3.2].2024-01-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2025/Lunaria rediviva{TMSH;SN-1,2(2,3)}[3.3].2024-01-24.utca.ron");

            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2025/Lunaria rediviva{TMSH;SN-2}[1.1].2024-01-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2025/Lunaria rediviva{TMSH;SN-2}[1.2].2024-01-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2025/Lunaria rediviva{TMSH;SN-2}[1.3].2024-01-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2025/Lunaria rediviva{TMSH;SN-2}[2.1].2024-01-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2025/Lunaria rediviva{TMSH;SN-2}[2.2].2024-01-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2025/Lunaria rediviva{TMSH;SN-2}[3.1].2024-01-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2025/Lunaria rediviva{TMSH;SN-2}[3.2].2024-01-24.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/Sidorov2025/Lunaria rediviva{TMSH;SN-2}[3.3].2024-01-24.utca.ron");
        });
        ui.separator();
        // Third party
        ui.heading("Third party");
        ui.menu_button_with_scroll("Reske (1997)", |ui| {
            ui.doi("10.1007/s11746-997-0016-1");
            ui.heading("Soybean");
            // preset(ui, &SOYBEAN_SEED_COMMODITY);
            ui.heading("Sunflower");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/ThirdParty/Reske1997/Sunflower seed (Commodity).1997-08-01.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/ThirdParty/Reske1997/Sunflower seed (High linoleic).1997-08-01.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/ThirdParty/Reske1997/Sunflower seed (High oleic).1997-08-01.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/ThirdParty/Reske1997/Sunflower seed (High palmitic, high linoleic).1997-08-01.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/ThirdParty/Reske1997/Sunflower seed (High palmitic, high oleic).1997-08-01.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/ThirdParty/Reske1997/Sunflower seed (High stearic, high oleic).1997-08-01.utca.ron");
        });
        ui.menu_button_with_scroll("Martinez-Force (2004)", |ui| {
            ui.doi("10.1016/j.ab.2004.07.019");
            ui.heading("Hazelnut");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/ThirdParty/Martinez-Force2004/Hazelnut.2004-05-20.utca.ron");
            ui.heading("Olive");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/ThirdParty/Martinez-Force2004/Olive.2004-05-20.utca.ron");
            ui.heading("Rice");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/ThirdParty/Martinez-Force2004/Rice.2004-05-20.utca.ron");
            ui.heading("Soybean");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/ThirdParty/Martinez-Force2004/Soybean.2004-05-20.utca.ron");
            ui.heading("Sunflower");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/ThirdParty/Martinez-Force2004/Sunflower CAS-3.2004-05-20.utca.ron");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/ThirdParty/Martinez-Force2004/Sunflower RHA-274.2004-05-20.utca.ron");
            ui.heading("Walnut");
            _ = preset(ui, "https://raw.githubusercontent.com/ippras/utca/presets/ThirdParty/Martinez-Force2004/Walnut.2004-05-20.utca.ron");
        });
    }
}

impl Widget for Github {
    fn ui(mut self, ui: &mut Ui) -> Response {
        MenuButton::new(RichText::new(CLOUD_ARROW_DOWN).size(ICON_SIZE))
            .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
            .ui(ui, |ui| {
                ScrollArea::new([false, true]).show(ui, |ui| self.content(ui));
            })
            .0
    }
}

/// Extension methods for [`Ui`]
trait UiExt: Sized {
    fn menu_button_with_scroll<'a, R>(
        &mut self,
        atoms: impl IntoAtoms<'a>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<Option<ScrollAreaOutput<R>>>;
}

impl UiExt for Ui {
    fn menu_button_with_scroll<'a, R>(
        &mut self,
        atoms: impl IntoAtoms<'a>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<Option<ScrollAreaOutput<R>>> {
        self.menu_button(atoms, |ui| ScrollArea::vertical().show(ui, add_contents))
    }
}

/// Preset
#[instrument(skip(ui), err)]
fn preset(ui: &mut Ui, input: &str) -> Result<()> {
    let url = Url::parse(input)?;
    let (name, date) = parse(&url)?;
    if ui
        .button(format!("{CLOUD_ARROW_DOWN} {name} {date}"))
        .clicked()
    {
        load(ui.ctx(), url);
    }
    Ok(())
}

/// Parse preset url
fn parse<'a>(url: &'a Url) -> Result<(Cow<'a, str>, &'a str)> {
    let segment = url
        .path_segments()
        .context("Preset get path segments")?
        .last()
        .context("Preset get last path segment")?;
    let input = segment.trim_end_matches(".utca.ron");
    let (name, date) = input
        .rsplit_once(".")
        .context("Preset parse name and date")?;
    Ok((decode(name)?, date))
}

fn load(ctx: &Context, url: Url) {
    let ctx = ctx.clone();
    _ = spawn(async move {
        if let Ok(frame) = try_load(&url).await {
            trace!(?frame);
            ctx.data_mut(|data| data.insert_temp(Id::new(DATA), frame));
        }
    });
}

#[instrument(err)]
async fn try_load(url: &Url) -> Result<HashedMetaDataFrame> {
    let request = Request::get(url);
    let response = fetch_async(request).await.map_err(Error::msg)?;
    let text = response.text().context("Try load get response text")?;
    trace!(?text);
    Ok(ron::de::from_str(text)?)
}
