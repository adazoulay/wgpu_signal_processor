// vertex shader
struct VertexInput {
    @location(0) position: f32, //! Instead of vec2<f32>, single vec<f32>. Use 1 / pid to plot x coordinates
    @builtin(vertex_index) vertex_idx: u32
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}


@group(0) @binding(0) var<uniform> uniform_buffer: vec4<f32>;



@vertex
fn freq_domain_main(in: VertexInput) -> VertexOutput {    
    let local_uniform = uniform_buffer;
    
    let i = f32(in.vertex_idx);
    let x = ((2.0 * i) / local_uniform[1]) - 1.0; 

    let y = (in.position / local_uniform[0]) * 1.5; 

    var out: VertexOutput;
    out.position = vec4<f32>(x , y, 0.0, 1.0);
    return out;
}


@vertex
fn time_domain_main(in: VertexInput) -> VertexOutput {    
   let local_uniform = uniform_buffer;
    
    let i = f32(in.vertex_idx);
    let x = ((2.0 * i) / local_uniform[1]) - 1.0; 

    let y = (in.position / local_uniform[0]) * 1.5; 

    var out: VertexOutput;
    out.position = vec4<f32>(x , y, 0.0, 1.0);
    return out;

}


// fragment shader
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
 






