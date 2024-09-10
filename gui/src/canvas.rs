use std::sync::Arc;

use eframe::{
    egui_glow::CallbackFn,
    glow::{self, HasContext, SCISSOR_TEST, VIEWPORT},
};
use egui::{Context, PaintCallback, Ui};

/// A canvas that allows drawing using OpenGL.
/// The content to draw must implement [GlDrawable] and be set in [GlCanvas::new].
pub struct GlCanvas<C: GlDrawable + 'static> {
    content: C,
}

impl<C: GlDrawable + 'static> GlCanvas<C> {
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
            let content = self.content;

            ui.painter().add(PaintCallback {
                rect,
                callback: Arc::new(CallbackFn::new(move |_info, painter| {
                    let gl = painter.gl().as_ref();
                    unsafe {
                        let mut vp = [0, 0, 0, 0];
                        gl.get_parameter_i32_slice(VIEWPORT, &mut vp);
                        gl.enable(SCISSOR_TEST);
                        gl.scissor(vp[0], vp[1], vp[2], vp[3]);
                    }

                    content.draw(gl);

                    unsafe {
                        gl.disable(SCISSOR_TEST);
                    }
                })),
            });

            ctx.request_repaint();
        });
    }
}

/// A [GlDrawable] is something that can be drawn using OpenGL.
/// It has a [draw][GlDrawable::draw]-function to do that.
pub trait GlDrawable: Send + Sync + Copy {
    fn draw(&self, gl: &glow::Context);
}

/// A closure can be a [GlDrawable].
impl<F: Fn(&glow::Context) + Send + Sync + Copy> GlDrawable for F {
    fn draw(&self, gl: &glow::Context) {
        self(gl)
    }
}
