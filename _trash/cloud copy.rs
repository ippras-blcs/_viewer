use anyhow::{Error, Result};
use egui::{Id, ScrollArea, Spinner, TextStyle, Ui};
use futures::FutureExt;
use google_drive::{drive_hub, DriveHubExt, File};
use polars::frame::DataFrame;
use poll_promise::Promise;
// use std::sync::mpsc::Sender;
use tokio::{
    runtime::Runtime,
    spawn,
    sync::mpsc::Sender,
    task::{spawn_local, JoinHandle},
};
use tracing::{error, trace};

pub struct GoogleDrive {
    join_handle: Option<JoinHandle<Result<Vec<File>>>>,
    sender: Sender<Error>,
}

impl GoogleDrive {
    pub fn new(sender: Sender<Error>) -> Self {
        Self {
            join_handle: None,
            sender,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        let join_handle = self.join_handle.get_or_insert_with(|| {
            spawn(async {
                let drive_hub = drive_hub().await?;
                let files = drive_hub.list_files().await?;
                println!("files: {files:?}");
                Ok(files)
            })
        });
        println!("promise1");
        if let Err(error) = || -> Result<()> {
            if let Some(files) = join_handle.now_or_never() {
                let files = files??;
                let height = ui.text_style_height(&TextStyle::Body);
                ScrollArea::vertical().show_rows(ui, height, files.len(), |ui, row_range| {
                    for index in row_range {
                        let file = &files[index];
                        if let Some(name) = &file.name {
                            if let Some(id) = &file.id {
                                if ui.button(name).clicked() {
                                    error!("clicked");
                                    // let id = id.to_owned();
                                    // let sender = self.sender.clone();
                                    // tokio::spawn(async move {
                                    //     let drive_hub = drive_hub().await?;
                                    //     let data_frame = drive_hub.download_file(&id).await?;
                                    //     sender.clone().send(data_frame).await?;
                                    //     Ok::<_, Error>(())
                                    // });
                                }
                            }
                        }
                    }
                });
            } else {
                ui.add(Spinner::new().size(256.0));
            }
            Ok(())
        }() {
            ui.label(error.to_string());
            error!(%error);
        };
        // if let Some(result) = promise.ready() {
        //     println!("promise ready");
        //     match result {
        //         Ok(files) => {
        //             let height = ui.text_style_height(&TextStyle::Body);
        //             ScrollArea::vertical().show_rows(ui, height, files.len(), |ui, row_range| {
        //                 for index in row_range {
        //                     let file = &files[index];
        //                     if let Some(name) = &file.name {
        //                         if let Some(id) = &file.id {
        //                             if ui.button(name).clicked() {
        //                                 error!("clicked");
        //                                 // let id = id.to_owned();
        //                                 // let sender = self.sender.clone();
        //                                 // tokio::spawn(async move {
        //                                 //     let drive_hub = drive_hub().await?;
        //                                 //     let data_frame = drive_hub.download_file(&id).await?;
        //                                 //     sender.clone().send(data_frame).await?;
        //                                 //     Ok::<_, Error>(())
        //                                 // });
        //                             }
        //                         }
        //                     }
        //                 }
        //             });
        //         }
        //         Err(error) => {
        //             ui.label(error.to_string());
        //         }
        //     };
        // } else {
        //     ui.add(Spinner::new().size(256.0));
        // }
    }
}
