use crate::{canvas::WgpuCanvas, future_helper::FutureHelper, menu::MenuBar};

/// The main App to draw using [egui]/[eframe]
pub struct App {
    future_helper: FutureHelper,
    menu_bar: MenuBar,
    /// The contents of the currently opened file
    file_contents: Vec<u8>,
}

impl App {
    /// Create a new instance of the [App]
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            future_helper: FutureHelper::new().expect("Failed to create FutureHelper"),
            menu_bar: MenuBar::new(),
            file_contents: Vec::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if a new file was read
        if let Ok(c) = self.menu_bar.file_open_channel().try_recv() {
            self.file_contents = c;
        }

        // Menu
        egui::TopBottomPanel::top("app_menu")
            .show(ctx, |ui| self.menu_bar.draw(&self.future_helper, ctx, ui));

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!(
                "File: {:?}{}",
                &self.file_contents[..32.min(self.file_contents.len())],
                if self.file_contents.len() > 32 {
                    " (truncated)"
                } else {
                    ""
                }
            ));

            WgpuCanvas::new(crate::canvas::EmptyCanvas::new()).draw(ctx, ui);
        });
    }
}
