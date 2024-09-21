use naga_oil::compose::Composer;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BlendComponent, BlendFactor, BlendState, Buffer, BufferAddress,
    BufferUsages, ColorTargetState, ColorWrites, Device, FragmentState, MultisampleState,
    PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, TextureFormat, VertexAttribute, VertexBufferLayout, VertexState,
    VertexStepMode,
};

use crate::{globals::Globals, shaders::compile_shader, viewport::Viewport};

pub mod machine;
pub mod primitive;

/// The spec of a [Component].
pub struct ComponentSpec<'a, Spec: bytemuck::NoUninit> {
    /// The specifications for the instances
    specs: &'a [Spec],
    /// The vertex attributes
    attributes: &'a [VertexAttribute],
    /// The shader source code.
    /// Will be compiled using the passed [Composer].
    shader_source: &'static str,
    /// The path to the shader
    shader_path: &'static str,
    /// Optional uniform buffer group.
    /// Will be bound at group `2`.
    uniform: Option<(&'a [BindGroupLayoutEntry], &'a [BindGroupEntry<'a>])>,
}

/// A drawable component.
/// Groups together common setup.
///
/// Assumes a [Component] generates its vertices on  the GPU
/// and takes an instance buffer.
///
/// Binds [Globals] and [Viewport] to group `0` and `1`.
/// Allows binding a local uniform to group `2`.
pub struct Component {
    render_pipeline: RenderPipeline,
    instance_buffer: Buffer,
    instance_count: u32,
    bind_group: BindGroup,
}

impl Component {
    /// Creates a new [Component]
    pub fn new<Spec: bytemuck::NoUninit>(
        device: &Device,
        format: TextureFormat,
        globals: &Globals,
        viewport: &Viewport,
        shader_composer: &mut Composer,
        ComponentSpec {
            specs,
            attributes,
            shader_source,
            shader_path,
            uniform,
        }: ComponentSpec<Spec>,
    ) -> Self {
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: bytemuck::cast_slice(specs),
            usage: BufferUsages::VERTEX,
        });

        let instance_buffer_layout = VertexBufferLayout {
            array_stride: size_of::<Spec>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes,
        };

        let shader = compile_shader(
            device,
            shader_composer,
            shader_source,
            shader_path,
            Default::default(),
        )
        .unwrap_or_else(|_| panic!("Failed to load shader: {}", shader_path));

        let uniform = uniform.unwrap_or_default();

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: uniform.0,
            label: Some("uniform buffer group layout"),
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: uniform.1,
            label: Some("uniform buffer group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                globals.bind_group_layout(),
                viewport.bind_group_layout(),
                &bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[instance_buffer_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: BlendComponent::OVER,
                    }),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            render_pipeline,
            instance_buffer,
            instance_count: specs.len() as u32,
            bind_group,
        }
    }

    /// Draws this component
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        render_pass.set_bind_group(2, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..self.instance_count);
    }
}
