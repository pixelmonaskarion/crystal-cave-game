use bespoke_engine::{binding::{simple_layout_entry, Binding}, shader::ShaderType};
use bytemuck::{bytes_of, NoUninit};
use cgmath::Vector3;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Light {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
}

impl Light {
    pub fn new(position: Vector3<f32>, color: Vector3<f32>) -> Self {
        Self {
            position,
            color,
        }
    }

    fn to_raw(&self) -> RawLight {
        RawLight {
            position: self.position.into(),
            color: self.color.into(),
            padding1: 0.0,
            padding2: 0.0,
        }
    }
}

#[derive(NoUninit, Clone, Copy)]
#[repr(C)]
struct RawLight {
    position: [f32; 3],
    padding1: f32,
    color: [f32; 3],
    padding2: f32,
}

impl Binding for Light {
    fn create_resources<'a>(&'a self) -> Vec<bespoke_engine::binding::Resource> {
        vec![bespoke_engine::binding::Resource::Simple(bytes_of(&self.to_raw()).to_vec())]
    }

    fn layout(_ty: Option<wgpu::BindingType>) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![simple_layout_entry(0)]
    }

    fn shader_type() -> bespoke_engine::shader::ShaderType {
        ShaderType {
            var_types: vec!["<uniform>".into()],
            wgsl_types: vec!["Light".into()]
        }    
    }
}