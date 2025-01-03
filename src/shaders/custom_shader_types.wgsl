struct ScreenInfo {
    screen_size: vec2f,
    time: f32,
    camera: Camera,
}

struct Light {
    position: vec3f,
    color: vec3f,
}

fn mix_colors(back: vec4f, front: vec4f) -> vec4f{
    let pre_back = vec4f(back.rgb * back.a, back.a);
    let pre_front = vec4f(front.rgb * front.a, front.a);
    let final_rgb = back.rgb + (front.rgb * (1 - back.a));       
    let final_a = back.a + (front.a * (1.0 - back.a));
    return vec4f(final_rgb, final_a);

}