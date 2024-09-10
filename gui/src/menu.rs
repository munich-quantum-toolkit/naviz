//! [MenuBar] to show a menu on the top.

use std::sync::mpsc::{channel, Receiver, Sender};

use crate::{future_helper::FutureHelper, util::WEB};

/// The menu bar struct which contains the state of the menu
pub struct MenuBar {
    file_open_channel: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
}

impl MenuBar {
    /// Create a new [MenuBar]
    pub fn new() -> Self {
        Self {
            file_open_channel: channel(),
        }
    }

    /// Get the file open channel.
    ///
    /// Whenever a new file is opened,
    /// its content will be sent over this channel.
    pub fn file_open_channel(&self) -> &Receiver<Vec<u8>> {
        &self.file_open_channel.1
    }

    /// Draw the [MenuBar]
    pub fn draw(&mut self, future_helper: &FutureHelper, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    self.choose_file(future_helper);
                }

                if !WEB {
                    // Quit-button only on native
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            });
        });
    }

    /// Show the file-choosing dialog and read the file if a new file was selected
    fn choose_file(&self, future_helper: &FutureHelper) {
        future_helper.execute_maybe_to(
            async move {
                if let Some(path) = rfd::AsyncFileDialog::new()
                    .add_filter("NAViz Input File", &["naviz"])
                    .pick_file()
                    .await
                {
                    Some(path.read().await)
                } else {
                    None
                }
            },
            self.file_open_channel.0.clone(),
        );
    }
}
