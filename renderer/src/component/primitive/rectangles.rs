use std::ops::Deref;

use naga_oil::compose::Composer;
use wgpu::{Device, TextureFormat};

use crate::{component::Component, globals::Globals, viewport::Viewport};

use super::lines::{LineSpec, Lines};

pub struct RectangleSpec {
    // Position of the rectangle
    pub start: [f32; 2],
    // Size of the rectangle
    pub size: [f32; 2],
    /// The color of the line
    pub color: [u8; 4],
    /// The width of the line
    pub width: f32,
    /// The length of a dash-segment (both drawn and non-drawn)
    pub segment_length: f32,
    /// The duty-cycle of a dash-segment (how much of the segment should be drawn)
    pub duty: f32,
}

/// A [Component] which draws one or multiple rectangles to the screen
pub struct Rectangles(Lines);

impl Rectangles {
    pub fn new(
        device: &Device,
        format: TextureFormat,
        globals: &Globals,
        viewport: &Viewport,
        shader_composer: &mut Composer,
        rectangles: &[RectangleSpec],
    ) -> Self {
        Self(Lines::new(
            device,
            format,
            globals,
            viewport,
            shader_composer,
            &rectangles_to_lines(rectangles),
        ))
    }
}

/// Converts a slice of [RectangleSpec]s to a [Vec] of [LineSpec]s
fn rectangles_to_lines(rectangles: &[RectangleSpec]) -> Vec<LineSpec> {
    rectangles
        .iter()
        .flat_map(
            |RectangleSpec {
                 start: [x, y],
                 size: [w, h],
                 color,
                 width,
                 segment_length,
                 duty,
             }| {
                // Offset positions by half line-width to prevent ugly corners
                let delta = width / 2.;
                // +----->
                // |      ^
                // v      |
                //  <-----+
                [
                    LineSpec {
                        start: [*x - delta, *y],
                        end: [*x + *w + delta, *y],
                        color: *color,
                        width: *width,
                        segment_length: *segment_length,
                        duty: *duty,
                    },
                    LineSpec {
                        end: [*x + *w, *y - delta],
                        start: [*x + *w, *y + *h + delta],
                        color: *color,
                        width: *width,
                        segment_length: *segment_length,
                        duty: *duty,
                    },
                    LineSpec {
                        start: [*x + *w + delta, *y + *h],
                        end: [*x - delta, *y + *h],
                        color: *color,
                        width: *width,
                        segment_length: *segment_length,
                        duty: *duty,
                    },
                    LineSpec {
                        end: [*x, *y + *h + delta],
                        start: [*x, *y - delta],
                        color: *color,
                        width: *width,
                        segment_length: *segment_length,
                        duty: *duty,
                    },
                ]
            },
        )
        .collect()
}

impl Deref for Rectangles {
    type Target = Component;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
