// use std::path::Path;

// use glium::Program;

// use crate::{core::Window, LErr};

// use super::{Uniform, UniformBuffer};

use std::{borrow::Cow, collections::HashMap, error::Error, fs::File, io::Read, path::Path};

use wgpu::{
    BindGroup, FragmentState, MultisampleState, PipelineLayoutDescriptor, PrimitiveState,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, VertexState,
};

use crate::{Binder, ToBinder, Vertex};

use super::Graphics;

///Used to tell the GPU how to draw the shapes provided.
pub struct Brush {
    compiled_shader: ShaderModule,
    cached_pipeline: Option<RenderPipeline>,
    cached_bindings: Vec<(u32, BindGroup)>,
    assets_to_bind: HashMap<u32, Binder>,
    needs_update: bool,
}

impl Brush {
    /// Create brush from shader path.
    pub fn from_path(graphics: &impl Graphics, shader_path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut source = String::new();
        File::open(shader_path)?.read_to_string(&mut source)?;
        Self::from_source(graphics, source)
    }

    /// Create brush directly providing the shader source.
    pub fn from_source(
        graphics: &impl Graphics, shader_source: String,
    ) -> Result<Self, Box<dyn Error>> {
        let device = graphics.get_device();
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_source.as_str())),
        });
        Ok(Self {
            compiled_shader: shader,
            assets_to_bind: HashMap::new(),
            cached_bindings: Vec::new(),
            cached_pipeline: None,
            needs_update: true,
        })
    }

    /// Bind asset given a group and location index. Both indices need to match with shader's or it
    /// will panic when baking and linking with rendering pipeline.
    pub fn bind(&mut self, group_index: u32, loc_index: u32, asset: impl ToBinder + 'static) {
        let asset = Box::new(asset);
        if let Some(binder) = self.assets_to_bind.get_mut(&group_index) {
            binder.bind(loc_index, asset);
        } else {
            let mut binder = Binder::new();
            binder.bind(loc_index, asset);
            self.assets_to_bind.insert(group_index, binder);
        }
        self.needs_update = true;
    }

    /// Returns if brush has been modified and needs to update the GPU with new data.
    pub fn needs_update(&self) -> bool { self.needs_update }

    /// Update GPU bindings and pipelines with current brush data.
    pub fn update(&mut self, graphics: &impl Graphics) {
        let device = graphics.get_device();
        self.cached_bindings.clear();
        let mut bgls = Vec::new();
        for (i, binder) in self.assets_to_bind.iter() {
            let (bgl, bg) = binder.bake(graphics);
            bgls.push((i, bgl));
            self.cached_bindings.push((*i, bg));
        }
        bgls.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap());
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bgls.iter().map(|x| &x.1).collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &self.compiled_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::DESC],
            },
            fragment: Some(FragmentState {
                module: &self.compiled_shader,
                entry_point: "fs_main",
                targets: &[Some(
                    graphics
                        .get_surface()
                        .get_supported_formats(graphics.get_adapter())[0]
                        .into(),
                )],
            }),
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });
        self.cached_pipeline = Some(pipeline);
        self.needs_update = false;
    }

    pub fn get_pipeline(&self) -> &RenderPipeline { self.cached_pipeline.as_ref().unwrap() }

    pub fn get_bind_groups(&self) -> &Vec<(u32, BindGroup)> { &self.cached_bindings }
}
