use bespoke_engine::{binding::Descriptor, camera::Camera, culling::AABB, model::{Model, ToRaw}};
use bytemuck::{bytes_of, NoUninit};
use cgmath::{vec4, InnerSpace, Vector3};
use wgpu::Device;

use crate::instance::Instance;

#[repr(C)]
#[derive(NoUninit, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl Descriptor for Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

impl ToRaw for Vertex {
    fn to_raw(&self) -> Vec<u8> {
        bytes_of(self).to_vec()
    }
}

const VERTICES: [Vertex; 36] = [
    //+y
    Vertex {position: [0.0, 1.0, 0.0], normal: [0.0, 1.0, 0.0] },
    Vertex {position: [1.0, 1.0, 1.0], normal: [0.0, 1.0, 0.0] },
    Vertex {position: [1.0, 1.0, 0.0], normal: [0.0, 1.0, 0.0] },

    Vertex {position: [0.0, 1.0, 0.0], normal: [0.0, 1.0, 0.0] },
    Vertex {position: [0.0, 1.0, 1.0], normal: [0.0, 1.0, 0.0] },
    Vertex {position: [1.0, 1.0, 1.0], normal: [0.0, 1.0, 0.0] },

    //-y
    Vertex {position: [1.0, 0.0, 1.0], normal: [0.0, -1.0, 0.0] },
    Vertex {position: [0.0, 0.0, 0.0], normal: [0.0, -1.0, 0.0] },
    Vertex {position: [1.0, 0.0, 0.0], normal: [0.0, -1.0, 0.0] },

    Vertex {position: [0.0, 0.0, 1.0], normal: [0.0, -1.0, 0.0] },
    Vertex {position: [0.0, 0.0, 0.0], normal: [0.0, -1.0, 0.0] },
    Vertex {position: [1.0, 0.0, 1.0], normal: [0.0, -1.0, 0.0] },

    //+x
    Vertex {position: [1.0, 0.0, 0.0], normal: [1.0, 0.0, 0.0] },
    Vertex {position: [1.0, 1.0, 1.0], normal: [1.0, 0.0, 0.0] },
    Vertex {position: [1.0, 0.0, 1.0], normal: [1.0, 0.0, 0.0] },

    Vertex {position: [1.0, 0.0, 0.0], normal: [1.0, 0.0, 0.0] },
    Vertex {position: [1.0, 1.0, 0.0], normal: [1.0, 0.0, 0.0] },
    Vertex {position: [1.0, 1.0, 1.0], normal: [1.0, 0.0, 0.0] },

    //-x
    Vertex {position: [0.0, 0.0, 1.0], normal: [-1.0, 0.0, 0.0] },
    Vertex {position: [0.0, 1.0, 0.0], normal: [-1.0, 0.0, 0.0] },
    Vertex {position: [0.0, 0.0, 0.0], normal: [-1.0, 0.0, 0.0] },

    Vertex {position: [0.0, 0.0, 1.0], normal: [-1.0, 0.0, 0.0] },
    Vertex {position: [0.0, 1.0, 1.0], normal: [-1.0, 0.0, 0.0] },
    Vertex {position: [0.0, 1.0, 0.0], normal: [-1.0, 0.0, 0.0] },

    //+z
    Vertex {position: [1.0, 0.0, 1.0], normal: [0.0, 0.0, 1.0] },
    Vertex {position: [1.0, 1.0, 1.0], normal: [0.0, 0.0, 1.0] },
    Vertex {position: [0.0, 0.0, 1.0], normal: [0.0, 0.0, 1.0] },

    Vertex {position: [1.0, 1.0, 1.0], normal: [0.0, 0.0, 1.0] },
    Vertex {position: [0.0, 1.0, 1.0], normal: [0.0, 0.0, 1.0] },
    Vertex {position: [0.0, 0.0, 1.0], normal: [0.0, 0.0, 1.0] },

    //-z
    Vertex {position: [0.0, 0.0, 0.0], normal: [0.0, 0.0, -1.0] },
    Vertex {position: [1.0, 1.0, 0.0], normal: [0.0, 0.0, -1.0] },
    Vertex {position: [1.0, 0.0, 0.0], normal: [0.0, 0.0, -1.0] },

    Vertex {position: [0.0, 0.0, 0.0], normal: [0.0, 0.0, -1.0] },
    Vertex {position: [0.0, 1.0, 0.0], normal: [0.0, 0.0, -1.0] },
    Vertex {position: [1.0, 1.0, 0.0], normal: [0.0, 0.0, -1.0] },
];

pub fn new_cube(device: &Device, position: Vector3<f32>) -> (Model, Instance) {
    let instance = Instance {position: position.into(),  ..Default::default()};
    (Model::new_instances(VERTICES.iter().map(|vtx| Vertex {position: [vtx.position[0]*2.-1., vtx.position[1]*2.-1., vtx.position[2]*2.-1.], normal: vtx.normal}).collect(), &(0..VERTICES.len() as u32).collect::<Vec<_>>(), vec![instance.clone()], AABB { dimensions: [1.0; 3] }, device), instance)
}

pub fn in_front(device: &Device, camera: &Camera) -> (Model, Instance) {
    let camera = camera.clone();
    let middle = vec4(0.0, 0.0, 0.1, 1.0);
    let pos_w = camera.build_inverse_matrix()*middle;
    let position = pos_w.truncate()/pos_w.w;
    println!("{position:?}");
    let position = camera.eye+(position-camera.eye).normalize()*5.0;
    new_cube(device, position)
}