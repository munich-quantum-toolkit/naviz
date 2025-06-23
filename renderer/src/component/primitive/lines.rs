use std::ops::{Deref, DerefMut};

use naga_oil::compose::Composer;
use wgpu::{BufferAddress, Device, TextureFormat, VertexAttribute, VertexFormat};

use crate::{
    component::{Component, ComponentSpec},
    globals::Globals,
    viewport::Viewport,
};

/// A [Component] which draws one or multiple lines to the screen
pub struct Lines(Component<LineSpec>);

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineSpec {
    /// The start of the line
    pub start: [f32; 2],
    /// The end of the line
    pub end: [f32; 2],
    /// The color of the line
    pub color: [u8; 4],
    /// The width of the line
    pub width: f32,
    /// The length of a dash-segment (both drawn and non-drawn)
    pub segment_length: f32,
    /// The duty-cycle of a dash-segment (how much of the segment should be drawn)
    pub duty: f32,
}

impl Lines {
    /// Create new [Lines]
    pub fn new(
        device: &Device,
        format: TextureFormat,
        globals: &Globals,
        viewport: &Viewport,
        shader_composer: &mut Composer,
        lines: &[LineSpec],
    ) -> Self {
        Self(Component::new(
            device,
            format,
            globals,
            viewport,
            shader_composer,
            ComponentSpec {
                specs: lines,
                attributes: &[
                    VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: VertexFormat::Float32x2,
                    },
                    VertexAttribute {
                        offset: size_of::<[f32; 2]>() as BufferAddress,
                        shader_location: 1,
                        format: VertexFormat::Float32x2,
                    },
                    VertexAttribute {
                        offset: (size_of::<[f32; 2]>() + size_of::<[f32; 2]>()) as BufferAddress,
                        shader_location: 2,
                        format: VertexFormat::Uint32,
                    },
                    VertexAttribute {
                        offset: (size_of::<[f32; 2]>() + size_of::<[f32; 2]>() + size_of::<u32>())
                            as BufferAddress,
                        shader_location: 3,
                        format: VertexFormat::Float32,
                    },
                    VertexAttribute {
                        offset: (size_of::<[f32; 2]>()
                            + size_of::<[f32; 2]>()
                            + size_of::<u32>()
                            + size_of::<f32>()) as BufferAddress,
                        shader_location: 4,
                        format: VertexFormat::Float32,
                    },
                    VertexAttribute {
                        offset: (size_of::<[f32; 2]>()
                            + size_of::<[f32; 2]>()
                            + size_of::<u32>()
                            + size_of::<f32>()
                            + size_of::<f32>()) as BufferAddress,
                        shader_location: 5,
                        format: VertexFormat::Float32,
                    },
                ],
                shader_source: include_str!("lines.wgsl"),
                shader_path: "lines.wgsl",
                uniform: None,
            },
        ))
    }
}

impl Deref for Lines {
    type Target = Component<LineSpec>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Lines {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
