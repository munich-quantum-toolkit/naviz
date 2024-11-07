use std::num::NonZeroU64;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device, Queue,
};

/// Implementors of this trait can update the passed buffer.
/// Will grow (and shrink) if needed.
pub trait BufferUpdater {
    fn update<T: bytemuck::NoUninit>(
        &mut self,
        buffer: &mut Buffer,
        data: &[T],
        label: Option<&'static str>,
        usage: BufferUsages,
    );
}

impl BufferUpdater for (&Device, &Queue) {
    fn update<T: bytemuck::NoUninit>(
        &mut self,
        buffer: &mut Buffer,
        data: &[T],
        label: Option<&'static str>,
        usage: BufferUsages,
    ) {
        let data_size = size_of_val(data) as u64;
        let buffer_size = buffer.size();

        let (device, queue) = self;

        if buffer_size < data_size /* need to increase size */
        || buffer_size / 2 > data_size
        /* can free some memory */
        {
            buffer.destroy();
            *buffer = device.create_buffer_init(&BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(data),
                usage,
            });
        } else if let Some(data_size) = NonZeroU64::new(data_size) {
            queue
                .write_buffer_with(buffer, 0, data_size)
                .unwrap()
                .copy_from_slice(bytemuck::cast_slice(data));
        }
    }
}
