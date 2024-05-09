use eframe::wgpu;

use super::*;

pipeline!(pub(in crate::gfx) struct Pipeline {
    type = wgpu::RenderPipeline;

    struct Bindings<'a> {
        something: &'a wgpu::Buffer = pub(FRAGMENT) bindings::WHATEVER_YOU_LIKE,
    }

    struct PipelineParams {
        target_format: wgpu::TextureFormat,
    }
    let pipeline_descriptor = RenderPipelineDescriptor {
        label: "circles_pipeline",
        vertex_entry_point: "vertex",
        fragment_entry_point: "fragment",
        vertex_buffers: &[UvVertex::LAYOUT, CircleInstance::LAYOUT],
        primitive: wgpu::PrimitiveState{
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },
        fragment_target: Some(wgpu::ColorTargetState::from(target_format)),
        ..Default::default()
    };
});
