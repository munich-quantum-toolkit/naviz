use eframe::egui_wgpu::{Callback, CallbackTrait};
use egui::{Color32, Context, Ui, Vec2};

/// A canvas that allows drawing using OpenGL.
/// The content to draw must implement [CanvasContent] and be set in [WgpuCanvas::new].
pub struct WgpuCanvas<C: CanvasContent + 'static> {
    content: C,
    aspect: f32,
}

impl<C: CanvasContent + 'static> WgpuCanvas<C> {
    /// Create a new [WgpuCanvas] that renders the specified content.
    pub fn new(content: C, aspect: f32) -> Self {
        Self { content, aspect }
    }

    /// Draws this canvas.
    /// Takes remaining space of parent.
    /// Also requests a repaint immediately.
    pub fn draw(&self, ctx: &Context, ui: &mut Ui) {
        egui::Frame::canvas(ui.style())
            .fill(self.content.background_color())
            .show(ui, |ui| {
                let available = ui.available_size();
                let desired = constrain_to_aspect(available, self.aspect);
                let (_, rect) = ui.allocate_space(desired);
                ui.painter()
                    .add(Callback::new_paint_callback(rect, self.content.clone()));

                ctx.request_repaint();
            });
    }
}

/// constrains the passed size (in the [Vec2]) to be the passed `aspect`.
/// Will shrink one of the dimensions if needed.
fn constrain_to_aspect(Vec2 { x: mut w, y: mut h }: Vec2, aspect: f32) -> Vec2 {
    match (w.is_finite(), h.is_finite()) {
        (true, true) => {
            if w / aspect < h {
                h = w / aspect;
            }
            if h * aspect < w {
                w = h * aspect;
            }
        }
        (false, true) => w = h * aspect,
        (true, false) => h = w / aspect,
        (false, false) => { /* Infinite in both directions => always correct aspect ratio */ }
    }
    Vec2 { x: w, y: h }
}

pub trait CanvasContent: CallbackTrait + Clone {
    fn background_color(&self) -> Color32;
}

/// An empty canvas.
///
/// Draws nothing
#[derive(Clone, Copy)]
pub struct EmptyCanvas {}

#[allow(dead_code)]
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
