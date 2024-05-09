use std::sync::Arc;

use eframe::{
    egui,
    egui_wgpu::ScreenDescriptor,
    wgpu::{self, Extent3d},
};

use crate::gfx::structs::CircleInstance;

use super::{
    bindings::{BindGroups, WgpuPassExt},
    GraphicsState,
};

pub(crate) struct RenderResources {
    pub(crate) gfx: Arc<GraphicsState>,
    pub(crate) circles: Vec<CircleInstance>,
    pub(crate) texture_size: Extent3d,
    pub(crate) clear: bool,
}
impl eframe::egui_wgpu::CallbackTrait for RenderResources {
    fn prepare(
        &self,
        device: &eframe::wgpu::Device,
        _queue: &eframe::wgpu::Queue,
        _descriptor: &ScreenDescriptor,
        egui_encoder: &mut eframe::wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> std::vec::Vec<eframe::wgpu::CommandBuffer> {
        let bind_groups =
            self.gfx
                .circle_pipeline
                .bind_groups(super::pipelines::circles::Bindings {
                    something: &self.gfx.buffer,
                });

        // Send circle data to GPU
        self.gfx
            .circle_instance_buffer
            .lock()
            .write_all(&self.gfx, &mut self.circles.clone());

        // Replace texture if the size is wrong
        let mut texture = self.gfx.texture.lock();
        if let Some(t) = &*texture {
            if t.size() != self.texture_size {
                *texture = None;
            }
        }
        let texture = texture.get_or_insert_with(|| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Texture"),
                size: self.texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.gfx.target_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[
                    self.gfx.target_format.add_srgb_suffix(),
                    self.gfx.target_format.remove_srgb_suffix(),
                ],
            })
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.gfx.target_format.remove_srgb_suffix()),
            ..Default::default()
        });
        let circle_instance_buffer = self
            .gfx
            .circle_instance_buffer
            .lock()
            .at_len_at_least(&self.gfx, self.circles.len());
        let mut render_pass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: if self.clear {
                        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                    } else {
                        wgpu::LoadOp::Load
                    },
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(&self.gfx.circle_pipeline.pipeline);
        render_pass.set_bind_groups(&bind_groups);
        render_pass.set_vertex_buffer(0, self.gfx.uv_vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(
            1,
            circle_instance_buffer.slice(
                self.gfx
                    .circle_instance_buffer
                    .lock()
                    .slice_bounds(self.circles.len()),
            ),
        );
        render_pass.draw(0..4, 0..self.circles.len() as u32);

        let pipeline = &self.gfx.blit_pipeline;
        let bind_groups = pipeline.bind_groups(super::pipelines::blit::Bindings {
            src_texture: &texture_view,
            src_sampler: &self.gfx.sampler,
        });
        callback_resources.insert(bind_groups);
        vec![]
    }

    fn paint<'a>(
        &'a self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'a>,
        callback_resources: &'a eframe::egui_wgpu::CallbackResources,
    ) {
        let Some(bind_groups) = callback_resources.get::<BindGroups>() else {
            panic!("lost bind groups for blitting puzzle view");
            return;
        };
        render_pass.set_pipeline(&self.gfx.blit_pipeline.pipeline);
        render_pass.set_bind_groups(bind_groups);
        render_pass.set_vertex_buffer(0, self.gfx.blit_uv_vertex_buffer.slice(..));
        render_pass.draw(0..4, 0..1);
    }
}
