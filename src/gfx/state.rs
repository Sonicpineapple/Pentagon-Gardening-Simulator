use parking_lot::Mutex;
use std::{fmt, sync::Arc};

use eframe::{egui, wgpu};
use wgpu::util::DeviceExt;

use super::structs::CircleInstance;

pub(crate) struct GraphicsState {
    pub(crate) device: Arc<wgpu::Device>,
    pub(crate) queue: Arc<wgpu::Queue>,
    pub(super) circle_pipeline: super::pipelines::circles::Pipeline,
    pub(super) blit_pipeline: super::pipelines::blit::Pipeline,
    pub(super) buffer: wgpu::Buffer,
    pub(super) uv_vertex_buffer: wgpu::Buffer,
    pub(super) blit_uv_vertex_buffer: wgpu::Buffer,
    pub(super) circle_instance_buffer: Mutex<super::cache::CachedDynamicBuffer>,
    pub(super) texture: Mutex<Option<wgpu::Texture>>,
    pub(super) sampler: wgpu::Sampler,
}
impl GraphicsState {
    pub(crate) fn new(render_state: &eframe::egui_wgpu::RenderState) -> Self {
        let device = Arc::clone(&render_state.device);
        let queue = Arc::clone(&render_state.queue);
        let uv_vertex_buffer = create_buffer_init::<super::structs::UvVertex>(
            &device,
            "uv_vertices",
            &super::structs::UvVertex::SQUARE,
            wgpu::BufferUsages::VERTEX,
        );
        let blit_uv_vertex_buffer = create_buffer_init::<super::structs::UvVertex>(
            &device,
            "uv_vertices",
            &super::structs::UvVertex::BLIT_SQUARE,
            wgpu::BufferUsages::VERTEX,
        );
        let shader_module = device.create_shader_module(include_wgsl!("shader.wgsl"));
        let circle_pipeline = super::pipelines::circles::Pipeline::new(
            &device,
            &shader_module,
            super::pipelines::circles::PipelineParams {
                target_format: render_state.target_format,
            },
        );
        let blit_pipeline = super::pipelines::blit::Pipeline::new(
            &device,
            &shader_module,
            super::pipelines::blit::PipelineParams {
                target_format: render_state.target_format,
            },
        );
        let buffer = create_buffer_init(&device, "String", &[2], wgpu::BufferUsages::UNIFORM);
        let circle_instance_buffer =
            Mutex::new(super::cache::CachedDynamicBuffer::new::<CircleInstance>(
                Some("CircleInstance"),
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            ));
        let texture = Mutex::new(None);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            device,
            queue,
            circle_pipeline,
            blit_pipeline,
            buffer,
            uv_vertex_buffer,
            blit_uv_vertex_buffer,
            circle_instance_buffer,
            texture,
            sampler,
        }
    }

    pub(super) fn create_buffer_init<T: Default + bytemuck::NoUninit>(
        &self,
        label: impl fmt::Display,
        contents: &[T],
        usage: wgpu::BufferUsages,
    ) -> wgpu::Buffer {
        create_buffer_init(&self.device, label, contents, usage)
    }
    pub(super) fn create_buffer<T>(
        &self,
        label: impl fmt::Display,
        len: usize,
        usage: wgpu::BufferUsages,
    ) -> wgpu::Buffer {
        let size = std::mem::size_of::<T>() * len;
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&label.to_string()),
            size: wgpu::util::align_to(size as u64, wgpu::COPY_BUFFER_ALIGNMENT),
            usage,
            mapped_at_creation: false,
        })
    }

    pub(super) fn create_uniform_buffer<T>(&self, label: impl fmt::Display) -> wgpu::Buffer {
        self.create_buffer::<T>(
            label,
            1,
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        )
    }
}

fn create_buffer_init<T: Default + bytemuck::NoUninit>(
    device: &wgpu::Device,
    label: impl fmt::Display,
    contents: &[T],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    let mut contents = contents.to_vec();
    super::pad_buffer_to_wgpu_copy_buffer_alignment(&mut contents);

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&label.to_string()),
        contents: bytemuck::cast_slice::<T, u8>(contents.as_slice()),
        usage,
    })
}
