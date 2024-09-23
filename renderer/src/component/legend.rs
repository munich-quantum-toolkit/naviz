use naga_oil::compose::Composer;
use wgpu::{Device, Queue, RenderPass, TextureFormat};

use crate::{
    globals::Globals,
    viewport::{Viewport, ViewportProjection},
};

use super::primitive::{
    circles::{CircleSpec, Circles},
    text::{Alignment, HAlignment, Text, TextSpec, VAlignment},
};

#[derive(Clone, Copy, Debug)]
pub struct LegendEntrySpec<'a> {
    /// The text to display
    pub text: &'a str,
    /// The color for the circle next to the text.
    /// [None] will render no circle.
    pub color: Option<[u8; 4]>,
}

#[derive(Clone, Copy, Debug)]
pub struct LegendSpec<'a, EntryIterator>
where
    for<'r> &'r EntryIterator: IntoIterator<Item = &'r (&'a str, &'a [LegendEntrySpec<'a>])>,
{
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
    /// How much to skip before each heading.
    /// Includes the font size
    /// (i.e., a skip of `0` results in the next item overlapping the heading).
    pub heading_skip: f32,
    /// How much to skip before each entry.
    /// Includes the font size
    /// (i.e., a skip of `0` results in the next item overlapping the entry).
    pub entry_skip: f32,
    /// The radius of the circles showing the color
    pub color_circle_radius: f32,
    /// The padding between a circle and the text
    pub color_padding: f32,
    /// The legend entries
    pub entries: EntryIterator,
}

/// A component to draw the legend:
/// - A heading per block
/// - Entries, with an optional colored circle
pub struct Legend {
    viewport: Viewport,
    text: Text,
    colors: Circles,
}

impl Legend {
    pub fn new<'a, EntryIterator>(
        device: &Device,
        queue: &Queue,
        format: TextureFormat,
        globals: &Globals,
        shader_composer: &mut Composer,
        LegendSpec {
            viewport,
            screen_resolution,
            font_size,
            text_color,
            font,
            heading_skip,
            entry_skip,
            color_circle_radius,
            color_padding,
            entries,
        }: LegendSpec<'a, EntryIterator>,
    ) -> Self
    where
        for<'r> &'r EntryIterator: IntoIterator<Item = &'r (&'a str, &'a [LegendEntrySpec<'a>])>,
    {
        // Layout the texts.
        // Will always lay out heading, then all entries, each separated by `entry_skip`.
        // After the block, additional space will be skipped so that the distance to the next
        // heading will be `heading_skip`.
        let num_texts =
            entries.into_iter().map(|e| e.1.len()).sum::<usize>() + entries.into_iter().count();
        let num_colors = entries
            .into_iter()
            .map(|e| e.1.iter().filter(|e| e.color.is_some()).count())
            .sum::<usize>();
        let mut colors = Vec::with_capacity(num_colors);
        let mut texts = Vec::with_capacity(num_texts);
        let mut y = heading_skip; // Start with margin from top
        for (heading, elements) in entries.into_iter() {
            // heading
            texts.push((
                *heading,
                (0., y),
                Alignment(HAlignment::Left, VAlignment::Center),
            ));
            y += entry_skip;

            // entries
            for LegendEntrySpec { text, color } in elements.iter() {
                // text
                texts.push((
                    text,
                    (2. * color_circle_radius + color_padding, y),
                    Alignment(HAlignment::Left, VAlignment::Center),
                ));

                // colored circle
                if let Some(color) = color {
                    colors.push(CircleSpec {
                        center: [color_circle_radius, y],
                        radius: color_circle_radius,
                        radius_inner: 0.,
                        color: *color,
                    });
                }
                y += entry_skip;
            }

            // end: increase skip to heading_skip
            y += heading_skip - entry_skip;
        }

        let viewport_projection = viewport;
        let viewport = Viewport::new(viewport, device);

        Self {
            text: Text::new(
                device,
                queue,
                format,
                TextSpec {
                    viewport_projection,
                    font_size,
                    font_family: font,
                    texts,
                    color: text_color,
                    screen_resolution,
                },
            ),
            colors: Circles::new(device, format, globals, &viewport, shader_composer, &colors),
            viewport,
        }
    }

    /// Draws this [Legend].
    ///
    /// May overwrite bind groups.
    /// If `REBIND` is `true`, will call the passed `rebind`-function to rebind groups.
    pub fn draw<'a, const REBIND: bool>(
        &'a self,
        render_pass: &mut RenderPass<'a>,
        rebind: impl Fn(&mut RenderPass),
    ) {
        self.viewport.bind(render_pass);
        self.colors.draw(render_pass);
        self.text.draw::<REBIND>(render_pass, rebind);
    }
}
