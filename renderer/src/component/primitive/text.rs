use glam::{Mat4, Vec3};
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer,
};
use wgpu::{Device, MultisampleState, Queue, RenderPass, TextureFormat};

use crate::viewport::ViewportProjection;

#[derive(Clone, Copy, Default)]
pub enum HAlignment {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Clone, Copy, Default)]
pub enum VAlignment {
    Top,
    #[default]
    Center,
    Bottom,
}

#[derive(Clone, Copy, Default)]
pub struct Alignment(pub HAlignment, pub VAlignment);

pub struct TextSpec<'a> {
    /// The viewport projection to render in.
    /// Does not use viewport (or globals) directly,
    /// but renders using [glyphon].
    pub viewport_projection: ViewportProjection,
    /// The size of the font
    pub font_size: f32,
    /// The font to use
    pub font_family: &'a str,
    /// The texts to render: (`text`, `position`, `alignment`)
    pub texts: &'a [(&'a str, (f32, f32), Alignment)],
    /// The color of the texts to render
    pub color: [u8; 4],
}

/// A component that renders text
pub struct Text {
    atlas: TextAtlas,
    glyphon_viewport: glyphon::Viewport,
    text_renderer: TextRenderer,
}

impl Text {
    pub fn new(
        device: &Device,
        queue: &Queue,
        format: TextureFormat,
        TextSpec {
            viewport_projection,
            font_size,
            font_family,
            texts,
            color,
        }: TextSpec,
    ) -> Self {
        let color = Color::rgba(color[0], color[1], color[2], color[3]);

        let mut font_system = FontSystem::new();
        // Load a default font
        // Used when system-fonts cannot be loaded (e.g., on web)
        font_system
            .db_mut()
            .load_font_data(include_bytes!(env!("DEFAULT_FONT_PATH")).to_vec());

        let mut swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let mut glyphon_viewport = glyphon::Viewport::new(device, &cache);
        glyphon_viewport.update(
            queue,
            Resolution {
                width: viewport_projection.source.width as u32,
                height: viewport_projection.source.height as u32,
            },
        );
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let mut text_renderer =
            TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);

        // Convert the passed texts to text buffers
        let text_buffers: Vec<_> = texts
            .iter()
            .map(|(text, pos, alignment)| {
                (
                    to_text_buffer(text, &mut font_system, font_size, font_family),
                    pos,
                    alignment,
                )
            })
            .collect();
        // and then to text areas
        let text_areas = text_buffers.iter().map(|(buf, pos, alignment)| {
            to_text_area(buf, **pos, **alignment, color, viewport_projection)
        });

        // bake the text to display
        text_renderer
            .prepare(
                device,
                queue,
                &mut font_system,
                &mut atlas,
                &glyphon_viewport,
                text_areas,
                &mut swash_cache,
            )
            .unwrap();

        Self {
            atlas,
            glyphon_viewport,
            text_renderer,
        }
    }

    /// Draws this [Text].
    ///
    /// Will overwrite bind groups.
    /// If `REBIND` is `true`, will call the passed `rebind`-function to rebind groups.
    pub fn draw<'a, const REBIND: bool>(
        &'a self,
        render_pass: &mut RenderPass<'a>,
        rebind: impl FnOnce(&mut RenderPass),
    ) {
        self.text_renderer
            .render(&self.atlas, &self.glyphon_viewport, render_pass)
            .unwrap();

        if REBIND {
            rebind(render_pass);
        }
    }
}

/// Creates a [glyphon::Buffer] of the passed `text`.
fn to_text_buffer(
    text: &str,
    font_system: &mut FontSystem,
    font_size: f32,
    font_family: &str,
) -> Buffer {
    let mut text_buffer = Buffer::new(font_system, Metrics::new(font_size, 1.2 * font_size));
    text_buffer.set_size(font_system, None, None);
    text_buffer.set_text(
        font_system,
        text,
        Attrs::new().family(Family::Name(font_family)),
        Shaping::Advanced,
    );
    text_buffer.shape_until_scroll(font_system, false);
    text_buffer
}

/// Creates a [TextArea] of the passed [glyphon::Buffer].
/// Will handle alignment.
fn to_text_area(
    text_buffer: &Buffer,
    pos: (f32, f32),
    alignment: Alignment,
    color: Color,
    viewport: ViewportProjection,
) -> TextArea {
    let bounds = TextBounds {
        left: 0,
        top: 0,
        right: viewport.source.width as i32,
        bottom: viewport.source.height as i32,
    };

    let (x, y) = get_aligned_position(
        alignment,
        pos,
        || {
            text_buffer
                .layout_runs()
                .map(|r| r.line_w)
                .fold(0., f32::max)
        },
        || text_buffer.layout_runs().map(|r| r.line_height).sum(),
    );

    // Average width and height scale:
    // Get average of width and height and divide by 100% (width/height of 2)
    let scale = ((viewport.target.width + viewport.target.height) / 2.) / 2.;

    // Transform the coordinates:
    let mat: Mat4 = viewport.into();
    // Into wgsl view-space
    let (x, y, _) = mat.transform_point3(Vec3::new(x, y, 0.)).into();
    fn map(val: f32, in_start: f32, in_end: f32, out_start: f32, out_end: f32) -> f32 {
        out_start + ((out_end - out_start) / (in_end - in_start)) * (val - in_start)
    }
    // Then map back into glyphon viewport-space
    let x = map(x, -1., 1., 0., viewport.source.width);
    let y = map(y, -1., 1., viewport.source.height, 0.);

    TextArea {
        buffer: text_buffer,
        left: x,
        top: y,
        scale,
        bounds,
        default_color: color,
        custom_glyphs: &[],
    }
}

/// Aligns an element at the passed position.
/// The width and height will be determined lazily if needed.
fn get_aligned_position(
    alignment: Alignment,
    (x, y): (f32, f32),
    w: impl FnOnce() -> f32,
    h: impl FnOnce() -> f32,
) -> (f32, f32) {
    let x = match alignment.0 {
        HAlignment::Left => x,
        HAlignment::Center => x - w() / 2.,
        HAlignment::Right => x - w(),
    };
    let y = match alignment.1 {
        VAlignment::Top => y,
        VAlignment::Center => y - h() / 2.,
        VAlignment::Bottom => y - h(),
    };
    (x, y)
}
