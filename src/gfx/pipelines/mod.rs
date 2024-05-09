use eframe::wgpu;

use crate::gfx::structs::{CircleInstance, UvVertex};

pub(super) mod blit;
pub(super) mod circles;

#[rustfmt::skip]
mod bindings {
    use eframe::wgpu;
    use wgpu::BufferBindingType::Uniform;
    
    use wgpu::SamplerBindingType::Filtering;
    use wgpu::TextureSampleType::Float;
    use wgpu::TextureViewDimension::D2;

    use crate::gfx::bindings::{buffer, sampler, texture, BindingMetadata};

    pub(super) const WHATEVER_YOU_LIKE: BindingMetadata = buffer(0, 0, Uniform);
    pub(super) const BLIT_SRC_TEXTURE: BindingMetadata = texture(0, 1, D2, Float { filterable: true });
    pub(super) const BLIT_SRC_SAMPLER: BindingMetadata = sampler(0, 2, Filtering);
}

#[derive(Default)]
struct RenderPipelineDescriptor<'a> {
    label: &'a str,
    vertex_entry_point: &'a str,
    fragment_entry_point: &'a str,
    vertex_buffers: &'a [wgpu::VertexBufferLayout<'a>],
    primitive: wgpu::PrimitiveState,
    depth_stencil: Option<wgpu::DepthStencilState>,
    multisample: wgpu::MultisampleState,
    fragment_target: Option<wgpu::ColorTargetState>,
}
impl RenderPipelineDescriptor<'_> {
    pub fn create_pipeline(
        self,
        device: &wgpu::Device,
        shader_module: &wgpu::ShaderModule,
        pipeline_layout: &wgpu::PipelineLayout,
    ) -> wgpu::RenderPipeline {
        let vertex_entry_point = match self.vertex_entry_point {
            "" => format!("{}_vertex", self.label),
            s => s.to_owned(),
        };
        let fragment_entry_point = match self.fragment_entry_point {
            "" => format!("{}_fragment", self.label),
            s => s.to_owned(),
        };

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{}_pipeline", self.label)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader_module,
                entry_point: &vertex_entry_point,
                buffers: self.vertex_buffers,
            },
            primitive: self.primitive,
            depth_stencil: self.depth_stencil,
            multisample: self.multisample,
            fragment: Some(wgpu::FragmentState {
                module: shader_module,
                entry_point: &fragment_entry_point,
                targets: &[self.fragment_target],
            }),
            multiview: None,
        })
    }
}
