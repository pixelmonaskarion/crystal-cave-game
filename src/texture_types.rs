use std::ops::Index;

use bespoke_engine::{binding::{Binding, Resource}, shader::ShaderType, surface_context::SurfaceCtx, texture::{DepthTexture, Texture}};
use wgpu::Device;

pub struct TextureLayer {
    pub diffuse: Texture,
    pub material: Texture,
    pub normal: Texture,
    pub shadows: Texture,
}

impl TextureLayer {
    pub fn new(surface_ctx: &dyn SurfaceCtx) -> Self {
        Self {
            diffuse: Texture::blank_texture(surface_ctx.device(), surface_ctx.config().width, surface_ctx.config().height, surface_ctx.config().format),
            material: Texture::blank_texture(surface_ctx.device(), surface_ctx.config().width, surface_ctx.config().height, surface_ctx.config().format),
            normal: Texture::blank_texture(surface_ctx.device(), surface_ctx.config().width, surface_ctx.config().height, surface_ctx.config().format),
            shadows: Texture::blank_texture(surface_ctx.device(), surface_ctx.config().width, surface_ctx.config().height, surface_ctx.config().format),
        }
    }
}

impl Binding for TextureLayer {
    fn layout(ty: Option<wgpu::BindingType>) -> Vec<wgpu::BindGroupLayoutEntry> {
        (0..4).map(|i| {
            vec![
                wgpu::BindGroupLayoutEntry {
                    binding: i*2,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: i*2+1,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ]
        }).collect::<Vec<Vec<_>>>().concat()
        
    }

    fn create_resources<'a>(&'a self) -> Vec<bespoke_engine::binding::Resource> {
        vec![
            Resource::Bespoke(wgpu::BindingResource::TextureView(&self.material.view)),
            Resource::Bespoke(wgpu::BindingResource::Sampler(&self.material.sampler)),
            Resource::Bespoke(wgpu::BindingResource::TextureView(&self.normal.view)),
            Resource::Bespoke(wgpu::BindingResource::Sampler(&self.normal.sampler)),
            Resource::Bespoke(wgpu::BindingResource::TextureView(&self.diffuse.view)),
            Resource::Bespoke(wgpu::BindingResource::Sampler(&self.diffuse.sampler)),
            Resource::Bespoke(wgpu::BindingResource::TextureView(&self.shadows.view)),
            Resource::Bespoke(wgpu::BindingResource::Sampler(&self.shadows.sampler)),
        ]
    }

    fn shader_type() -> ShaderType {
        ShaderType {
            var_types: vec!["".into(); 8],
            wgsl_types: vec![vec!["texture_2d<f32>".into(), "sampler".into()]; 4].concat(),
        }
    }
}

pub struct CrystalDepth {
    pub front: DepthTexture,
    pub back: DepthTexture,
}

impl CrystalDepth {
    pub fn new(surface_ctx: &dyn SurfaceCtx) -> Self {
        Self {
            front: DepthTexture::create_depth_texture(surface_ctx.device(), surface_ctx.config().width, surface_ctx.config().height, "Frontface Depth Texture"),
            back: DepthTexture::create_depth_texture(surface_ctx.device(), surface_ctx.config().width, surface_ctx.config().height, "Backface Depth Texture"),
        }
    }
}

impl Binding for CrystalDepth {
    fn layout(ty: Option<wgpu::BindingType>) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Depth,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                // This should match the filterable field of the
                // corresponding Texture entry above.
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Depth,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                // This should match the filterable field of the
                // corresponding Texture entry above.
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ]
    }

    fn create_resources<'a>(&'a self) -> Vec<bespoke_engine::binding::Resource> {
        vec![
            Resource::Bespoke(wgpu::BindingResource::TextureView(&self.front.view)),
            Resource::Bespoke(wgpu::BindingResource::Sampler(&self.front.sampler)),
            Resource::Bespoke(wgpu::BindingResource::TextureView(&self.back.view)),
            Resource::Bespoke(wgpu::BindingResource::Sampler(&self.back.sampler)),
        ]
    }

    fn shader_type() -> ShaderType {
        ShaderType {
            var_types: vec!["".into(); 4],
            wgsl_types: vec!["texture_depth_2d".into(), "sampler".into(), "texture_depth_2d".into(), "sampler".into()],
        }
    }
}

pub struct DepthCube {
    pub xp: DepthTexture,
    pub xn: DepthTexture,
    pub yp: DepthTexture,
    pub yn: DepthTexture,
    pub zp: DepthTexture,
    pub zn: DepthTexture,
}

impl DepthCube {
    pub fn new(device: &Device, width: u32, height: u32) -> Self {
        Self {
            xp: DepthTexture::create_depth_texture(device, width, height, "X Positive Depth Texture"),
            xn: DepthTexture::create_depth_texture(device, width, height, "X Negative Depth Texture"),
            yp: DepthTexture::create_depth_texture(device, width, height, "Y Positive Depth Texture"),
            yn: DepthTexture::create_depth_texture(device, width, height, "Y Negative Depth Texture"),
            zp: DepthTexture::create_depth_texture(device, width, height, "Z Positive Depth Texture"),
            zn: DepthTexture::create_depth_texture(device, width, height, "Z Negative Depth Texture"),
        }
    }
}

impl Index<usize> for DepthCube {
    type Output = DepthTexture;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.xp,
            1 => &self.xn,
            2 => &self.yp,
            3 => &self.yn,
            4 => &self.zp,
            5 => &self.zn,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl Binding for DepthCube {
    fn layout(ty: Option<wgpu::BindingType>) -> Vec<wgpu::BindGroupLayoutEntry> {
        (0..6).map(|i| {
            vec![wgpu::BindGroupLayoutEntry {
                binding: i*2,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Depth,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: i*2+1,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            }]
        }).collect::<Vec<Vec<_>>>().concat()
    }

    fn create_resources<'a>(&'a self) -> Vec<bespoke_engine::binding::Resource> {
        vec![
            Resource::Bespoke(wgpu::BindingResource::TextureView(&self.xp.view)),
            Resource::Bespoke(wgpu::BindingResource::Sampler(&self.xp.sampler)),
            Resource::Bespoke(wgpu::BindingResource::TextureView(&self.xn.view)),
            Resource::Bespoke(wgpu::BindingResource::Sampler(&self.xn.sampler)),
            Resource::Bespoke(wgpu::BindingResource::TextureView(&self.yp.view)),
            Resource::Bespoke(wgpu::BindingResource::Sampler(&self.yp.sampler)),
            Resource::Bespoke(wgpu::BindingResource::TextureView(&self.yn.view)),
            Resource::Bespoke(wgpu::BindingResource::Sampler(&self.yn.sampler)),
            Resource::Bespoke(wgpu::BindingResource::TextureView(&self.zp.view)),
            Resource::Bespoke(wgpu::BindingResource::Sampler(&self.zp.sampler)),
            Resource::Bespoke(wgpu::BindingResource::TextureView(&self.zn.view)),
            Resource::Bespoke(wgpu::BindingResource::Sampler(&self.zn.sampler)),
        ]
    }

    fn shader_type() -> ShaderType {
        ShaderType {
            var_types: vec!["".into(); 12],
            wgsl_types: vec![vec!["texture_depth_2d".into(), "sampler".into()]; 6].concat(),
        }
    }
}