// vertex shader
struct VertexInput {
    @location(0) position: vec2<f32>
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}


@group(0) @binding(0) var<uniform> bounds: vec4<f32>;


@vertex
fn time_domain_main(in: VertexInput) -> VertexOutput {    
    var out: VertexOutput;
    
    let min_value = f32(0);
    let max_value = bounds[0];

    let x = in.position[0];
    let y = 2.0 * (in.position[1] - min_value) / (max_value - min_value) - 1.0;

    out.position = vec4<f32>(x, y, 0.0, 1.0);
    return out;

}

@vertex
fn freq_domain_main(in: VertexInput) -> VertexOutput {    
    var out: VertexOutput;

    let max_amplitude = bounds[0];
    let scale_factor = 1.0 / max_amplitude;

    let slice_size = bounds[1];

    let x = -1.0 + ((2.0 * in.position[0] )/ (slice_size - f32(1)));
    let y = in.position[1] * scale_factor;

    out.position = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}



// fragment shader

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
 