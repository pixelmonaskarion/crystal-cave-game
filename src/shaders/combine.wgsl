t_material: $0,0;
s_material: $0,1;
t_normal: $0,2;
s_normal: $0,3;
t_diffuse: $0,4;
s_diffuse: $0,5;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;
    return out;
}

struct FragmentOutput {
  @location(0) material: vec4f,
  @location(1) normal: vec4f,
  @location(2) diffuse: vec4f,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    out.material = textureSample(t_material, s_material, in.tex_coords);
    out.normal = textureSample(t_normal, s_normal, in.tex_coords);
    out.diffuse = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return out;
}