// Vertex shader

struct Camera {
    view_proj: mat4x4<f32>,
    resolution: vec2<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) I_Point0_: vec3<f32>,
    @location(1) I_Point1_: vec3<f32>,
    @builtin(vertex_index) index: u32,
};

//struct Polyline {
//    model: mat4x4<f32>,
//};

//@group(1) @binding(0)
//var<uniform> polyline: Polyline;
//
struct PolylineMaterial {
    color: vec4<f32>,
    depth_bias: f32,
    width: f32,
};
@group(1) @binding(0)
var<uniform> line_material: PolylineMaterial;


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(
    vertex: VertexInput,
    //instance: InstanceInput,
) -> VertexOutput {
    var positions: array<vec3<f32>, 6u> = array<vec3<f32>, 6u>(
        vec3<f32>(0.0, -0.5, 0.0),
        vec3<f32>(0.0, -0.5, 1.0),
        vec3<f32>(0.0, 0.5, 1.0),
        vec3<f32>(0.0, -0.5, 0.0),
        vec3<f32>(0.0, 0.5, 1.0),
        vec3<f32>(0.0, 0.5, 0.0)
    );
    let position = positions[vertex.index];

    //let clip0 = camera.view_proj * polyline.model * vec4<f32>(vertex.I_Point0_, 1.0);
    //let clip1 = camera.view_proj * polyline.model * vec4<f32>(vertex.I_Point1_, 1.0);
    let clip0 = camera.view_proj * vec4<f32>(vertex.I_Point0_, 1.0);
    let clip1 = camera.view_proj * vec4<f32>(vertex.I_Point1_, 1.0);
    let clip = mix(clip0, clip1, position.z);

    let resolution = camera.resolution;
    let screen0 = resolution * (0.5 * clip0.xy / clip0.w + 0.5);
    let screen1 = resolution * (0.5 * clip1.xy / clip1.w + 0.5);

    let xBasis = normalize(screen1 - screen0);
    let yBasis = vec2<f32>(-xBasis.y, xBasis.x);

    var line_width = line_material.width;
    var color = line_material.color;

    //#ifdef POLYLINE_PERSPECTIVE
    //line_width = line_width / clip.w;
    //if (line_width < 1.0) {
    //    color.a = color.a * line_width;
    //    line_width = 1.0;
    //}
    //#endif

    let pt0 = screen0 + line_width * (position.x * xBasis + position.y * yBasis);
    let pt1 = screen1 + line_width * (position.x * xBasis + position.y * yBasis);
    let pt = mix(pt0, pt1, position.z);

    var depth: f32 = clip.z;
    if (line_material.depth_bias >= 0.0) {
        depth = depth * (1.0 - line_material.depth_bias);
    } else {
        let epsilon = 4.88e-04;
        // depth * (clip.w / depth)^-depth_bias. So that when -depth_bias is 1.0, this is equal to clip.w
        // and when equal to 0.0, it is exactly equal to depth.
        // the epsilon is here to prevent the depth from exceeding clip.w when -depth_bias = 1.0 
        // clip.w represents the near plane in homogenous clip space in bevy, having a depth
        // of this value means nothing can be in front of this
        // The reason this uses an exponential function is that it makes it much easier for the 
        // user to chose a value that is convinient for them
        depth = depth * exp2(-line_material.depth_bias * log2(clip.w / depth - epsilon));
    }

    return VertexOutput(vec4<f32>(clip.w * ((2.0 * pt) / resolution - 1.0), depth, clip.w), color);

    //var out: VertexOutput;
    //out.color = line_material.color;
    //out.clip_position = camera.view_proj * vec4<f32>(position, 1.0);
    //return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color);
}