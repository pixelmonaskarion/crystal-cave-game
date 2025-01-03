use std::{collections::HashMap, path::Path, time::{SystemTime, UNIX_EPOCH}};

use bespoke_engine::{binding::{create_layout, simple_layout_entry, Binding, Descriptor, UniformBinding}, camera::{Camera, CameraRaw}, culling::CullingCompute, mesh::{self, MeshModel, ModelVertex}, model::{Model, Render, ToRaw}, shader::{Shader, ShaderConfig, ShaderType}, surface_context::SurfaceCtx, texture::{DepthTexture, Texture}, window::{BasicVertex, WindowConfig, WindowHandler}};
use bytemuck::{bytes_of, NoUninit};
use cgmath::{Vector2, Vector3};
use wgpu::{util::DeviceExt, Buffer, Color, Features, Limits, RenderPass};
use winit::{dpi::PhysicalPosition, event::{KeyEvent, TouchPhase}, keyboard::{KeyCode, PhysicalKey::Code}};

use crate::{cube::in_front, instance::Instance, light::Light, load_resource, point_shadow::PointShadowRenderer, texture_types::{CrystalDepth, DepthCube, TextureLayer}};

pub struct Game {
    camera_binding: UniformBinding<Camera>,
    camera: Camera,
    screen_size: [f32; 2],
    screen_info_binding: UniformBinding<ScreenInfo>,
    start_time: u128,
    keys_down: Vec<KeyCode>,
    touch_positions: HashMap<u64, PhysicalPosition<f64>>,
    moving_bc_finger: Option<u64>,
    cube: Model,
    cube_instance: Instance,
    cube_instance_buffer: Buffer,
    cube_shader: Shader,
    cube_backface_shader: Shader,
    cube_frontface_shader: Shader,
    // material_texture_binding: UniformBinding<Texture>,
    // normal_texture_binding: UniformBinding<Texture>,
    deferred_post_process_shader: Shader,
    shadows_post_process_shader: Shader,
    post_process_shader: Shader,
    combine_post_process_shader: Shader,
    // backface_depth_texture: UniformBinding<DepthTexture>,
    // frontface_depth_texture: UniformBinding<DepthTexture>,
    crystal_depth: UniformBinding<CrystalDepth>,
    depth_texture: UniformBinding<DepthTexture>,
    // backface_blur_depth_storage: UniformBinding<StorageTexture>,
    // frontface_blur_depth_storage: UniformBinding<StorageTexture>,
    light: Light,
    light_uniform: UniformBinding<Light>,
    banana_model: MeshModel,
    // blur: BlurCompute,
    culling: CullingCompute,
    cave_model: MeshModel,
    cave_shader: Shader,
    layers: Vec<UniformBinding<TextureLayer>>,
    default_layer: UniformBinding<TextureLayer>,
    point_shadows: PointShadowRenderer,
    depth_cube: UniformBinding<DepthCube>,
}

#[repr(C)]
#[derive(NoUninit, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_pos: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex {
    #[allow(dead_code)]
    pub fn pos(&self) -> Vector3<f32> {
        return Vector3::new(self.position[0], self.position[1], self.position[2]);
    }
}

impl Descriptor for Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
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


impl Game {
    pub fn new(surface_ctx: &dyn SurfaceCtx) -> Self {
        let screen_size = [surface_ctx.size().0 as f32, surface_ctx.size().1 as f32];
        let camera = Camera {
            eye: Vector3::new(1.0, 0.0, 0.0),
            aspect: screen_size[0] / screen_size[1],
            fovy: 70.0,
            znear: 0.1,
            zfar: 100.0,
            ground: 0.0,
            sky: 0.0,
        };
        let screen_info_binding = UniformBinding::new(surface_ctx.device(), "Screen Info", ScreenInfo::new(screen_size, 0.0, camera.to_raw()), None);
        let camera_binding = UniformBinding::new(surface_ctx.device(), "Camera", camera.clone(), None);
        let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let (cube, cube_instance) = in_front(surface_ctx.device(), &camera);
        let cube_instance_buffer = surface_ctx.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Cube Instance Buffer"),
                contents: &cube_instance.to_raw(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            }
        );
        // let material_buffer = Texture::blank_texture(surface_ctx.device(), surface_ctx.size().0, surface_ctx.size().1, surface_ctx.config().format);
        // let material_texture_binding = UniformBinding::new(surface_ctx.device(), "Material Storage Binding", material_buffer, None);
        // let normal_buffer = Texture::blank_texture(surface_ctx.device(), surface_ctx.size().0, surface_ctx.size().1, surface_ctx.config().format);
        // let normal_texture_binding = UniformBinding::new(surface_ctx.device(), "Normal Storage Binding", normal_buffer, None);
        let default_layer = UniformBinding::new(surface_ctx.device(), "Default Layer", TextureLayer::new(surface_ctx), None);
        let light = Light::new(Vector3::new(0.0, 5.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
        let light_uniform = UniformBinding::new(surface_ctx.device(), "Light", light, None);
        let cube_shader = Shader::new_uniform(include_str!("shaders/cube.wgsl"), surface_ctx.device(), vec![surface_ctx.config().format; 2], vec![&camera_binding, &screen_info_binding, &light_uniform], &[mesh::ModelVertex::desc() /*cube::Vertex::desc()*/, Instance::desc()], ShaderConfig::default());
        let cube_backface_shader = Shader::new_uniform(include_str!("shaders/cube.wgsl"), surface_ctx.device(), vec![surface_ctx.config().format; 2], vec![&camera_binding, &screen_info_binding, &light_uniform], &[mesh::ModelVertex::desc() /*cube::Vertex::desc()*/, Instance::desc()], ShaderConfig { face_cull: Some(wgpu::FrontFace::Cw), depth_only: true, depth_compare: wgpu::CompareFunction::Greater, ..Default::default() });
        let cube_frontface_shader = Shader::new_uniform(include_str!("shaders/cube.wgsl"), surface_ctx.device(), vec![surface_ctx.config().format; 2], vec![&camera_binding, &screen_info_binding, &light_uniform], &[mesh::ModelVertex::desc() /*cube::Vertex::desc()*/, Instance::desc()], ShaderConfig { depth_only: true, ..Default::default() });
        // let backface_depth_texture = DepthTexture::create_depth_texture(surface_ctx.device(), screen_size[0] as u32, screen_size[1] as u32, "Backface Depth Texture");
        // let backface_depth_texture = UniformBinding::new(surface_ctx.device(), "Backface Depth Texture", backface_depth_texture, None);
        // let frontface_depth_texture = DepthTexture::create_depth_texture(surface_ctx.device(), screen_size[0] as u32, screen_size[1] as u32, "Frontface Depth Texture");
        // let frontface_depth_texture = UniformBinding::new(surface_ctx.device(), "Frontface Depth Texture", frontface_depth_texture, None);
        let crystal_depth = UniformBinding::new(surface_ctx.device(), "Crystal Depth", CrystalDepth::new(surface_ctx), None);
        let depth_texture = DepthTexture::create_depth_texture(surface_ctx.device(), screen_size[0] as u32, screen_size[1] as u32, "Depth Texture");
        let depth_texture = UniformBinding::new(surface_ctx.device(), "Depth Texture", depth_texture, None);
        let mut banana_model = MeshModel::load_model(Some("Banana".into()), Path::new("res/Banana_OBJ/Banana.obj"), load_resource, surface_ctx.device(), surface_ctx.queue(), &create_layout::<Texture>(surface_ctx.device())).unwrap();
        banana_model.enable_material_binding = false;
        for model in &mut banana_model.models {
            model.update_instances(vec![cube_instance.clone()], surface_ctx.device());
        }
        
        // let backface_blur_depth_storage = UniformBinding::new(surface_ctx.device(), "Backface Blur Depth Storage", StorageTexture::from_texture(Texture::blank_texture(surface_ctx.device(), screen_size[0] as u32 / 4, screen_size[1] as u32 / 4, wgpu::TextureFormat::Rgba32Float)), None);
        // let frontface_blur_depth_storage = UniformBinding::new(surface_ctx.device(), "Frontface Blur Depth Storage", StorageTexture::from_texture(Texture::blank_texture(surface_ctx.device(), screen_size[0] as u32 / 4, screen_size[1] as u32 / 4, wgpu::TextureFormat::Rgba32Float)), None);
        // let blur = BlurCompute::new(include_str!("shaders/blur.wgsl"), &material_texture_binding.layout, &material_texture_binding.shader_type, surface_ctx.device());
        
        let post_process_shader = Shader::new_post_process(
            include_str!("shaders/post_process.wgsl"),
            surface_ctx.device(),
            surface_ctx.config().format,
            vec![&create_layout::<Texture>(surface_ctx.device())], 
            vec![&Texture::shader_type()]
        );
        let combine_post_process_shader = Shader::new_uniform(
            include_str!("shaders/combine.wgsl"),
            surface_ctx.device(),
            vec![surface_ctx.config().format; 3],
            vec![&default_layer], 
            &[BasicVertex::desc()],
            ShaderConfig { enable_depth_texture: false, ..Default::default() }
        );
        
        let culling = CullingCompute::new("struct Instance { model_matrix: mat4x4<f32> }", "model_matrix", surface_ctx.device());
        let point_shadows = PointShadowRenderer::new(surface_ctx, &[ModelVertex::desc(), Instance::desc()]);
        let depth_cube = UniformBinding::new(surface_ctx.device(), "Depth Cube", DepthCube::new(surface_ctx.device(), 500, 500), None);
        
        let deferred_post_process_shader = Shader::new_post_process(
            include_str!("shaders/deferred_post_process.wgsl"),
            surface_ctx.device(),
            surface_ctx.config().format,
            vec![&create_layout::<Texture>(surface_ctx.device()), &depth_texture.layout, &default_layer.layout, &screen_info_binding.layout, &crystal_depth.layout, &light_uniform.layout], 
            vec![&Texture::shader_type(), &depth_texture.shader_type, &default_layer.shader_type, &screen_info_binding.shader_type, &crystal_depth.shader_type, &light_uniform.shader_type]
        );

        let shadows_post_process_shader = Shader::new_post_process(
            include_str!("shaders/shadows.wgsl"),
            surface_ctx.device(),
            surface_ctx.config().format,
            vec![&depth_texture.layout, &light_uniform.layout, &depth_cube.layout, &camera_binding.layout, &point_shadows.camera_layout], 
            vec![&depth_texture.shader_type, &light_uniform.shader_type, &depth_cube.shader_type, &camera_binding.shader_type, &ShaderType::buffer_type(false, "mat4x4f".into())]
        );

        let cave_shader = Shader::new(include_str!("shaders/model.wgsl"), surface_ctx.device(), vec![surface_ctx.config().format; 3], vec![&create_layout::<Texture>(surface_ctx.device()), &camera_binding.layout, &screen_info_binding.layout, &light_uniform.layout], vec![&Texture::shader_type(), &camera_binding.shader_type, &screen_info_binding.shader_type, &light_uniform.shader_type], &[mesh::ModelVertex::desc(), Instance::desc()], ShaderConfig::default());
        let cave_model = MeshModel::load_model(Some("Cave".into()), Path::new("res/cave/valdenfer_jpg_1.obj"), load_resource, surface_ctx.device(), surface_ctx.queue(), &create_layout::<Texture>(surface_ctx.device())).unwrap();
        Self {
            camera_binding,
            camera,
            screen_size,
            screen_info_binding,
            start_time,
            keys_down: vec![],
            touch_positions: HashMap::new(),
            moving_bc_finger: None,
            cube,
            cube_instance,
            cube_instance_buffer,
            cube_shader,
            cube_backface_shader,
            cube_frontface_shader,
            // material_texture_binding,
            // normal_texture_binding,
            deferred_post_process_shader,
            shadows_post_process_shader,
            post_process_shader,
            combine_post_process_shader,
            // backface_depth_texture,
            // frontface_depth_texture,
            crystal_depth,
            depth_texture,
            light,
            light_uniform,
            banana_model,
            // frontface_blur_depth_storage,
            // backface_blur_depth_storage,
            // blur,
            culling,
            cave_model,
            cave_shader,
            layers: vec![],
            default_layer,
            point_shadows,
            depth_cube,
        }
    }
}

impl WindowHandler for Game {
    fn resize(&mut self, surface_ctx: &dyn SurfaceCtx, new_size: Vector2<u32>) {
        self.camera.aspect = new_size.x as f32 / new_size.y as f32;
        self.screen_size = [new_size.x as f32, new_size.y as f32];
        // self.backface_depth_texture.set_data(surface_ctx.device(), DepthTexture::create_depth_texture(surface_ctx.device(), self.screen_size[0] as u32, self.screen_size[1] as u32, "Back face Depth Texture"));
        self.depth_texture.set_data(surface_ctx.device(), DepthTexture::create_depth_texture(surface_ctx.device(), self.screen_size[0] as u32, self.screen_size[1] as u32, "Back face Depth Texture"));
        // self.backface_depth_texture.set_data(surface_ctx.device(), DepthTexture::create_depth_texture(surface_ctx.device(), self.screen_size[0] as u32, self.screen_size[1] as u32, "Back face Depth Texture"));
        // self.frontface_depth_texture.set_data(surface_ctx.device(), DepthTexture::create_depth_texture(surface_ctx.device(), self.screen_size[0] as u32, self.screen_size[1] as u32, "Back face Depth Texture"));
        self.crystal_depth.set_data(surface_ctx.device(), CrystalDepth::new(surface_ctx));
        self.default_layer.set_data(surface_ctx.device(), TextureLayer::new(surface_ctx));
        // self.material_texture_binding.set_data(surface_ctx.device(), Texture::blank_texture(surface_ctx.device(), self.screen_size[0] as u32, self.screen_size[1] as u32, self.material_texture_binding.value.format));
        // self.normal_texture_binding.set_data(surface_ctx.device(), Texture::blank_texture(surface_ctx.device(), self.screen_size[0] as u32, self.screen_size[1] as u32, self.normal_texture_binding.value.format));
    }

    fn render<'a: 'b, 'b>(&'a mut self, surface_ctx: &dyn SurfaceCtx, _render_pass: & mut RenderPass<'b>, delta: f64) {
        self.update(delta);
        let mut encoder = surface_ctx.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        // {
        //     let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //         label: Some("Shadow Render Pass"),
        //         color_attachments: &[],
        //         timestamp_writes: None,
        //         occlusion_query_set: None,
        //         depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
        //             view: &self.backface_depth_texture.value.view,
        //             depth_ops: Some(wgpu::Operations {
        //                 load: wgpu::LoadOp::Clear(0.0),
        //                 store: wgpu::StoreOp::Store,
        //             }),
        //             stencil_ops: None,
        //         }),
        //     });
        //     self._render(surface_ctx, &mut render_pass, true, delta);
        // }
        // let default_layer = TextureLayer::new(surface_ctx);
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Deferred Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.default_layer.value.material.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                }), Some(wgpu::RenderPassColorAttachment {
                    view: &self.default_layer.value.normal.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                }), Some(wgpu::RenderPassColorAttachment {
                    view: &self.default_layer.value.diffuse.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                timestamp_writes: None,
                occlusion_query_set: None,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.value.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
            });
            self._render(surface_ctx, &mut render_pass, false, delta);
        }
        // self.layers.push(default_layer);
        let crystal_layer = UniformBinding::new(surface_ctx.device(), "Crystal Layer", self.render_crystal(surface_ctx, delta), None);
        self.layers.push(crystal_layer);
        // self.blur.blur(&self.material_texture_binding, &self.frontface_blur_depth_storage, self.material_texture_binding.value.texture.size(), 0.25, surface_ctx.device(), surface_ctx.queue());

        self.point_shadows.set_light(&self.light, surface_ctx);

        self.cave_model.enable_material_binding = false;
        for i in 0..6 {
            let mut render_pass = self.point_shadows.setup_render(&self.depth_cube.value, surface_ctx, &mut encoder, i);
            self.cave_model.render_instances(&mut render_pass, &self.cube_instance_buffer, 0..1);
            self.banana_model.render(&mut render_pass);
        }
        self.cave_model.enable_material_binding = true;

        surface_ctx.queue().submit([encoder.finish()]);
        // self._render(surface_ctx, render_pass, false, delta);
    }

    fn config(&self) -> Option<WindowConfig> {
        Some(WindowConfig { background_color: Some(Color::BLACK), enable_post_processing: Some(true) })
    }

    fn mouse_moved(&mut self, _surface_ctx: &dyn SurfaceCtx, _mouse_pos: PhysicalPosition<f64>) {

    }
    
    fn input_event(&mut self, _surface_ctx: &dyn SurfaceCtx, input_event: &KeyEvent) {
        if let Code(code) = input_event.physical_key {
            if input_event.state.is_pressed() {
                if !self.keys_down.contains(&code) {
                    self.keys_down.push(code);
                }
            } else {
                if let Some(i) = self.keys_down.iter().position(|x| x == &code) {
                    self.keys_down.remove(i);
                }
            }
        }
    }
    
    fn mouse_motion(&mut self, _surface_ctx: &dyn SurfaceCtx, delta: (f64, f64)) {
        self.camera.ground += (delta.0 / 500.0) as f32;
        self.camera.sky -= (delta.1 / 500.0) as f32;
        self.camera.sky = self.camera.sky.clamp(std::f32::consts::PI*-0.499, std::f32::consts::PI*0.499);
    }
    
    fn touch(&mut self, surface_ctx: &dyn SurfaceCtx, touch: &winit::event::Touch) {
        match touch.phase {
            TouchPhase::Moved => {
                if let Some(last_position) = self.touch_positions.get(&touch.id) {
                    let delta = (touch.location.x-last_position.x, touch.location.y-last_position.y);
                    self.mouse_motion(surface_ctx, delta);
                    self.touch_positions.insert(touch.id, touch.location);
                }
            }
            TouchPhase::Started => {
                if touch.location.x <= self.screen_size[0] as f64 / 2.0 {
                    self.touch_positions.insert(touch.id, touch.location);
                } else {
                    self.moving_bc_finger = Some(touch.id);
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                self.touch_positions.remove(&touch.id);
                if self.moving_bc_finger == Some(touch.id) {
                    self.moving_bc_finger = None;
                }
            }
        }
    }
    
    fn post_process_render<'a: 'b, 'c: 'b, 'b>(&'a mut self, surface_ctx: &'c dyn SurfaceCtx, render_pass: & mut RenderPass<'b>, surface_texture: &'c UniformBinding<Texture>) {
        // self.blur.blur(&self.backface_depth_texture, &self.backface_blur_depth_storage, self.backface_depth_texture.value.texture.size(), surface_context.device(), surface_context.queue());
        let combined_layer = UniformBinding::new(surface_ctx.device(), "Combined Value", TextureLayer::new(surface_ctx), None);
        let mut encoder = surface_ctx.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut layers = vec![];
            std::mem::swap(&mut layers, &mut self.layers);
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Combine Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &combined_layer.value.material.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                }), Some(wgpu::RenderPassColorAttachment {
                    view: &combined_layer.value.normal.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                }), Some(wgpu::RenderPassColorAttachment {
                    view: &combined_layer.value.diffuse.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                timestamp_writes: None,
                occlusion_query_set: None,
                depth_stencil_attachment: None,
            });
            
            render_pass.set_pipeline(&self.combine_post_process_shader.pipeline);
            render_pass.set_bind_group(0, &self.default_layer.binding, &[]);
            // render_pass.set_bind_group(0, &self.default_layer.binding, &[]);
            // render_pass.set_bind_group(1, &self.default_layer.v.normal.binding, &[]);
            // render_pass.set_bind_group(2, &self.default_layer.diffuse.binding, &[]);
            surface_ctx.screen_model().render(&mut render_pass);
            for layer in layers {
                render_pass.set_bind_group(0, &layer.binding, &[]);
                // render_pass.set_bind_group(1, &layer.normal.binding, &[]);
                // render_pass.set_bind_group(2, &layer.diffuse.binding, &[]);
                surface_ctx.screen_model().render(&mut render_pass);
            }
        }
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shadows Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &combined_layer.value.shadows.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                timestamp_writes: None,
                occlusion_query_set: None,
                depth_stencil_attachment: None,
            });
            self.shadows_post_process_shader.bind(&mut render_pass);
            render_pass.set_bind_group(0, &self.depth_texture.binding, &[]);
            render_pass.set_bind_group(1, &self.light_uniform.binding, &[]);
            render_pass.set_bind_group(2, &self.depth_cube.binding, &[]);
            render_pass.set_bind_group(3, &self.camera_binding.binding, &[]);
            render_pass.set_bind_group(4, &self.point_shadows.camera_bind_group, &[]);
            surface_ctx.screen_model().render(&mut render_pass);
        }
        surface_ctx.queue().submit([encoder.finish()]);

        render_pass.set_pipeline(&self.deferred_post_process_shader.pipeline);
        render_pass.set_bind_group(0, &surface_texture.binding, &[]);
        render_pass.set_bind_group(1, &self.depth_texture.binding, &[]);
        render_pass.set_bind_group(2, &combined_layer.binding, &[]);
        render_pass.set_bind_group(3, &self.screen_info_binding.binding, &[]);
        render_pass.set_bind_group(4, &self.crystal_depth.binding, &[]);
        
        // render_pass.set_bind_group(6, &self.frontface_depth_texture.binding, &[]);
        // render_pass.set_bind_group(6, &combined_layer.normal.binding, &[]);
        render_pass.set_bind_group(5, &self.light_uniform.binding, &[]);
        // render_pass.set_bind_group(6, &self.depth_cube.binding, &[]);
        // render_pass.set_bind_group(7, &self.point_shadows.camera_bind_group, &[]);
        
        surface_ctx.screen_model().render(render_pass);
    }
    
    fn limits() -> wgpu::Limits {
        Limits {
            max_bind_groups: 7,
            max_texture_dimension_2d: 8976,
            ..Default::default()
        }
    }
    
    fn other_window_event(&mut self, _surface_context: &dyn SurfaceCtx, _event: &winit::event::WindowEvent) {
        
    }

    fn required_features() -> wgpu::Features {
        Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
    }

    fn surface_config() -> Option<bespoke_engine::window::SurfaceConfig> {
        None
    }

    fn custom_shader_type_source() -> String {
        include_str!("shaders/custom_shader_types.wgsl").into()
    }
}

impl Game {
    fn update(&mut self, delta: f64) {
        let speed = 0.005 * delta as f32;
        if self.keys_down.contains(&KeyCode::KeyW) || self.moving_bc_finger.is_some() {
            self.camera.eye += self.camera.get_walking_vec() * speed;
        }
        if self.moving_bc_finger.is_some() {
            self.camera.eye += self.camera.get_forward_vec() * speed;
        }
        if self.keys_down.contains(&KeyCode::KeyS) {
            self.camera.eye -= self.camera.get_walking_vec() * speed;
        }
        if self.keys_down.contains(&KeyCode::KeyA) {
            self.camera.eye -= self.camera.get_right_vec() * speed;
        }
        if self.keys_down.contains(&KeyCode::KeyD) {
            self.camera.eye += self.camera.get_right_vec() * speed;
        }
        if self.keys_down.contains(&KeyCode::Space) {
            self.camera.eye += Vector3::unit_y() * speed;
        }
        if self.keys_down.contains(&KeyCode::ShiftLeft) {
            self.camera.eye -= Vector3::unit_y() * speed;
        }
        if self.keys_down.contains(&KeyCode::Tab) {
            self.light.position = self.camera.eye;
        }
    }

    fn _render<'a: 'b, 'b>(&'a mut self, surface_ctx: &dyn SurfaceCtx, render_pass: & mut RenderPass<'b>, backface: bool, _delta: f64) {
        // self.material_storage_binding.set_data(surface_ctx.device(), StorageTexture::from_texture(Texture::blank_texture(surface_ctx.device(), self.screen_size[0] as u32, self.screen_size[1] as u32, TextureFormat::Rgba32Float)));
        // self.normal_storage_binding.set_data(surface_ctx.device(), StorageTexture::from_texture(Texture::blank_texture(surface_ctx.device(), self.screen_size[0] as u32, self.screen_size[1] as u32, TextureFormat::Rgba32Float)));
        self.camera_binding.set_data(&surface_ctx.device(), self.camera.clone());
        let time = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()-self.start_time) as f32 / 1000.0;
        self.screen_info_binding.set_data(&surface_ctx.device(), ScreenInfo::new(self.screen_size, time, self.camera.to_raw()));
        self.light_uniform.set_data(surface_ctx.device(), self.light);

        // self.cube = in_front(&surface_ctx.device(), &self.camera);
        // if backface {
        //     render_pass.set_pipeline(&self.cube_backface_shader.pipeline);
        // } else {
        //     render_pass.set_pipeline(&self.cube_shader.pipeline);
        // }

        // render_pass.set_bind_group(0, &self.camera_binding.binding, &[]);
        // render_pass.set_bind_group(1, &self.screen_info_binding.binding, &[]);
        // render_pass.set_bind_group(2, &self.light_uniform.binding, &[]);
        
        // // self.cube.render(render_pass);
        // self.banana_model.render_culled(&self.camera_binding, render_pass, &mut self.culling, surface_ctx);

        render_pass.set_pipeline(&self.cave_shader.pipeline);
        render_pass.set_bind_group(1, &self.camera_binding.binding, &[]);
        render_pass.set_bind_group(2, &self.screen_info_binding.binding, &[]);
        render_pass.set_bind_group(3, &self.light_uniform.binding, &[]);
        self.cave_model.render_instances(render_pass, &self.cube_instance_buffer, 0..1);
    }

    fn render_crystal(&mut self, surface_ctx: &dyn SurfaceCtx, delta: f64) -> TextureLayer {
        // let backface_depth_texture = DepthTexture::create_depth_texture(surface_ctx.device(), self.screen_size[0] as u32, self.screen_size[1] as u32, "Backface Depth Texture");
        // let backface_depth_texture = UniformBinding::new(surface_ctx.device(), "Backface Depth Texture", backface_depth_texture, None);
        // let frontface_depth_texture = DepthTexture::create_depth_texture(surface_ctx.device(), self.screen_size[0] as u32, self.screen_size[1] as u32, "Frontface Depth Texture");
        // let frontface_depth_texture = UniformBinding::new(surface_ctx.device(), "Frontface Depth Texture", frontface_depth_texture, None);
        let mut encoder = surface_ctx.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Crystal Back Shadow Render Pass"),
                color_attachments: &[],
                timestamp_writes: None,
                occlusion_query_set: None,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.crystal_depth.value.back.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&self.cube_backface_shader.pipeline);
            self.render_crystal_inner(surface_ctx, &mut render_pass, true, delta);
        }
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Crystal Front Shadow Render Pass"),
                color_attachments: &[],
                timestamp_writes: None,
                occlusion_query_set: None,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.crystal_depth.value.front.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&self.cube_frontface_shader.pipeline);
            self.render_crystal_inner(surface_ctx, &mut render_pass, false, delta);
        }
        let crystal_layer = TextureLayer::new(surface_ctx);
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Crystal Deferred Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &crystal_layer.material.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                }), Some(wgpu::RenderPassColorAttachment {
                    view: &crystal_layer.normal.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                timestamp_writes: None,
                occlusion_query_set: None,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.value.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&self.cube_shader.pipeline);
            self.render_crystal_inner(surface_ctx, &mut render_pass, false, delta);
        }
        // let post_process_texture = Texture::blank_texture(surface_ctx.device(), surface_ctx.config().width, surface_ctx.config().height, surface_ctx.config().format);
        // let surface_texture = Texture::blank_texture(surface_ctx.device(), surface_ctx.config().width, surface_ctx.config().height, surface_ctx.config().format);
        // let surface_texture = UniformBinding::new(surface_ctx.device(), "Crystal Surface Texture", surface_texture, None);
        // {
        //     let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //         label: Some("Crystal Post Processing Render Pass"),
        //         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //             view: &post_process_texture.view,
        //             resolve_target: None,
        //             ops: wgpu::Operations {
        //                 load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
        //                 store: wgpu::StoreOp::Store,
        //             },
        //         })],
        //         timestamp_writes: None,
        //         occlusion_query_set: None,
        //         depth_stencil_attachment: None,
        //     });

        //     render_pass.set_pipeline(&self.deferred_post_process_shader.pipeline);
        //     render_pass.set_bind_group(0, &surface_texture.binding, &[]);
        //     render_pass.set_bind_group(1, &self.depth_texture.binding, &[]);
        //     render_pass.set_bind_group(2, &self.material_texture_binding.binding, &[]);
        //     render_pass.set_bind_group(3, &self.camera_binding.binding, &[]);
        //     render_pass.set_bind_group(4, &self.screen_info_binding.binding, &[]);
        //     render_pass.set_bind_group(5, &backface_depth_texture.binding, &[]);
        //     render_pass.set_bind_group(6, &self.normal_texture_binding.binding, &[]);
        //     render_pass.set_bind_group(7, &self.light_uniform.binding, &[]);
            
        //     surface_ctx.screen_model().render(&mut render_pass);
        // }
        surface_ctx.queue().submit([encoder.finish()]);
        // post_process_texture
        crystal_layer
    }

    fn render_crystal_inner<'a: 'b, 'b>(&'a mut self, surface_ctx: &dyn SurfaceCtx, render_pass: & mut RenderPass<'b>, backface: bool, _delta: f64) {
        render_pass.set_bind_group(0, &self.camera_binding.binding, &[]);
        render_pass.set_bind_group(1, &self.screen_info_binding.binding, &[]);
        render_pass.set_bind_group(2, &self.light_uniform.binding, &[]);
        
        // self.cube.render(render_pass);
        self.banana_model.render(render_pass);
        // self.banana_model.render_culled(&self.camera_binding, render_pass, &mut self.culling, surface_ctx);
    }
}

#[derive(NoUninit, Clone, Copy)]
#[repr(C)]
pub struct ScreenInfo {
    screen_size: [f32; 2],
    time: f32,
    padding: f32,
    camera_raw: CameraRaw,
}

impl ScreenInfo {
    pub fn new(screen_size: [f32; 2], time: f32, camera_raw: CameraRaw) -> Self {
        Self {
            screen_size,
            time,
            padding: 0.0,
            camera_raw,
        }
    }
}

impl Binding for ScreenInfo {
    fn create_resources<'a>(&'a self) -> Vec<bespoke_engine::binding::Resource> {
        vec![bespoke_engine::binding::Resource::Simple(bytes_of(self).to_vec())]
    }

    fn layout(_ty: Option<wgpu::BindingType>) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![simple_layout_entry(0)]
    }

    fn shader_type() -> bespoke_engine::shader::ShaderType {
        ShaderType {
            var_types: vec!["<uniform>".into()],
            wgsl_types: vec!["ScreenInfo".into()]
        }    
    }
}