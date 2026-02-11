use egui::{Label, Response, RichText, Sense, TextStyle, Ui, Widget};
use egui_phosphor::regular::{COPYRIGHT, GITHUB_LOGO, GLOBE, WARNING};

/// About widget
#[derive(Debug, Default)]
pub(crate) struct About;

impl Widget for About {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical_centered(|ui| {
            let version = env!("CARGO_PKG_VERSION");
            ui.heading(format!("UTCA {version}"));
            ui.label("Ultimate TAG Calculation Application");
            // Links
            ui.separator();
            ui.collapsing(RichText::new("Links").heading(), |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(GLOBE).heading())
                        .on_hover_text("web");
                    ui.hyperlink_to(
                        "https://ippras.github.io/utca",
                        "https://ippras.github.io/utca",
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new(GITHUB_LOGO).heading())
                        .on_hover_text("github.com");
                    ui.hyperlink_to(
                        "https://github.com/ippras/utca",
                        "https://github.com/ippras/utca",
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new(WARNING).heading())
                        .on_hover_text("report an issue");
                    ui.hyperlink_to(
                        "https://github.com/ippras/utca/issues",
                        "https://github.com/ippras/utca/issues",
                    );
                });
            });
            // Dedications
            ui.collapsing(RichText::new("Dedications").heading(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Giorgi Kazakov:");
                    ui.label("Моим родителям, Тане и Володе, посвящается.");
                });
            });
            // Copyright
            ui.separator();
            ui.horizontal(|ui| {
                let width = ui.fonts_mut(|fonts| {
                    fonts.glyph_width(&TextStyle::Body.resolve(ui.style()), ' ')
                });
                ui.spacing_mut().item_spacing.x = width;
                ui.label(COPYRIGHT);
                ui.label("2024");
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.add(Label::new("Giorgi Kazakov").sense(Sense::click()));
                ui.spacing_mut().item_spacing.x = width;
                ui.label(",");
                ui.add(Label::new("Roman Sidorov").sense(Sense::click()));
            });
        })
        .response
    }
}
