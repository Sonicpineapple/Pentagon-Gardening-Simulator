//! Structs shared between the CPU and GPU (vertices, uniforms, etc.).

use eframe::wgpu;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, bytemuck::NoUninit, bytemuck::Zeroable)]
pub(super) struct UvVertex {
    pub position: [f32; 2],
    pub offset: [f32; 2],
}
impl UvVertex {
    const fn new(position: [f32; 2], offset: [f32; 2]) -> Self {
        Self { position, offset }
    }
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
        ],
    };
    pub const SQUARE: [Self; 4] = [
        UvVertex::new([-1.0, 1.0], [-1.0, 1.0]),
        UvVertex::new([1.0, 1.0], [1.0, 1.0]),
        UvVertex::new([-1.0, -1.0], [-1.0, -1.0]),
        UvVertex::new([1.0, -1.0], [1.0, -1.0]),
    ];

    pub const BLIT_SQUARE: [Self; 4] = [
        UvVertex::new([-1.0, 1.0], [0.0, 0.0]),
        UvVertex::new([1.0, 1.0], [1.0, 0.0]),
        UvVertex::new([-1.0, -1.0], [0.0, 1.0]),
        UvVertex::new([1.0, -1.0], [1.0, 1.0]),
    ];
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, bytemuck::NoUninit, bytemuck::Zeroable)]
pub struct CircleInstance {
    pub col: [f32; 4],
    pub centre: [f32; 2],
    pub scale: [f32; 2],
}
impl CircleInstance {
    pub const fn new(centre: [f32; 2], scale: [f32; 2], col: [f32; 4]) -> Self {
        Self { centre, scale, col }
    }
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![
            2 => Float32x4,
            3 => Float32x2,
            4 => Float32x2,
        ],
    };
}
