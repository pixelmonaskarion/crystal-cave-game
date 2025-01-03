struct Params {
  image_size: vec2u,
  output_scale: f32,
}

t_input: $0,0;
s_input: $0,1;
params: $1;
outputTex: $2;

struct Flip {
  value : u32,
}
@group(3) @binding(0) var<uniform> flip : Flip;

@compute @workgroup_size(1, 1, 1)
fn main(
  // @builtin(workgroup_id) WorkGroupID : vec3u,
  @builtin(global_invocation_id) invocation_id : vec3u
) {
    var acc = vec4f(0.0);
    let steps = 1.0;
    let dist = 0;
    let dist_f = vec2f(f32(dist)/f32(params.image_size.x), f32(dist)/f32(params.image_size.y));
    let tex_coords = vec2f(f32(invocation_id.x)/f32(params.image_size.x), f32(invocation_id.y)/f32(params.image_size.y));
    for (var h = 0.0; h < 1.0; h += 1/steps) {
      for (var k = 0.0; k < 1.0; k += 1/steps) {
        acc += textureSampleLevel(t_input, s_input, tex_coords+vec2f(mix(-1*dist_f.x, dist_f.x, h), mix(-1*dist_f.y, dist_f.y, k)), 0.0)/(steps*steps);
      }
        // acc += textureSampleLevel(t_input, s_input, tex_coords+vec2f(mix(-1*dist_f.x, dist_f.x, h), 0.0), 0.0)/(steps*2.0);
    }
    
    textureStore(outputTex, vec2u(u32(f32(invocation_id.x)*params.output_scale), u32(f32(invocation_id.y)*params.output_scale)), acc);
}