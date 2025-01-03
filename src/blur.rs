use bespoke_engine::{binding::{create_layout, Binding, Uniform, UniformBinding, WgslType}, compute::ComputeShader, shader::ShaderType, texture::StorageTexture};
use bytemuck::{Pod, Zeroable};
use wgpu::{Device, Extent3d, Queue};

pub struct BlurCompute {
    shader: ComputeShader,
    pub params: BlurParams,
    params_binding: UniformBinding<BlurParams>,
    flip_binding: UniformBinding<u32>,
}

impl BlurCompute {
    pub fn new(source: &str, input_layout: &wgpu::BindGroupLayout, input_shader_type: &ShaderType, device: &Device) -> Self {
        let shader  = ComputeShader::new(
            source, 
            &[input_layout, &create_layout::<BlurParams>(device), &create_layout::<StorageTexture>(device), &create_layout::<u32>(device)], 
            vec![input_shader_type, &BlurParams::shader_type(), &StorageTexture::shader_type(), &u32::shader_type()], 
            device
        );
        let params = BlurParams {
            image_size: [0; 2],
            output_scale: 0.0,
            padding: 0.0,
        };
        let params_binding = UniformBinding::new(device, "Blur Params", params, None);
        let flip_binding = UniformBinding::new(device, "Flip Texture", 0, None);
        Self {
            shader,
            params,
            flip_binding,
            params_binding,
        }
    }

    pub fn blur(&mut self, input: &dyn Uniform, output: &dyn Uniform, input_size: Extent3d, output_scale: f32, device: &Device, queue: &Queue) {
        self.params.image_size = [input_size.width, input_size.height];
        self.params.output_scale = output_scale;
        self.params_binding.set_data(device, self.params);
        self.flip_binding.set_data(device, 0);
        let groups = [
            input_size.width,
            input_size.height,
            1,
        ];
        self.shader.run_once(vec![&input.binding(), &self.params_binding.binding, &output.binding(), &self.flip_binding.binding], groups, device, queue);
    }
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct BlurParams {
    image_size: [u32; 2],
    output_scale: f32,
    padding: f32,
}

impl WgslType for BlurParams {
    fn wgsl_name() -> String {
        "Params".into()
    }
}