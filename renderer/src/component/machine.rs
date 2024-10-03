use naviz_state::{
    config::{Config, HPosition, LineConfig, MachineConfig, VPosition, ZoneConfig},
    state::State,
};
use wgpu::{Device, Queue, RenderPass};

use crate::{
    buffer_updater::BufferUpdater,
    viewport::{Viewport, ViewportProjection},
};

use super::{
    primitive::{
        circles::{CircleSpec, Circles},
        lines::{LineSpec, Lines},
        rectangles::{RectangleSpec, Rectangles},
        text::{Alignment, HAlignment, Text, TextSpec, VAlignment},
    },
    ComponentInit,
};

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
    viewport_projection: ViewportProjection,
}

/// Padding between the grid and the legend (numbers and labels)
const LABEL_PADDING: f32 = 12.;

impl Machine {
    pub fn new(
        ComponentInit {
            device,
            queue,
            format,
            globals,
            shader_composer,
            config,
            state: _,
            viewport_projection,
            screen_resolution,
        }: ComponentInit,
    ) -> Self {
        let mut text_buffer = Vec::new();
        let (lines, traps, labels, zones) =
            get_specs(config, viewport_projection, &mut text_buffer);
        let viewport = Viewport::new(viewport_projection, device);

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
            coordinate_legend: Text::new(device, queue, format, labels, screen_resolution),
            zones: Rectangles::new(device, format, globals, &viewport, shader_composer, zones),
            viewport,
            viewport_projection,
        }
    }

    /// Updates this [Machine] to resemble the new [State].
    /// If `FULL` is `true`, also update this [Machine] to resemble the new [Config].
    /// Not that all elements which depend on [State] will always update to resemble the new [State],
    /// regardless of the value of `FULL`.
    pub fn update<const FULL: bool>(
        &mut self,
        updater: &mut impl BufferUpdater,
        device: &Device,
        queue: &Queue,
        config: &Config,
        _state: &State,
    ) {
        if FULL {
            let mut text_buffer = Vec::new();
            let (lines, traps, labels, zones) =
                get_specs(config, self.viewport_projection, &mut text_buffer);
            self.background_grid.update(updater, &lines);
            self.static_traps.update(updater, &traps);
            self.coordinate_legend.update((device, queue), labels);
            self.zones.update(updater, zones);
        }
    }

    /// Updates the viewport resolution of this [Machine]
    pub fn update_viewport(
        &mut self,
        device: &Device,
        queue: &Queue,
        screen_resolution: (u32, u32),
    ) {
        self.coordinate_legend
            .update_viewport((device, queue), screen_resolution);
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

/// Gets the specs for [Machine] from the passed [State] and [Config].
fn get_specs<'a>(
    config: &'a Config,
    viewport_projection: ViewportProjection,
    text_buffer: &'a mut Vec<(String, (f32, f32), Alignment)>,
) -> (
    Vec<LineSpec>,
    Vec<CircleSpec>,
    TextSpec<'a, impl IntoIterator<Item = (&'a str, (f32, f32), Alignment)>>,
    Vec<RectangleSpec>,
) {
    let MachineConfig { grid, traps, zones } = &config.machine;

    // Create the LineSpecs for the grid; first x, then y
    let lines: Vec<_> = range_f32(0., viewport_projection.source.width, grid.step.0)
        .map(|x| LineSpec {
            start: [x, 0.],
            end: [x, viewport_projection.source.height],
            color: grid.line.color,
            width: grid.line.width,
            segment_length: grid.line.segment_length,
            duty: grid.line.duty,
        })
        .chain(
            range_f32(0., viewport_projection.source.height, grid.step.1).map(|y| LineSpec {
                start: [0., y],
                end: [viewport_projection.source.width, y],
                color: grid.line.color,
                width: grid.line.width,
                segment_length: grid.line.segment_length,
                duty: grid.line.duty,
            }),
        )
        .collect();

    // Create the CircleSpecs for the static traps
    let traps: Vec<_> = traps
        .positions
        .iter()
        .map(|(x, y)| CircleSpec {
            center: [*x, *y],
            radius: traps.radius,
            radius_inner: traps.radius - traps.line_width,
            color: traps.color,
        })
        .collect();

    // Create the text specs for the numbers numbers (x then y)
    // First create strings, then convert to text spec
    *text_buffer = range_f32(0., viewport_projection.source.width, grid.legend.step.0)
        .map(|x| {
            (
                format!("{x}"),
                (
                    x,
                    grid.legend.position.0.get(
                        -LABEL_PADDING,
                        viewport_projection.source.height + LABEL_PADDING,
                    ),
                ),
                Alignment(HAlignment::Center, get_v_alignment(grid.legend.position.0)),
            )
        })
        .chain(
            range_f32(0., viewport_projection.source.height, grid.legend.step.1).map(|y| {
                (
                    format!("{y}"),
                    (
                        grid.legend.position.1.get(
                            -LABEL_PADDING,
                            viewport_projection.source.width + LABEL_PADDING,
                        ),
                        y,
                    ),
                    Alignment(get_h_alignment(grid.legend.position.1), VAlignment::Center),
                )
            }),
        )
        .collect();
    let texts = text_buffer.iter().map(|(t, p, a)| (t.as_str(), *p, *a));
    // Add axis labels
    let texts = texts.chain([
        (
            grid.legend.labels.0.as_str(),
            (
                grid.legend.position.1.inverse().get(
                    -LABEL_PADDING,
                    viewport_projection.source.width + LABEL_PADDING,
                ),
                grid.legend
                    .position
                    .0
                    .get(0., viewport_projection.source.height),
            ),
            Alignment(
                get_h_alignment(grid.legend.position.1.inverse()),
                VAlignment::Center,
            ),
        ),
        (
            grid.legend.labels.1.as_str(),
            (
                grid.legend
                    .position
                    .1
                    .get(0., viewport_projection.source.width),
                grid.legend.position.0.inverse().get(
                    -LABEL_PADDING,
                    viewport_projection.source.height + LABEL_PADDING,
                ),
            ),
            Alignment(
                HAlignment::Center,
                get_v_alignment(grid.legend.position.0.inverse()),
            ),
        ),
    ]);

    let zones: Vec<_> = zones
        .iter()
        .copied()
        .map(
            |ZoneConfig {
                 start,
                 size,
                 line:
                     LineConfig {
                         width,
                         segment_length,
                         duty,
                         color,
                     },
             }| RectangleSpec {
                start: start.into(),
                size: size.into(),
                color,
                width,
                duty,
                segment_length,
            },
        )
        .collect();

    (
        lines,
        traps,
        TextSpec {
            viewport_projection,
            font_size: grid.legend.font.size,
            font_family: &grid.legend.font.family,
            texts,
            color: grid.legend.font.color,
        },
        zones,
    )
}

/// Gets the [VAlignment] based on the passed [VPosition]
#[inline]
fn get_v_alignment(p: VPosition) -> VAlignment {
    p.get(VAlignment::Bottom, VAlignment::Top)
}

/// Gets the [HAlignment] based on the passed [HPosition]
#[inline]
fn get_h_alignment(p: HPosition) -> HAlignment {
    p.get(HAlignment::Right, HAlignment::Left)
}
