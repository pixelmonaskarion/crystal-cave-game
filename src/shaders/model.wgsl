t_diffuse: $0,0;
s_diffuse: $0,1;
camera: $1;
screen_info: $2;
light: $3;

//CUBE
// struct VertexInput {
//     @location(0) position: vec3<f32>,
//     @location(1) normal: vec3<f32>,
// }

//MODEL
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    // @location(1) world_position: vec3f,
    @location(1) tex_coords: vec2f,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    let rotation_matrix = mat3x3(model_matrix[0].xyz, model_matrix[1].xyz, model_matrix[2].xyz);
    out.normal = rotation_matrix*model.normal;
    out.tex_coords = model.tex_coords;
    // out.world_position = (model_matrix * vec4<f32>(model.position, 1.0)).xyz;
    return out;
}

struct FragmentOutput {
  @location(0) material: vec4f,
  @location(1) normal: vec4f,
  @location(2) color: vec4f,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    // let tex_coords_u = vec2u(u32(in.clip_position.x), u32(in.clip_position.y));
    // if in.clip_position.z <= current_depth {
    //     textureStore(material_buffer, tex_coords_u, vec4f(1.0));
    //     let texture_normal = (in.normal+vec3f(1.0))*0.5;
    //     textureStore(normal_buffer, tex_coords_u, vec4f(texture_normal, 1.0));
    // }
    out.normal = vec4f((in.normal+vec3f(1.0))*0.5, 1.0);
    out.material = vec4f(0.0);
    out.color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return out;

    //DEBUG
    // return vec4f(in.normal, 1.0);
    // return vec4f(vec3f(translate_depth(in.clip_position.z)), 1.0);

    // return vec4f(1.0, 0.0, 1.0, 1.0);
}

fn translate_depth(depth: f32) -> f32 {
    let near = 0.1;
    let far = 100.0;
    let r = (2.0 * near) / (far + near - depth * (far - near));
    return r;
}


//render twice, once with reverse triangle faces, to calculate the depth of the crystal