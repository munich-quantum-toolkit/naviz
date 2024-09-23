use wgpu::{Device, Queue, RenderPass, TextureFormat};

use crate::viewport::ViewportProjection;

use super::primitive::text::{Alignment, HAlignment, Text, TextSpec, VAlignment};

#[derive(Clone, Copy, Debug)]
pub struct TimeSpec<'a> {
    /// The viewport to render in
    pub viewport: ViewportProjection,
    /// The resolution of the screen.
    /// Will render text at this resolution.
    pub screen_resolution: (u32, u32),
    /// The font size
    pub font_size: f32,
    /// The color of the text
    pub text_color: [u8; 4],
    /// The font
    pub font: &'a str,
    /// The full text to render
    pub text: &'a str,
}

/// A component to display the time on the screen
pub struct Time {
    text: Text,
}

impl Time {
    pub fn new(
        device: &Device,
        queue: &Queue,
        format: TextureFormat,
        TimeSpec {
            viewport,
            screen_resolution,
            font_size,
            text_color,
            font,
            text,
        }: TimeSpec,
    ) -> Self {
        Self {
            text: Text::new(
                device,
                queue,
                format,
                TextSpec {
                    viewport_projection: viewport,
                    font_size,
                    font_family: font,
                    texts: [(
                        text,
                        (0., viewport.source.height / 2.),
                        Alignment(HAlignment::Left, VAlignment::Center),
                    )],
                    color: text_color,
                    screen_resolution,
                },
            ),
        }
    }

    /// Draws this [Time].
    ///
    /// May overwrite bind groups.
    /// If `REBIND` is `true`, will call the passed `rebind`-function to rebind groups.
    #[inline]
    pub fn draw<'a, const REBIND: bool>(
        &'a self,
        render_pass: &mut RenderPass<'a>,
        rebind: impl Fn(&mut RenderPass),
    ) {
        self.text.draw::<REBIND>(render_pass, rebind);
    }
}
