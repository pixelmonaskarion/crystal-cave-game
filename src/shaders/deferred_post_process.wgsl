t_screen: $0,0;
s_screen: $0,1;
t_depth: $1,0;
s_depth: $1,1;
t_material: $2,0;
s_material: $2,1;
t_normal: $2,2;
s_normal: $2,3;
t_diffuse: $2,4;
s_diffuse: $2,5;
t_shadows: $2,6;
s_shadows: $2,7;

screen_info: $3;

t_backface_depth: $4,0;
s_backface_depth: $4,1;
t_frontface_depth: $4,2;
s_frontface_depth: $4,3;

light: $5;

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
    var color = textureSample(t_screen, s_screen, in.tex_coords.xy);
    let border_width = 10u;
    let tex_coords_u = vec2u(u32(in.tex_coords.x*screen_info.screen_size.x), u32(in.tex_coords.y*screen_info.screen_size.y));
    let material = textureLoad(t_material, tex_coords_u, 0);
    if material.w == 1.0 {
        let w_material = textureLoad(t_material, tex_coords_u - vec2u(border_width, 0), 0);
        let e_material = textureLoad(t_material, tex_coords_u + vec2u(border_width, 0), 0);
        let s_material = textureLoad(t_material, tex_coords_u - vec2u(0, border_width), 0);
        let n_material = textureLoad(t_material, tex_coords_u + vec2u(0, border_width), 0);
        let frontface_depth = translate_depth(textureSample(t_frontface_depth, s_frontface_depth, in.tex_coords.xy));
        let backface_depth = translate_depth(textureSample(t_backface_depth, s_backface_depth, in.tex_coords.xy));
        var diff = backface_depth-frontface_depth;
        diff *= (100-0.1);
        diff /= 2.0;
        // color = vec4f(vec3f(diff), 1.0);
        // color = vec4f(mix(vec3f(173.0/255.0, 3.0/255.0, 252.0/255.0), vec3f(186.0/255.0, 0.0/255.0, 207.0/255.0), diff), 1.0);
        color = vec4f(mix(vec3f(1.0, 0.0, 0.0), vec3f(0.0, 0.0, 1.0), diff), 1.0);

        var result = lighting_result(in, true, 0.1, 0.01, 0.25, 8.0);
        if material.w != w_material.w || material.w != e_material.w || material.w != s_material.w || material.w != n_material.w {
            result.w = max(result.w * 2.0, 0.5);
        }
        color.w -= result.w;
        color = mix_colors(color, result);
    } else if material.w == 0.0 {
        color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
        // color = vec4f(1.0);
        var result = lighting_result(in, true, 0.1, 0.01, 0.0, 1.0);
        // color.w -= result.w;
        color = vec4f(color.xyz*(result.xyz*result.a), color.a);
    }
 
    //DEBUG
    // color = textureLoad(t_material, tex_coords_u, 0);
    // color = textureSample(t_shadows, s_shadows, in.tex_coords);
    // color = textureLoad(t_screen, tex_coords_u, 0);
    // color = vec4f(vec3f(translate_depth(textureSample(t_backface_depth, s_backface_depth, in.tex_coords.xy))), 1.0);
    // color = vec4f(vec3f(translate_depth(textureSample(t_depth, s_depth, in.tex_coords))), 1.0);
    // color = textureLoad(t_depth, tex_coords_u);
    // color = vec4f(vec3f(translate_depth(textureSample(t_depth_cube_xp, s_depth_cube_xp, in.tex_coords))), 1.0);

    // var light_depth = 0.0;
    // let i = i32(screen_info.time) % 6;
    // if i == 0 {
    //     light_depth = textureSample(t_depth_cube_xp, s_depth_cube_xp, in.tex_coords);
    // }
    // if i == 1 {
    //     light_depth = textureSample(t_depth_cube_xn, s_depth_cube_xn, in.tex_coords);
    // }
    // if i == 2 {
    //     light_depth = textureSample(t_depth_cube_yp, s_depth_cube_yp, in.tex_coords);
    // }
    // if i == 3 {
    //     light_depth = textureSample(t_depth_cube_yn, s_depth_cube_yn, in.tex_coords);
    // }
    // if i == 4 {
    //     light_depth = textureSample(t_depth_cube_zp, s_depth_cube_zp, in.tex_coords);
    // }
    // if i == 5 {
    //     light_depth = textureSample(t_depth_cube_zn, s_depth_cube_zn, in.tex_coords);
    // }
    // color = vec4f(vec3f(translate_depth(light_depth)), 1.0);

    return color;
}

fn lighting_result(in: VertexOutput, point: bool, diffuse_strength: f32, ambient_strength: f32, specular_strength: f32, specular_pow: f32) -> vec4f {
    // let ambient_strength = 0.0;
    let ambient = ambient_strength;

    let tex_coords_u = vec2u(u32(in.tex_coords.x*screen_info.screen_size.x), u32(in.tex_coords.y*screen_info.screen_size.y));
    let screen_depth = textureSample(t_depth, s_depth, in.tex_coords.xy);
    let clip_pos = vec4(in.tex_coords.x * 2.0 - 1.0, in.tex_coords.y * -2.0 + 1.0, screen_depth, 1.0);
    let view_pos = screen_info.camera.inverse_proj * clip_pos;
    let world_position = view_pos.xyz / view_pos.w;

    let normal = normalize(textureSample(t_normal, s_normal, in.tex_coords.xy).xyz * 2 - vec3f(1.0));

    let light_dir = normalize(light.position - world_position);

    let light_dist = 50.0;
    let diff = max(dot(normal, light_dir), 0.0) * diffuse_strength * (1.0-distance(light.position, world_position)/light_dist);
    // let diffuse = diff * light.color;

    // let specular_strength = 0.25;
    let view_dir = normalize(screen_info.camera.position - world_position);
    let reflect_dir = reflect(-light_dir, normal);

    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), specular_pow) * specular_strength;
    // let specular = specular_strength * spec * light.color;

    let shadow = calculate_shadow(in);

    var result = (diff + spec)*shadow + ambient;
    // result = shadow + ambient;
    return vec4f(light.color, result);
}

fn calculate_shadow(in: VertexOutput) -> f32 {
    return textureSample(t_shadows, s_shadows, in.tex_coords).x;
    // for(var i: i32 = 0; i < 6; i++) {
    //     let camera = light_cameras[i];
    //     let camera_space_pos_w = camera * vec4f(world_position, 1.0);
    //     let camera_space_pos = camera_space_pos_w.xyz / camera_space_pos_w.w;
    //     let tex_coords = vec2f((camera_space_pos.x+1.0)/2.0, (camera_space_pos.y-1.0)/-2.0);
    //     if tex_coords.x >= 0.0 && tex_coords.x <= 1.0 && tex_coords.y >= 0.0 && tex_coords.y <= 1.0 && translate_depth(camera_space_pos.z) > 0.0 && translate_depth(camera_space_pos.z) < 1.0 {
    //         let screen_depth = textureSample(t_depth, s_depth, in.tex_coords);
    //         var light_depth = 0.0;
    //         if i == 0 {
    //             light_depth = textureSample(t_depth_cube_xp, s_depth_cube_xp, tex_coords);
    //         }
    //         if i == 1 {
    //             light_depth = textureSample(t_depth_cube_xn, s_depth_cube_xn, tex_coords);
    //         }
    //         if i == 2 {
    //             light_depth = textureSample(t_depth_cube_yp, s_depth_cube_yp, tex_coords);
    //         }
    //         if i == 3 {
    //             light_depth = textureSample(t_depth_cube_yn, s_depth_cube_yn, tex_coords);
    //         }
    //         if i == 4 {
    //             light_depth = textureSample(t_depth_cube_zp, s_depth_cube_zp, tex_coords);
    //         }
    //         if i == 5 {
    //             light_depth = textureSample(t_depth_cube_zn, s_depth_cube_zn, tex_coords);
    //         }
    //         var shadow_value = max(0.0, translate_depth(camera_space_pos.z)-translate_depth(light_depth))*100.0;
    //         if (shadow_value > 0.1) {
    //             shadow_value = 1.0;
    //         }
    //         // return f32(i)/6.0;

    //         // return vec4f(vec3f(translate_depth(screen_pos.z)), 1.0);
    //         if (translate_depth(camera_space_pos.z) >= 0.0 && translate_depth(camera_space_pos.z) < 1.0 && screen_depth != 1.0) {
    //             return (1.0-shadow_value);
    //         }
    //         return 1.0;

    //         // return translate_depth(light_depth);
    //         // if light_depth <= camera_space_pos.z {
    //         //     return light_depth;
    //         // } else {
    //         //     return 1.0;
    //         // }
    //     }
    // }
    // return 0.0;
}

fn translate_depth(depth: f32) -> f32 {
    let near = 0.1;
    let far = 100.0;
    let r = (2.0 * near) / (far + near - depth * (far - near));
    return r;
}