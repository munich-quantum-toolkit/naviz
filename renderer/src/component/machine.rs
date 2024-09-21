use naga_oil::compose::Composer;
use wgpu::{Device, Queue, RenderPass, TextureFormat};

use crate::{
    globals::Globals,
    viewport::{Viewport, ViewportProjection},
};

use super::primitive::{
    circles::{CircleSpec, Circles},
    lines::{LineSpec, Lines},
    rectangles::{RectangleSpec, Rectangles},
    text::{Alignment, HAlignment, Text, TextSpec, VAlignment},
};

/// A vertical position ([Top][VPosition::Top] or [Bottom][VPosition::Bottom])
pub enum VPosition {
    Top,
    Bottom,
}

impl VPosition {
    /// Selects the y-coordinate based off of this position
    #[inline]
    fn get_y(&self, top: f32, bottom: f32) -> f32 {
        match self {
            Self::Top => top,
            Self::Bottom => bottom,
        }
    }

    /// Gets an alignment based off of this position
    #[inline]
    fn get_alignment(&self) -> VAlignment {
        match self {
            Self::Top => VAlignment::Bottom,
            Self::Bottom => VAlignment::Top,
        }
    }

    /// Gets the inverse of this position
    #[inline]
    fn inverse(&self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
        }
    }
}

/// A horizontal position ([Left][HPosition::Left] or [Right][HPosition::Right])
pub enum HPosition {
    Left,
    Right,
}

impl HPosition {
    /// Selects the y-coordinate based off of this position
    #[inline]
    fn get_x(&self, left: f32, right: f32) -> f32 {
        match self {
            Self::Left => left,
            Self::Right => right,
        }
    }

    /// Gets an alignment based off of this position
    #[inline]
    fn get_alignment(&self) -> HAlignment {
        match self {
            Self::Left => HAlignment::Right,
            Self::Right => HAlignment::Left,
        }
    }

    /// Gets the inverse of this position
    #[inline]
    fn inverse(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

pub struct MachineSpec<'a> {
    /// The viewport to fill.
    /// Will be the target of the grid.
    /// Labels will be drawn outside of the viewport
    pub viewport: ViewportProjection,
    /// The step to draw grid-lines in
    pub grid_step: (f32, f32),
    /// The color of the grid lines
    pub grid_color: [u8; 4],
    /// The line width of the grid lines
    pub grid_line_width: f32,
    /// The segment length of the grid lines
    pub grid_segment_length: f32,
    /// The fraction filled in each grid segment
    pub grid_segment_duty: f32,
    /// The step to display the numbers in
    pub legend_step: (f32, f32),
    /// The font size of the numbers and labels
    pub legend_font_size: f32,
    /// The color of the numbers and labels
    pub legend_color: [u8; 4],
    /// The font of the numbers and labels
    pub legend_font: &'a str,
    /// The labels for the x- and y-axis respectively
    pub legend_labels: (&'a str, &'a str),
    /// Where to draw the numbers and labels
    pub legend_position: (VPosition, HPosition),
    /// The positions of the static traps
    pub traps: &'a [(f32, f32)],
    /// The radius of all static traps
    pub trap_radius: f32,
    /// The line width of the static traps
    pub trap_line_width: f32,
    /// The color of the static traps
    pub trap_color: [u8; 4],
    /// The specifications for the zones. See [RectangleSpec]
    pub zones: &'a [RectangleSpec],
}

/// A component to draw the machine background:
/// - Background grid and coordinate legend
/// - Static traps
/// - Zones
pub struct Machine {
    viewport: Viewport,
    background_grid: Lines,
    static_traps: Circles,
    coordinate_legend: Text,
    zones: Rectangles,
}

/// Padding between the grid and the legend (numbers and labels)
const LABEL_PADDING: f32 = 12.;

impl Machine {
    pub fn new(
        device: &Device,
        queue: &Queue,
        format: TextureFormat,
        globals: &Globals,
        shader_composer: &mut Composer,
        MachineSpec {
            viewport,
            grid_step,
            grid_color,
            grid_line_width,
            grid_segment_length,
            grid_segment_duty,
            legend_step,
            legend_font_size,
            legend_color,
            legend_font,
            legend_labels,
            legend_position: (legend_pos_x, legend_pos_y),
            traps,
            trap_radius,
            trap_line_width,
            trap_color,
            zones,
        }: MachineSpec,
    ) -> Self {
        // Create the LineSpecs for the grid; first x, then y
        let lines: Vec<_> = range_f32(0., viewport.source.width, grid_step.0)
            .map(|x| LineSpec {
                start: [x, 0.],
                end: [x, viewport.source.height],
                color: grid_color,
                width: grid_line_width,
                segment_length: grid_segment_length,
                duty: grid_segment_duty,
            })
            .chain(
                range_f32(0., viewport.source.height, grid_step.1).map(|y| LineSpec {
                    start: [0., y],
                    end: [viewport.source.width, y],
                    color: grid_color,
                    width: grid_line_width,
                    segment_length: grid_segment_length,
                    duty: grid_segment_duty,
                }),
            )
            .collect();

        // Create the CircleSpecs for the static traps
        let traps: Vec<_> = traps
            .iter()
            .map(|(x, y)| CircleSpec {
                center: [*x, *y],
                radius: trap_radius,
                radius_inner: trap_radius - trap_line_width,
                color: trap_color,
            })
            .collect();

        // Create the text specs for the numbers numbers (x then y)
        // First create strings, then convert to text spec
        let texts: Vec<_> = range_f32(0., viewport.source.width, legend_step.0)
            .map(|x| {
                (
                    format!("{x}"),
                    (
                        x,
                        legend_pos_x.get_y(-LABEL_PADDING, viewport.source.height + LABEL_PADDING),
                    ),
                    Alignment(HAlignment::Center, legend_pos_x.get_alignment()),
                )
            })
            .chain(
                range_f32(0., viewport.source.height, legend_step.1).map(|y| {
                    (
                        format!("{y}"),
                        (
                            legend_pos_y
                                .get_x(-LABEL_PADDING, viewport.source.width + LABEL_PADDING),
                            y,
                        ),
                        Alignment(legend_pos_y.get_alignment(), VAlignment::Center),
                    )
                }),
            )
            .collect();
        let mut texts: Vec<_> = texts.iter().map(|(t, p, a)| (t.as_str(), *p, *a)).collect();
        // Add axis labels
        texts.push((
            legend_labels.0,
            (
                legend_pos_y
                    .inverse()
                    .get_x(-LABEL_PADDING, viewport.source.width + LABEL_PADDING),
                legend_pos_x.get_y(0., viewport.source.height),
            ),
            Alignment(legend_pos_y.inverse().get_alignment(), VAlignment::Center),
        ));
        texts.push((
            legend_labels.1,
            (
                legend_pos_y.get_x(0., viewport.source.width),
                legend_pos_x
                    .inverse()
                    .get_y(-LABEL_PADDING, viewport.source.height + LABEL_PADDING),
            ),
            Alignment(HAlignment::Center, legend_pos_x.inverse().get_alignment()),
        ));

        let viewport_projection = viewport;
        let viewport = Viewport::new(viewport, device);

        Self {
            background_grid: Lines::new(
                device,
                format,
                globals,
                &viewport,
                shader_composer,
                &lines,
            ),
            static_traps: Circles::new(device, format, globals, &viewport, shader_composer, &traps),
            coordinate_legend: Text::new(
                device,
                queue,
                format,
                TextSpec {
                    viewport_projection,
                    font_size: legend_font_size,
                    font_family: legend_font,
                    texts: &texts,
                    color: legend_color,
                },
            ),
            zones: Rectangles::new(device, format, globals, &viewport, shader_composer, zones),
            viewport,
        }
    }

    /// Draws this [Machine].
    ///
    /// May overwrite bind groups.
    /// If `REBIND` is `true`, will call the passed `rebind`-function to rebind groups.
    pub fn draw<'a, const REBIND: bool>(
        &'a self,
        render_pass: &mut RenderPass<'a>,
        rebind: impl Fn(&mut RenderPass),
    ) {
        self.viewport.bind(render_pass);
        self.background_grid.draw(render_pass);
        self.static_traps.draw(render_pass);
        self.zones.draw(render_pass);
        self.coordinate_legend.draw::<REBIND>(render_pass, rebind);
    }
}

/// Creates an iterator that yields [f32]s from `start` to `end` (both included) in steps of `step`
fn range_f32(start: f32, end: f32, step: f32) -> impl Iterator<Item = f32> {
    let len = end - start;
    let steps = (len / step) as u64;
    (0..=steps).map(move |i| start + (i as f32 * step))
}
