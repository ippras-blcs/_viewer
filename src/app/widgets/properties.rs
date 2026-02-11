use crate::{r#const::relative_atomic_mass::CH2, localize};
use egui::{Grid, Response, Ui, Widget};
use lipid::fatty_acid::{FattyAcid, Kind as FattyAcidKind, mass::Mass as _};
use polars::prelude::AnyValue;

/// Properties
pub(crate) struct Properties<'a> {
    pub(crate) fatty_acid: &'a FattyAcid,
}

impl<'a> Properties<'a> {
    pub(crate) fn new(fatty_acid: &'a mut FattyAcid) -> Self {
        Self { fatty_acid }
    }
}

impl Widget for Properties<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = ui.heading(localize!("properties"));
        Grid::new(ui.next_auto_id()).show(ui, |ui| {
            let mass = self.fatty_acid.mass(FattyAcidKind::Rcooh);
            ui.label(localize!("fatty_acid_mass"));
            ui.label(AnyValue::from(mass).to_string());
            ui.end_row();
            ui.label(localize!("methyl_ester_mass"));
            let value = AnyValue::from(mass + CH2);
            ui.label(value.to_string());
        });
        response
    }
}
