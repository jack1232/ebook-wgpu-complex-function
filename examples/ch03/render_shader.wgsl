@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var texSampler: sampler;

struct Output {
    @builtin(position) position: vec4f,
    @location(0) texCoord: vec2f,
}

@vertex
fn vs_main(@builtin(vertex_index) vIndex: u32) -> Output {
    var pos = array(
        vec2(-1.0, -1.0),
        vec2( 1.0, -1.0),
        vec2( 1.0,  1.0),
        vec2( 1.0,  1.0),
        vec2(-1.0,  1.0),
        vec2(-1.0, -1.0),
    );

    var output: Output;
    output.position = vec4(pos[vIndex].x, -pos[vIndex].y, 0.0, 1.0);
    output.texCoord = pos[vIndex] * 0.5 + 0.5;
    return output;
}

@fragment
fn fs_main(in: Output) -> @location(0) vec4f {
    return textureSample(texture, texSampler, in.texCoord);
}