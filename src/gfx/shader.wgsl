struct UvVertex {
    @location(0) position: vec2<f32>,
    @location(1) offset: vec2<f32>,
    @location(2) col: vec4<f32>,
    @location(3) centre: vec2<f32>,
    @location(4) scale: vec2<f32>,
}

@group(0) @binding(1) var blit_src_texture: texture_2d<f32>;
@group(0) @binding(2) var blit_src_sampler: sampler;

/*
 * BLITTING PIPELINE
 */

 struct UvVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
 }

struct UvVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn uv_vertex(in: UvVertexInput) -> UvVertexOutput {
    var out: UvVertexOutput;
    out.position = vec4(in.position, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

@fragment
fn blit_fragment(in: UvVertexOutput) -> @location(0) vec4<f32> {
    let col = textureSample(blit_src_texture, blit_src_sampler, in.uv);
    return vec4(col.rgb / 2. + 0.25, col.a);
}


@vertex
fn vertex (
    @builtin(instance_index) idx: u32,
    uv_vertex: UvVertex,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4(uv_vertex.centre + uv_vertex.scale*uv_vertex.position,0.,1.);
    out.col = uv_vertex.col;
    out.offset = uv_vertex.offset;
    out.idx = idx;
    return out;
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) col: vec4<f32>,
    @location(1) offset: vec2<f32>,
    @location(2) idx: u32,
}

@fragment
fn fragment (
    in: VertexOutput
) -> @location(0) vec4<f32> {
    if dot(in.offset,in.offset) <= 1. {
        return in.col;
    }
    discard;
}