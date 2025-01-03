t_depth: $0,0;
s_depth: $0,1;

light: $1;

t_depth_cube_xp: $2,0;
s_depth_cube_xp: $2,1;
t_depth_cube_xn: $2,2;
s_depth_cube_xn: $2,3;
t_depth_cube_yp: $2,4;
s_depth_cube_yp: $2,5;
t_depth_cube_yn: $2,6;
s_depth_cube_yn: $2,7;
t_depth_cube_zp: $2,8;
s_depth_cube_zp: $2,9;
t_depth_cube_zn: $2,10;
s_depth_cube_zn: $2,11;

player_camera: $3;
light_cameras: $4;

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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let screen_depth = textureSample(t_depth, s_depth, in.tex_coords.xy);
    let clip_pos = vec4(in.tex_coords.x * 2.0 - 1.0, in.tex_coords.y * -2.0 + 1.0, screen_depth, 1.0);
    let view_pos = player_camera.inverse_proj * clip_pos;
    let world_position = view_pos.xyz / view_pos.w;
    var color = vec4f(vec3f(calculate_shadow(in, world_position)), 1.0);

    return color;
}

fn calculate_shadow(in: VertexOutput, world_position: vec3f) -> f32 {
    for(var i: i32 = 0; i < 6; i++) {
        let camera = light_cameras[i];
        let camera_space_pos_w = camera * vec4f(world_position, 1.0);
        let camera_space_pos = camera_space_pos_w.xyz / camera_space_pos_w.w;
        let tex_coords = vec2f((camera_space_pos.x+1.0)/2.0, (camera_space_pos.y-1.0)/-2.0);
        if tex_coords.x >= 0.0 && tex_coords.x <= 1.0 && tex_coords.y >= 0.0 && tex_coords.y <= 1.0 && translate_depth(camera_space_pos.z) > 0.0 && translate_depth(camera_space_pos.z) < 1.0 {
            let screen_depth = textureSample(t_depth, s_depth, in.tex_coords);
            var light_depth = 0.0;
            if i == 0 {
                light_depth = textureSample(t_depth_cube_xp, s_depth_cube_xp, tex_coords);
            }
            if i == 1 {
                light_depth = textureSample(t_depth_cube_xn, s_depth_cube_xn, tex_coords);
            }
            if i == 2 {
                light_depth = textureSample(t_depth_cube_yp, s_depth_cube_yp, tex_coords);
            }
            if i == 3 {
                light_depth = textureSample(t_depth_cube_yn, s_depth_cube_yn, tex_coords);
            }
            if i == 4 {
                light_depth = textureSample(t_depth_cube_zp, s_depth_cube_zp, tex_coords);
            }
            if i == 5 {
                light_depth = textureSample(t_depth_cube_zn, s_depth_cube_zn, tex_coords);
            }
            var shadow_value = max(0.0, translate_depth(camera_space_pos.z)-translate_depth(light_depth))*100.0;
            if (shadow_value > 0.1) {
                shadow_value = 1.0;
            }
            // return f32(i)/6.0;

            // return vec4f(vec3f(translate_depth(screen_pos.z)), 1.0);
            if (translate_depth(camera_space_pos.z) >= 0.0 && translate_depth(camera_space_pos.z) < 1.0 && screen_depth != 1.0) {
                return (1.0-shadow_value);
            }
            return 1.0;

            // return translate_depth(light_depth);
            // if light_depth <= camera_space_pos.z {
            //     return light_depth;
            // } else {
            //     return 1.0;
            // }
        }
    }
    return 0.0;
}

fn translate_depth(depth: f32) -> f32 {
    let near = 0.1;
    let far = 100.0;
    let r = (2.0 * near) / (far + near - depth * (far - near));
    return r;
}