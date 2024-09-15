use wgpu::{Device, RenderPass, TextureFormat};

/// The main renderer, which renders the visualization output
pub struct Renderer {}

impl Renderer {
    /// Creates a new [Renderer] on the passed [Device] and for the passed [TextureFormat]
    pub fn new(_device: &Device, _format: TextureFormat) -> Self {
        Self {}
    }

    /// Draws the contents of this [Renderer] to the passed [RenderPass]
    pub fn draw<'a>(&'a self, _render_pass: &mut RenderPass<'a>) {}
}
