use std::{ops::Range, sync::Arc};

use eframe::wgpu;

use super::GraphicsState;

pub(super) struct CachedDynamicBuffer {
    label: Option<&'static str>,
    usage: wgpu::BufferUsages,
    element_size: usize,
    len: Option<usize>,
    buffer: Option<Arc<wgpu::Buffer>>,
}
impl CachedDynamicBuffer {
    pub fn new<T>(label: Option<&'static str>, usage: wgpu::BufferUsages) -> Self {
        Self {
            label,
            usage,
            element_size: std::mem::size_of::<T>(),
            len: None,
            buffer: None,
        }
    }

    pub fn at_len_at_least(&mut self, gfx: &GraphicsState, min_len: usize) -> Arc<wgpu::Buffer> {
        // Invalidate the buffer if it is too small.
        if let Some(len) = self.len {
            if len < min_len {
                self.buffer = None;
            }
        }

        Arc::clone(self.buffer.get_or_insert_with(|| {
            self.len = Some(min_len);
            Arc::new(gfx.device.create_buffer(&wgpu::BufferDescriptor {
                label: self.label,
                size: (min_len * self.element_size) as u64,
                usage: self.usage,
                mapped_at_creation: false,
            }))
        }))
    }

    pub fn slice(&mut self, gfx: &GraphicsState, len: usize) -> wgpu::BufferSlice<'_> {
        self.at_len_at_least(gfx, len);
        self.buffer
            .as_ref()
            .expect("buffer vanished")
            .slice(self.slice_bounds(len))
    }
    pub fn slice_bounds(&self, len: usize) -> Range<u64> {
        0..(len * self.element_size) as u64
    }

    pub fn write_all<T: Default + bytemuck::NoUninit>(
        &mut self,
        gfx: &GraphicsState,
        data: &mut Vec<T>,
    ) -> wgpu::BufferSlice<'_> {
        let original_len = data.len();
        super::pad_buffer_to_wgpu_copy_buffer_alignment(data);
        let buffer = self.at_len_at_least(gfx, data.len());
        gfx.queue
            .write_buffer(&buffer, 0, bytemuck::cast_slice(data));
        data.truncate(original_len); // undo padding
        self.slice(gfx, data.len())
    }
}
