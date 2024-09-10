use crate::canvas::GlCanvas;

/// The main App to draw using [egui]/[eframe]
pub struct App {}

impl App {
    /// Create a new instance of the [App]
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {}
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            GlCanvas::new(|gl: &eframe::glow::Context| unsafe {
                use eframe::glow::HasContext;
                gl.clear_color(0.8, 0.3, 0., 1.);
                gl.clear(eframe::glow::COLOR_BUFFER_BIT);
            })
            .draw(ctx, ui);
        });
    }
}
