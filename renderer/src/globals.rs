use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, Device, RenderPass,
};

/// The globals, which hold all uniform buffers unique to a frame.
///
/// Will bind to group `0`.
pub struct Globals {
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,
}

impl Globals {
    /// Creates new globals.
    pub fn new(device: &Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[],
            label: None,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[],
            label: None,
        });

        Self {
            bind_group,
            bind_group_layout,
        }
    }

    /// Binds these [Globals] to group `1`
    pub fn bind(&self, render_pass: &mut RenderPass<'_>) {
        render_pass.set_bind_group(0, &self.bind_group, &[]);
    }

    /// The bind group layout for these [Globals]
    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }
}
