#[macro_use]
mod macros;
mod bindings;
mod cache;
mod draw;
mod pipelines;
mod state;
mod structs;

pub(crate) use draw::RenderResources;
use eframe::wgpu;
pub use state::*;
pub use structs::CircleInstance;

/// Pads a buffer to `wgpu::COPY_BUFFER_ALIGNMENT`.
fn pad_buffer_to_wgpu_copy_buffer_alignment<T: Default + bytemuck::NoUninit>(buf: &mut Vec<T>) {
    loop {
        let bytes_len = bytemuck::cast_slice::<T, u8>(buf).len();
        if bytes_len > 0 && bytes_len as u64 % wgpu::COPY_BUFFER_ALIGNMENT == 0 {
            break;
        }
        buf.push(T::default());
    }
}
