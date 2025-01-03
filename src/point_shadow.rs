use bespoke_engine::{binding::UniformBinding, camera::vec_to_point, shader::{Shader, ShaderConfig, ShaderType}, surface_context::SurfaceCtx};
use bytemuck::bytes_of;
use cgmath::{vec3, Vector3};
use wgpu::{util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, CommandEncoder, RenderPass, VertexBufferLayout};

use crate::{light::Light, texture_types::DepthCube};

pub struct PointShadowRenderer {
    pub camera_bind_group: BindGroup,
    pub camera_layout: BindGroupLayout,
    pub index_uniform: UniformBinding<u32>,
    pub shader: Shader,
}

impl PointShadowRenderer {
    pub fn new(surface_ctx: &dyn SurfaceCtx, vertex_layout: &[VertexBufferLayout]) -> Self {
        let camera_layout = 
            surface_ctx.device().create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }]
            });
        let camera_buffer =
            surface_ctx.device().create_buffer(&wgpu::BufferDescriptor {
                size: size_of::<[[f32; 4]; 4]>() as u64 *6,
                label: Some(&format!("Point Shadow Camera Buffer")),
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
        let camera_bind_group = surface_ctx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &camera_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });
        let index_uniform = UniformBinding::new(surface_ctx.device(), "Point Light Index", 0, None);
        let shader = Shader::new(include_str!("shaders/point_shadow.wgsl"), surface_ctx.device(), vec![], vec![&camera_layout, &index_uniform.layout], vec![&ShaderType::buffer_type(false, "mat4x4f".into()), &index_uniform.shader_type], vertex_layout, ShaderConfig { depth_only: true, ..Default::default() });
        Self {
            camera_bind_group,
            camera_layout,
            shader,
            index_uniform,
        }
    }

    pub fn set_light(&mut self, light: &Light, surface_ctx: &dyn SurfaceCtx) {
        let cameras: [[[f32; 4]; 4]; 6] = [[1,0,0], [-1,0,0], [0,1,0], [0,-1,0], [0,0,1], [0,0,-1]].map(|dir| {
            let up = match dir {
                [0,1,0] => vec3(0.0, 0.0, 1.0),
                [0,-1,0] => vec3(0.0, 0.0, -1.0),
                _ => vec3(0.0, 1.0, 0.0),
            };
            let view = cgmath::Matrix4::look_at_rh(vec_to_point(light.position), vec_to_point(light.position+vec3(dir[0] as f32, dir[1] as f32, dir[2] as f32)), up);
            let proj = cgmath::perspective(cgmath::Deg(90.0), 1.0, 0.1, 100.0);
            (proj * view).into()
        });
        let camera_buffer =
            surface_ctx.device().create_buffer_init(&BufferInitDescriptor {
                contents: bytes_of(&cameras),
                label: Some(&format!("Point Shadow Camera Buffer")),
                usage: wgpu::BufferUsages::STORAGE,
            });
        self.camera_bind_group = surface_ctx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &camera_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });
    }

    pub fn setup_render<'a>(&'a mut self, outputs: &DepthCube, surface_ctx: &dyn SurfaceCtx, encoder: &'a mut CommandEncoder, i: usize) -> RenderPass<'a> {
        self.index_uniform.set_data(surface_ctx.device(), i as u32);
        let depth_texture = &outputs[i];
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Point Light Render Pass"),
            color_attachments: &[],
            timestamp_writes: None,
            occlusion_query_set: None,
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
        });
        self.shader.bind(&mut render_pass);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.index_uniform.binding, &[]);
        render_pass
    }
}