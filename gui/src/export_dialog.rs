use egui::{Align2, Context, DragValue, Grid, Layout, Window};

/// Settings-Dialog for the export
pub struct ExportSettings {
    /// Resolution to render at
    resolution: (u32, u32),
    /// FPS to render at
    fps: u32,
    /// Whether the export settings dialog is shown
    show: bool,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            resolution: (1920, 1080),
            fps: 30,
            show: false,
        }
    }
}

impl ExportSettings {
    /// Shows these [ExportSettings] and resets to defaults
    pub fn show(&mut self) {
        *self = Default::default();
        self.show = true;
    }

    /// Draws these [ExportSettings] (if they are [shown][ExportSettings::show]).
    /// Returns `true` when the user accepted the settings.
    pub fn draw(&mut self, ctx: &Context) -> bool {
        let ok_clicked = Window::new("Export Video")
            .anchor(Align2::CENTER_CENTER, (0., 0.))
            .resizable(false)
            .movable(false)
            .collapsible(false)
            .open(&mut self.show)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    let w = Grid::new("export_dialog")
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("Resolution:");
                            ui.horizontal(|ui| {
                                ui.add(DragValue::new(&mut self.resolution.0));
                                ui.label("x");
                                ui.add(DragValue::new(&mut self.resolution.1));
                            });
                            ui.end_row();

                            ui.label("FPS:");
                            ui.add(DragValue::new(&mut self.fps));
                            ui.end_row();
                        })
                        .response
                        .rect
                        .width();

                    ui.set_max_width(w);
                    ui.with_layout(Layout::top_down_justified(egui::Align::Center), |ui| {
                        ui.button("Ok").clicked()
                    })
                    .inner
                })
                .inner
            })
            .and_then(|r| r.inner)
            .unwrap_or(false);
        if ok_clicked && self.show {
            self.show = false;
        }
        ok_clicked
    }

    /// Gets the currently selected resolution.
    /// Note: Changes with user-input when shown.
    pub fn resolution(&self) -> (u32, u32) {
        self.resolution
    }

    /// Gets the currently selected fps.
    /// Note: Changes with user-input when shown.
    pub fn fps(&self) -> u32 {
        self.fps
    }
}
