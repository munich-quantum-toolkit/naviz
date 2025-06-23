use std::ops::{Deref, DerefMut};

use naga_oil::compose::Composer;
use wgpu::{BufferAddress, Device, TextureFormat, VertexAttribute, VertexFormat};

use crate::{
    component::{Component, ComponentSpec},
    globals::Globals,
    viewport::Viewport,
};

/// A [Component] which draws one or multiple circles to the screen
pub struct Circles(Component<CircleSpec>);

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleSpec {
    /// The center of the circle
    pub center: [f32; 2],
    /// The radius of the circle
    pub radius: f32,
    /// The radius of the inner (transparent) circle / cutout
    pub radius_inner: f32,
    /// The color of the circle
    pub color: [u8; 4],
}

impl Circles {
    /// Create new [Circles]
    pub fn new(
        device: &Device,
        format: TextureFormat,
        globals: &Globals,
        viewport: &Viewport,
        shader_composer: &mut Composer,
        circles: &[CircleSpec],
    ) -> Self {
        Self(Component::new(
            device,
            format,
            globals,
            viewport,
            shader_composer,
            ComponentSpec {
                specs: circles,
                attributes: &[
                    VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: VertexFormat::Float32x2,
                    },
                    VertexAttribute {
                        offset: size_of::<[f32; 2]>() as BufferAddress,
                        shader_location: 1,
                        format: VertexFormat::Float32,
                    },
                    VertexAttribute {
                        offset: (size_of::<[f32; 2]>() + size_of::<f32>()) as BufferAddress,
                        shader_location: 2,
                        format: VertexFormat::Float32,
                    },
                    VertexAttribute {
                        offset: (size_of::<[f32; 2]>() + size_of::<f32>() + size_of::<f32>())
                            as BufferAddress,
                        shader_location: 3,
                        format: VertexFormat::Uint32,
                    },
                ],
                shader_source: include_str!("circles.wgsl"),
                shader_path: "circles.wgsl",
                uniform: None,
            },
        ))
    }
}

impl Deref for Circles {
    type Target = Component<CircleSpec>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Circles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
