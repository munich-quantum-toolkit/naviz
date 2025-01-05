//! [MenuBar] to show a menu on the top.

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

use egui::{Align2, Grid, Window};
use export::ExportMenu;
use git_version::git_version;
#[cfg(not(target_arch = "wasm32"))]
use naviz_video::VideoProgress;

use crate::{future_helper::FutureHelper, util::WEB};

type SendReceivePair<T> = (Sender<T>, Receiver<T>);

/// The menu bar struct which contains the state of the menu
pub struct MenuBar {
    event_channel: SendReceivePair<MenuEvent>,
    /// Whether to draw the about-window
    about_open: bool,
    /// Export interaction handling (menu, config, progress)
    export_menu: ExportMenu,
    /// List of machines: `(id, name)`
    machines: Vec<(String, String)>,
    /// List of styles: `(id, name)`
    styles: Vec<(String, String)>,
}

/// An event which is triggered on menu navigation.
/// Higher-Level than just button-clicks.
pub enum MenuEvent {
    /// A file of the specified [FileType] with the specified content was opened
    FileOpen(FileType, Vec<u8>),
    /// A video should be exported to the specified path with the specified resolution and fps
    #[cfg(not(target_arch = "wasm32"))]
    ExportVideo {
        target: PathBuf,
        resolution: (u32, u32),
        fps: u32,
        /// Channel for progress updates
        progress: Sender<VideoProgress>,
    },
    /// A new machine with the specified `id` was selected
    SetMachine(String),
    /// A new style with the specified `id` was selected
    SetStyle(String),
}

/// The available FileTypes for opening
pub enum FileType {
    Instructions,
    Machine,
    Style,
}

/// Config options for what to show inside the menu
pub struct MenuConfig {
    /// Show export option
    pub export: bool,
}

impl FileType {
    pub fn name(&self) -> &'static str {
        match self {
            FileType::Instructions => "NAViz instructions",
            FileType::Machine => "NAViz machine",
            FileType::Style => "NAViz style",
        }
    }
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            FileType::Instructions => &["naviz"],
            FileType::Machine => &["namachine"],
            FileType::Style => &["nastyle"],
        }
    }
}

impl MenuBar {
    /// Create a new [MenuBar]
    pub fn new() -> Self {
        Self {
            event_channel: channel(),
            about_open: false,
            export_menu: ExportMenu::new(),
            machines: vec![],
            styles: vec![],
        }
    }

    /// Update the machine-list.
    /// Machines are `(id, name)`.
    pub fn update_machines(&mut self, machines: Vec<(String, String)>) {
        self.machines = machines;
        self.machines.sort_by(|(_, a), (_, b)| a.cmp(b));
    }

    /// Move the compatible machines to the top of the list
    pub fn set_compatible_machines(&mut self, machines: &[String]) {
        // Sort by containment in machines, then by name
        self.machines.sort_by(|(id_a, name_a), (id_b, name_b)| {
            machines
                .contains(id_a)
                .cmp(&machines.contains(id_b))
                .reverse()
                .then_with(|| name_a.cmp(name_b))
        });
    }

    /// Update the style-list.
    /// Styles are `(id, name)`.
    pub fn update_styles(&mut self, styles: Vec<(String, String)>) {
        self.styles = styles;
        self.styles.sort_by(|(_, a), (_, b)| a.cmp(b));
    }

    /// Get the file open channel.
    ///
    /// Whenever a new file is opened,
    /// its content will be sent over this channel.
    pub fn events(&self) -> &Receiver<MenuEvent> {
        &self.event_channel.1
    }

    /// Draw the [MenuBar]
    pub fn draw(
        &mut self,
        config: MenuConfig,
        future_helper: &FutureHelper,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        self.export_menu.process_events(&mut self.event_channel.0);

        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open Instructions").clicked() {
                    self.choose_file(FileType::Instructions, future_helper);
                }
                if ui.button("Open Machine").clicked() {
                    self.choose_file(FileType::Machine, future_helper);
                }
                if ui.button("Open Style").clicked() {
                    self.choose_file(FileType::Style, future_helper);
                }

                self.export_menu.draw_button(config.export, ui);

                if !WEB {
                    // Quit-button only on native
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            });

            // Machine selection
            ui.menu_button("Machine", |ui| {
                for (id, name) in &self.machines {
                    if ui.button(name).clicked() {
                        let _ = self.event_channel.0.send(MenuEvent::SetMachine(id.clone()));
                        ui.close_menu();
                    }
                }
            });

            // Style selection
            ui.menu_button("Style", |ui| {
                for (id, name) in &self.styles {
                    if ui.button(name).clicked() {
                        let _ = self.event_channel.0.send(MenuEvent::SetStyle(id.clone()));
                        ui.close_menu();
                    }
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    self.about_open = true;
                }
            });
        });

        self.export_menu.draw_windows(future_helper, ctx);

        self.draw_about_window(ctx);
    }

    /// Show the file-choosing dialog and read the file if a new file was selected
    fn choose_file(&self, file_type: FileType, future_helper: &FutureHelper) {
        future_helper.execute_maybe_to(
            async move {
                if let Some(path) = rfd::AsyncFileDialog::new()
                    .add_filter(file_type.name(), file_type.extensions())
                    .pick_file()
                    .await
                {
                    Some(MenuEvent::FileOpen(file_type, path.read().await))
                } else {
                    None
                }
            },
            self.event_channel.0.clone(),
        );
    }

    /// Draws the about-window if [Self::about_open] is `true`
    fn draw_about_window(&mut self, ctx: &egui::Context) {
        Window::new("About NAViz")
            .anchor(Align2::CENTER_CENTER, (0., 0.))
            .resizable(false)
            .open(&mut self.about_open)
            .collapsible(false)
            .show(ctx, |ui| {
                Grid::new("about_window").num_columns(2).show(ui, |ui| {
                    ui.label("Version");
                    ui.label(env!("CARGO_PKG_VERSION"));
                    ui.end_row();

                    ui.label("Build");
                    ui.label(git_version!(
                        args = ["--always", "--dirty=+dev", "--match=naviz-gui@*"],
                        fallback = "unknown"
                    ));
                    ui.end_row();

                    ui.label("License");
                    ui.label(env!("CARGO_PKG_LICENSE"));
                    ui.end_row();

                    ui.label("Source Code");
                    ui.hyperlink(env!("CARGO_PKG_REPOSITORY"));
                    ui.end_row();
                });
            });
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod export {
    //! Export-Menu on native

    use std::{
        path::PathBuf,
        sync::mpsc::{channel, Sender},
    };

    use egui::{Button, Context};

    use crate::{
        export_dialog::{ExportProgresses, ExportSettings},
        future_helper::FutureHelper,
    };

    use super::{MenuEvent, SendReceivePair};

    /// Menu components concerning export
    pub struct ExportMenu {
        /// Channel for selected export-settings
        export_channel: SendReceivePair<(PathBuf, (u32, u32), u32)>,
        /// The export-settings-dialog to show when the user wants to export a video
        export_settings: ExportSettings,
        /// The export-progress-dialogs to show
        export_progresses: ExportProgresses,
    }

    impl ExportMenu {
        /// Creates a new [ExportMenu]
        pub fn new() -> Self {
            Self {
                export_channel: channel(),
                export_settings: Default::default(),
                export_progresses: Default::default(),
            }
        }

        /// Processes events concerning export
        pub fn process_events(&mut self, menu_event_sender: &mut Sender<MenuEvent>) {
            if let Ok((target, resolution, fps)) = self.export_channel.1.try_recv() {
                let _ = menu_event_sender.send(MenuEvent::ExportVideo {
                    target,
                    resolution,
                    fps,
                    progress: self.export_progresses.add(),
                });
            }
        }

        /// Draws the menu button concerning export
        pub fn draw_button(&mut self, enabled: bool, ui: &mut egui::Ui) {
            ui.separator();
            if ui
                .add_enabled(enabled, Button::new("Export Video"))
                .clicked()
            {
                self.export_settings.show();
            }
        }

        /// Draws the windows concerning video export
        pub fn draw_windows(&mut self, future_helper: &FutureHelper, ctx: &Context) {
            if self.export_settings.draw(ctx) {
                self.export(future_helper);
            }

            self.export_progresses.draw(ctx);
        }

        /// Show the file-saving dialog and get the path to export to if a file was selected
        fn export(&self, future_helper: &FutureHelper) {
            let resolution = self.export_settings.resolution();
            let fps = self.export_settings.fps();
            future_helper.execute_maybe_to(
                async move {
                    rfd::AsyncFileDialog::new()
                        .save_file()
                        .await
                        .map(|handle| handle.path().to_path_buf())
                        .map(|target| (target, resolution, fps))
                },
                self.export_channel.0.clone(),
            );
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub mod export {
    //! Export-Menu-Stub for web (does not exist on web platforms)
    //!
    //! Signatures should match the export-module on native.
    //! See that module for documentation.

    use std::sync::mpsc::Sender;

    use egui::Context;

    use crate::future_helper::FutureHelper;

    use super::MenuEvent;

    pub struct ExportMenu {}

    impl ExportMenu {
        pub fn new() -> Self {
            Self {}
        }

        pub fn process_events(&mut self, _menu_event_sender: &mut Sender<MenuEvent>) {}

        pub fn draw_button(&mut self, _enabled: bool, _ui: &mut egui::Ui) {}

        pub fn draw_windows(&mut self, _future_helper: &FutureHelper, _ctx: &Context) {}
    }
}
