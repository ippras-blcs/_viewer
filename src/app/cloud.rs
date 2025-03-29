use anyhow::{Error, Result};
use egui::{style::Widgets, Label, ScrollArea, Sense, Spinner, TextStyle, Ui};
use egui_phosphor::regular::X;
use google_drive::{drive_hub, DriveHubExt, File};
use polars::frame::DataFrame;
use poll_promise::Promise;
use std::sync::mpsc::Sender;
use tracing::error;

/// Google drive
pub struct GoogleDrive {
    promise: Option<Promise<Result<Vec<File>>>>,
    data: Sender<DataFrame>,
    errors: Sender<Error>,
}

impl GoogleDrive {
    pub fn new(data: Sender<DataFrame>, errors: Sender<Error>) -> Self {
        Self {
            promise: None,
            data,
            errors,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.visuals_mut().widgets = if ui.style().visuals.dark_mode {
            Widgets::dark()
        } else {
            Widgets::light()
        };
        if let Some(promise) = self.promise.take() {
            match promise.ready() {
                Some(Ok(files)) => {
                    let height = ui.text_style_height(&TextStyle::Body);
                    ScrollArea::vertical().show_rows(ui, height, files.len(), |ui, row_range| {
                        for index in row_range {
                            let file = &files[index];
                            if let Some(name) = &file.name {
                                if let Some(id) = &file.id {
                                    ui.horizontal(|ui| {
                                        if ui.add(Label::new(name).sense(Sense::click())).clicked()
                                        {
                                            let id = id.to_owned();
                                            let errors = self.errors.clone();
                                            let data = self.data.clone();
                                            Promise::spawn_async(async move {
                                                if let Err(error) = async move {
                                                    let drive_hub = drive_hub().await?;
                                                    let data_frame =
                                                        drive_hub.download_file(&id).await?;
                                                    data.send(data_frame);
                                                    Ok::<_, Error>(())
                                                }
                                                .await
                                                {
                                                    errors.send(error.into());
                                                }
                                            });
                                        }
                                        if ui.button(X).clicked() {
                                            let id = id.to_owned();
                                            self.promise = Some(Promise::spawn_async(async move {
                                                let drive_hub = drive_hub().await?;
                                                drive_hub.files().delete(&id).doit().await?;
                                                let files = drive_hub.list_files().await?;
                                                error!("delete OK");
                                                Ok::<_, Error>(files)
                                            }));
                                        }
                                    });
                                }
                            }
                        }
                    });
                }
                Some(Err(error)) => {
                    ui.label(error.to_string());
                    error!(%error);
                }
                None => {
                    ui.add(Spinner::new().size(256.0));
                }
            }
            self.promise.get_or_insert(promise);
        } else {
            self.promise = Some(Promise::spawn_async(async {
                let drive_hub = drive_hub().await?;
                let mut files = drive_hub.list_files().await?;
                files.retain(|file| {
                    file.id
                        .as_ref()
                        .is_some_and(|id| id != "1X6HtoRvFesgM6HlUMP2oFGBi9sLb2EBg")
                });
                Ok(files)
            }));
        }
    }
}
