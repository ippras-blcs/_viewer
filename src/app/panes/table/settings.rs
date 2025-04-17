use crate::app::MAX_PRECISION;
use egui::{ComboBox, Grid, PopupCloseBehavior, RichText, Slider, Ui};
use egui_ext::LabeledSeparator as _;
use egui_l20n::{ResponseExt, UiExt as _};
use egui_phosphor::regular::{FUNNEL, FUNNEL_X};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fmt::{self, Display, Formatter},
    ops::Deref,
};

/// Table settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) resizable: bool,
    pub(crate) editable: bool,
    pub(crate) precision: usize,
    pub(crate) sticky_columns: usize,
    pub(crate) truncate: bool,

    pub(crate) filter: Filter,
    pub(crate) sort: Sort,
    pub(crate) order: Order,
}

impl Settings {
    pub(crate) const fn new() -> Self {
        Self {
            resizable: false,
            editable: false,
            precision: 2,
            sticky_columns: 0,
            truncate: false,

            filter: Filter::new(),
            sort: Sort::Timestamp,
            order: Order::Ascending,
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui, data_frame: &DataFrame) {
        Grid::new("TableSettings").show(ui, |ui| -> PolarsResult<()> {
            // Precision
            ui.label(ui.localize("precision"));
            ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
            ui.end_row();

            // // Sticky
            // ui.label(ui.localize("sticky_columns"));
            // ui.add(Slider::new(&mut self.sticky_columns));
            // ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            // Filter
            ui.separator();
            ui.labeled_separator(RichText::new(ui.localize("filter")).heading());
            ui.end_row();

            self.filter.show(ui, data_frame)?;
            ui.end_row();

            // Sort
            ui.separator();
            ui.labeled_separator(RichText::new(ui.localize("sort")).heading());
            ui.end_row();

            ui.label(ui.localize("sort"));
            ComboBox::from_id_salt("sort")
                .selected_text(ui.localize(self.sort.text()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.sort,
                        Sort::Identifier,
                        ui.localize(Sort::Identifier.text()),
                    )
                    .on_hover_localized(Sort::Identifier.hover_text());
                    ui.selectable_value(
                        &mut self.sort,
                        Sort::Timestamp,
                        ui.localize(Sort::Timestamp.text()),
                    )
                    .on_hover_localized(Sort::Timestamp.hover_text());
                    ui.selectable_value(
                        &mut self.sort,
                        Sort::Value,
                        ui.localize(Sort::Value.text()),
                    )
                    .on_hover_localized(Sort::Value.hover_text());
                })
                .response
                .on_hover_localized(self.sort.hover_text());
            ui.end_row();
            // Order
            ui.label(ui.localize("order"));
            ComboBox::from_id_salt("Order")
                .selected_text(ui.localize(self.order.text()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.order,
                        Order::Ascending,
                        ui.localize(Order::Ascending.text()),
                    )
                    .on_hover_localized(Order::Ascending.hover_text());
                    ui.selectable_value(
                        &mut self.order,
                        Order::Descending,
                        ui.localize(Order::Descending.text()),
                    )
                    .on_hover_localized(Order::Descending.hover_text());
                })
                .response
                .on_hover_localized(self.order.hover_text());
            ui.end_row();
            Ok(())
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

/// Filter
#[derive(Clone, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Filter {
    pub(crate) identifiers: BTreeSet<u64>,
}

impl Filter {
    pub(crate) const fn new() -> Self {
        Self {
            identifiers: BTreeSet::new(),
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui, data_frame: &DataFrame) -> PolarsResult<()> {
        ui.label(ui.localize("filter-by-identifier"))
            .on_hover_localized("filter-by-identifier.hover");
        let text = self.identifiers.len().to_string();
        let hover_text = format!("{self}");
        let inner_response = ComboBox::from_id_salt("Identifiers")
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .selected_text(text)
            .show_ui(ui, |ui| -> PolarsResult<()> {
                let identifiers = data_frame["Identifier"].u64()?.unique()?;
                for identifier in identifiers.iter().flatten() {
                    let checked = self.identifiers.contains(&identifier);
                    let response = ui.selectable_label(
                        checked,
                        RichText::new(format!("{identifier:x}")).monospace(),
                    );
                    if response.clicked() {
                        if checked {
                            self.identifiers.remove(&identifier);
                        } else {
                            self.identifiers.insert(identifier);
                        }
                    }
                    response.context_menu(|ui| {
                        if ui.button(format!("{FUNNEL} Select all")).clicked() {
                            self.identifiers = identifiers.iter().flatten().collect();
                            ui.close_menu();
                        }
                        if ui.button(format!("{FUNNEL_X} Unselect all")).clicked() {
                            self.identifiers = BTreeSet::new();
                            ui.close_menu();
                        }
                    });
                }
                Ok(())
            });
        inner_response.response.on_hover_text(hover_text);
        inner_response.inner.transpose()?;
        Ok(())
    }
}

impl Deref for Filter {
    type Target = BTreeSet<u64>;

    fn deref(&self) -> &Self::Target {
        &self.identifiers
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(first) = self.identifiers.first() {
            write!(f, "{first:x}")?;
            if let Some(last) = self.identifiers.last() {
                match self.identifiers.len() {
                    ..3 => f.write_str(",")?,
                    3.. => f.write_str("...")?,
                }
                write!(f, "{last:x}")?;
            }
        }
        Ok(())
    }
}

/// Sort
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Sort {
    Identifier,
    Timestamp,
    Value,
}

impl Sort {
    pub(crate) fn text(self) -> &'static str {
        match self {
            Self::Identifier => "sort_by_identifier",
            Self::Timestamp => "sort_by_timestamp",
            Self::Value => "sort_by_value",
        }
    }

    pub(crate) fn hover_text(self) -> &'static str {
        match self {
            Self::Identifier => "sort_by_identifier.hover",
            Self::Timestamp => "sort_by_timestamp.hover",
            Self::Value => "sort_by_value.hover",
        }
    }
}

/// Order
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Order {
    Ascending,
    Descending,
}

impl Order {
    pub(crate) fn text(self) -> &'static str {
        match self {
            Self::Ascending => "ascending_order",
            Self::Descending => "descending_order",
        }
    }

    pub(crate) fn hover_text(self) -> &'static str {
        match self {
            Self::Ascending => "ascending_order.hover",
            Self::Descending => "descending_order.hover",
        }
    }
}
