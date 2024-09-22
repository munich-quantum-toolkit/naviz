use naga_oil::compose::Composer;
use wgpu::{Device, Queue, RenderPass, TextureFormat};

use crate::{
    globals::Globals,
    viewport::{Viewport, ViewportProjection},
};

use super::primitive::{
    circles::{CircleSpec, Circles},
    lines::{LineSpec, Lines},
    text::{Alignment, HAlignment, Text, TextSpec, VAlignment},
};

/// The parameters for [Atoms]
#[derive(Clone, Copy, Debug)]
pub struct AtomsSpec<'a, AtomIterator>
where
    for<'r> &'r AtomIterator: IntoIterator<Item = &'r AtomSpec<'a>>,
{
    /// The viewport to render in
    pub viewport: ViewportProjection,
    /// The resolution of the screen.
    /// Will render text at this resolution.
    pub screen_resolution: (u32, u32),
    /// The atoms
    pub atoms: AtomIterator,
    /// The color of the shuttle lines
    pub shuttle_color: [u8; 4],
    /// The width of the shuttle lines
    pub shuttle_line_width: f32,
    /// The segment length of the shuttle lines
    pub shuttle_segment_length: f32,
    /// The duty-cycle of the shuttle lines
    pub shuttle_duty: f32,
    /// The font size of the labels
    pub label_font_size: f32,
    /// The font of the labels
    pub label_font: &'a str,
    /// The color of the labels
    pub label_color: [u8; 4],
}

/// The parameters of a single atom
#[derive(Clone, Copy, Debug)]
pub struct AtomSpec<'a> {
    /// The position of this atom
    pub pos: [f32; 2],
    /// The size of this atom
    pub size: f32,
    /// The color of this atom
    pub color: [u8; 4],
    /// Whether this atom is currently shuttling
    /// (i.e., whether to draw shuttle lines)
    pub shuttle: bool,
    /// The label of this atom
    pub label: &'a str,
}

/// A component to draw atoms:
/// - Circle representing atom
/// - Shuttle lines
/// - Label
pub struct Atoms {
    viewport: Viewport,
    atoms: Circles,
    shuttles: Lines,
    labels: Text,
}

impl Atoms {
    pub fn new<'a, AtomIterator>(
        device: &Device,
        queue: &Queue,
        format: TextureFormat,
        globals: &Globals,
        shader_composer: &mut Composer,
        AtomsSpec {
            viewport,
            screen_resolution,
            atoms,
            shuttle_color,
            shuttle_line_width,
            shuttle_segment_length,
            shuttle_duty,
            label_font_size,
            label_font,
            label_color,
        }: AtomsSpec<'a, AtomIterator>,
    ) -> Self
    where
        for<'r> &'r AtomIterator: IntoIterator<Item = &'r AtomSpec<'a>>,
    {
        // The circles for the atoms
        let atom_circles: Vec<_> = atoms
            .into_iter()
            .map(
                |AtomSpec {
                     pos,
                     size,
                     color,
                     shuttle: _,
                     label: _,
                 }| CircleSpec {
                    center: *pos,
                    radius: *size,
                    color: *color,
                    radius_inner: 0.,
                },
            )
            .collect();

        // The shuttle lines
        let shuttles: Vec<_> = atoms
            .into_iter()
            .filter(|s| s.shuttle)
            .flat_map(
                |AtomSpec {
                     pos: [x, y],
                     size: _,
                     color: _,
                     shuttle: _,
                     label: _,
                 }| {
                    [
                        LineSpec {
                            start: [*x, 0.],
                            end: [*x, viewport.source.height],
                            color: shuttle_color,
                            width: shuttle_line_width,
                            segment_length: shuttle_segment_length,
                            duty: shuttle_duty,
                        },
                        LineSpec {
                            start: [0., *y],
                            end: [viewport.source.width, *y],
                            color: shuttle_color,
                            width: shuttle_line_width,
                            segment_length: shuttle_segment_length,
                            duty: shuttle_duty,
                        },
                    ]
                },
            )
            .collect();

        // The labels
        let labels: Vec<_> = atoms
            .into_iter()
            .map(
                |AtomSpec {
                     pos: [x, y],
                     size: _,
                     color: _,
                     shuttle: _,
                     label,
                 }| {
                    (
                        *label,
                        (*x, *y),
                        Alignment(HAlignment::Center, VAlignment::Center),
                    )
                },
            )
            .collect();

        let viewport_projection = viewport;
        let viewport = Viewport::new(viewport, device);

        Self {
            atoms: Circles::new(
                device,
                format,
                globals,
                &viewport,
                shader_composer,
                &atom_circles,
            ),
            shuttles: Lines::new(
                device,
                format,
                globals,
                &viewport,
                shader_composer,
                &shuttles,
            ),
            labels: Text::new(
                device,
                queue,
                format,
                TextSpec {
                    viewport_projection,
                    font_size: label_font_size,
                    font_family: label_font,
                    texts: labels,
                    color: label_color,
                    screen_resolution,
                },
            ),
            viewport,
        }
    }

    /// Draws these [Atoms].
    ///
    /// May overwrite bind groups.
    /// If `REBIND` is `true`, will call the passed `rebind`-function to rebind groups.
    pub fn draw<'a, const REBIND: bool>(
        &'a self,
        render_pass: &mut RenderPass<'a>,
        rebind: impl Fn(&mut RenderPass),
    ) {
        self.viewport.bind(render_pass);
        self.shuttles.draw(render_pass);
        self.atoms.draw(render_pass);
        self.labels.draw::<REBIND>(render_pass, rebind);
    }
}
