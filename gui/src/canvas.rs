use eframe::egui_wgpu::{Callback, CallbackTrait};
use egui::{Context, Ui};

/// A canvas that allows drawing using OpenGL.
/// The content to draw must implement [GlDrawable] and be set in [GlCanvas::new].
pub struct WgpuCanvas<C: CallbackTrait + 'static> {
    content: C,
}

impl<C: CallbackTrait + Clone + 'static> WgpuCanvas<C> {
    /// Create a new [GlCanvas] that renders the specified content.
    pub fn new(content: C) -> Self {
        Self { content }
    }

    /// Draws this canvas.
    /// Takes remaining space of parent.
    /// Also requests a repaint immediately.
    pub fn draw(&self, ctx: &Context, ui: &mut Ui) {
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            let (_, rect) = ui.allocate_space(ui.available_size());
            ui.painter()
                .add(Callback::new_paint_callback(rect, self.content.clone()));

            ctx.request_repaint();
        });
    }
}

/// An empty canvas.
///
/// Draws nothing
#[derive(Clone, Copy)]
pub struct EmptyCanvas {}

impl EmptyCanvas {
    pub fn new() -> Self {
        Self {}
    }
}

impl CallbackTrait for EmptyCanvas {
    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        _render_pass: &mut eframe::wgpu::RenderPass<'a>,
        _callback_resources: &'a eframe::egui_wgpu::CallbackResources,
    ) {
    }
}
