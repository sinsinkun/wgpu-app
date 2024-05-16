#![allow(dead_code)]

use std::sync::Arc;
use std::num::NonZeroU64;
use std::ops::Range;

use winit::window::Window;
use winit::event::WindowEvent;
use winit::event::{ElementState, KeyEvent};

use wgpu::*;
use crate::lin_alg::Mat4;

// -- HELPER STRUCTS --
pub struct RVertex {
  pub position: [f32; 3],
  pub uv: [f32; 2],
  pub normal: [f32; 3],
}

pub struct RObject {
  pub visible: bool,
  vertex_buffer: wgpu::Buffer,
  uv_buffer: wgpu::Buffer,
  normal_buffer: wgpu::Buffer,
  pub vertex_count: usize,
  pub pipe_index: usize,
}

pub struct RBindGroup {
  base: wgpu::BindGroup,
  entries: Vec<wgpu::Buffer>,
}

pub struct RPipeline {
  pipe: wgpu::RenderPipeline,
  objects: Vec<RObject>,
  max_obj_count: usize,
  bind_group0: RBindGroup,
  // bind_group1: Option<RBindGroup>,
  // bind_group2: Option<RBindGroup>,
  // bind_group3: Option<RBindGroup>,
}

pub type RObjectId = usize;
pub type RPipelineId = usize;
pub type RTextureId = usize;

// -- PRIMARY RENDERER INTERFACE --
pub struct Renderer<'a> {
  surface: wgpu::Surface<'a>,
  surface_format: wgpu::TextureFormat,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  msaa: wgpu::Texture,
  zbuffer: wgpu::Texture,
  limits: wgpu::Limits,
  pub size: winit::dpi::PhysicalSize<u32>,
  pub pipelines: Vec<RPipeline>,
  pub textures: Vec<wgpu::Texture>
}

impl<'a> Renderer<'a> {
  // Creating some of the wgpu types requires async code
  pub async fn new(window: Arc<Window>) -> Renderer<'a> {
    let size = window.inner_size();

    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::PRIMARY,
      ..Default::default()
    });

    let surface = instance.create_surface(Arc::clone(&window)).unwrap();

    // handle for graphics card
    let adapter = instance.request_adapter(
      &wgpu::RequestAdapterOptions {
          power_preference: wgpu::PowerPreference::default(),
          compatible_surface: Some(&surface),
          force_fallback_adapter: false,
      },
    ).await.unwrap();

    // grab device & queue from adapter
    let (device, queue) = adapter.request_device(
      &wgpu::DeviceDescriptor {
        required_features: wgpu::Features::empty(),
        required_limits: { wgpu::Limits::default() },
        label: None,
      },
      None, // Trace path
    ).await.unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    // Shader code in this tutorial assumes an sRGB surface texture. Using a different
    // one will result in all the colors coming out darker. If you want to support non
    // sRGB surfaces, you'll need to account for that when drawing to the frame.
    let surface_format = surface_caps.formats.iter()
      .copied()
      .filter(|f| f.is_srgb())
      .next()
      .unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width,
      height: size.height,
      present_mode: surface_caps.present_modes[0],
      alpha_mode: surface_caps.alpha_modes[0],
      view_formats: vec![],
      desired_maximum_frame_latency: 2,
    };

    let texture_size = wgpu::Extent3d {
      width: config.width,
      height: config.height,
      depth_or_array_layers: 1,
    };

    // create msaa texture
    let msaa = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("msaa-texture"),
      size: texture_size,
      sample_count: 4,
      mip_level_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: surface_format,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[]
    });

    // create zbuffer texture
    let zbuffer = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("zbuffer-texture"),
      size: texture_size,
      sample_count: 4,
      mip_level_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Depth24Plus,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[]
    });

    return Self {
      surface,
      surface_format,
      device,
      queue,
      config,
      size,
      pipelines: Vec::new(),
      textures: Vec::new(),
      msaa,
      zbuffer,
      limits: Limits::default(),
    };
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
      // todo: remake msaa texture
      // todo: remake zbuffer texture
    }
  }

  pub fn input(&mut self, event: &WindowEvent) -> bool {
    match event {
      WindowEvent::KeyboardInput { 
        event: KeyEvent {
          logical_key: key,
          state: ElementState::Pressed,
          ..
        },
        ..
      } => {
        let debug = key.as_ref();
				println!("Pressed key: {debug:?}");
        true
      }
      #[allow(unused_variables)]
      WindowEvent::CursorMoved { device_id, position } => true,
      _ => true,
    }
  }

  pub fn update(&mut self) {
    // todo
  }

  pub fn add_texture(&mut self, width: u32, height: u32, texture_data: Option<&[u8]>) -> RTextureId {
    let id = self.textures.len();
    let texture_size = Extent3d { width, height, depth_or_array_layers: 1 };
    // create texture
    let texture = self.device.create_texture(&TextureDescriptor {
      label: Some("input-texture"),
      size: texture_size,
      sample_count: 1,
      mip_level_count: 1,
      dimension: TextureDimension::D2,
      format: TextureFormat::Rgba8Unorm,
      usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
      view_formats: &[]
    });
    if let Some(data) = texture_data {
      // copy image into texture
      self.queue.write_texture(
        ImageCopyTexture {
          texture: &texture,
          mip_level: 0,
          origin: Origin3d::ZERO,
          aspect: TextureAspect::All,
        }, 
        data,
        ImageDataLayout {
          offset: 0,
          bytes_per_row: Some(4 * width),
          rows_per_image: Some(height),
        },
        texture_size
      );
    }
    // add to cache
    self.textures.push(texture);
    id
  }

  pub fn update_texture() {
    todo!()
  }

  pub fn update_texture_size() {
    todo!()
  }

  pub fn add_pipeline(
    &mut self,
    shader: ShaderSource,
    max_obj_count: usize,
    texture_id: Option<usize>,
    cull_mode: Option<Face>,
  ) -> RPipelineId {
    let id: usize = self.pipelines.len();

    // build render pipeline
    let shader_mod = self.device.create_shader_module(ShaderModuleDescriptor {
      label: Some("shader-module"),
      source: shader,
    });
    let bind_group_layout = self.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      label: Some("bind-group-layout"),
      entries: &[
        // mvp matrix
        BindGroupLayoutEntry {
          binding: 0,
          visibility: ShaderStages::VERTEX,
          ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: true,
            min_binding_size: None,
          },
          count: None,
        },
        // texture
        BindGroupLayoutEntry {
          binding: 1,
          visibility: ShaderStages::FRAGMENT,
          ty: BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
          },
          count: None,
        },
        // texture sampler
        BindGroupLayoutEntry {
          binding: 2,
          visibility: ShaderStages::FRAGMENT,
          ty: BindingType::Sampler(SamplerBindingType::Filtering),
          count: None,
        },
      ]
    });
    let pipeline_layout = self.device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("pipeline-layout"),
      bind_group_layouts: &[&bind_group_layout],
      push_constant_ranges: &[]
    });
    let pipeline = self.device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("render-pipeline"),
      layout: Some(&pipeline_layout),
      vertex: VertexState {
        module: &shader_mod,
        entry_point: "vertexMain",
        buffers: &[
          VertexBufferLayout {
            array_stride: 12, // 4 bytes * 3
            attributes: &[VertexAttribute {
              format: VertexFormat::Float32x3,
              offset:0,
              shader_location: 0
            }],
            step_mode: VertexStepMode::Vertex
          },
          VertexBufferLayout {
            array_stride: 8, // 4 bytes * 2
            attributes: &[VertexAttribute {
              format: VertexFormat::Float32x2,
              offset:0,
              shader_location: 1
            }],
            step_mode: VertexStepMode::Vertex
          },
          VertexBufferLayout {
            array_stride: 12, // 4 bytes * 3
            attributes: &[VertexAttribute {
              format: VertexFormat::Float32x3,
              offset:0,
              shader_location: 2
            }],
            step_mode: VertexStepMode::Vertex
          },
        ],
        compilation_options: PipelineCompilationOptions::default(),
      },
      fragment: Some(FragmentState{
        module: &shader_mod,
        entry_point: "fragmentMain",
        targets: &[],
        compilation_options: PipelineCompilationOptions::default(),
      }),
      multisample: MultisampleState {
        count: 4,
        mask: 512,
        alpha_to_coverage_enabled: true,
      },
      depth_stencil: Some(DepthStencilState {
        format: TextureFormat::Depth24Plus,
        depth_write_enabled: true,
        depth_compare: CompareFunction::LessEqual,
        stencil: StencilState::default(),
        bias: DepthBiasState::default(),
      }),
      primitive: PrimitiveState {
        cull_mode,
        ..Default::default()
      },
      multiview: None,
    });

    // build bind group
    let bind_group: RBindGroup = self.add_bind_group(&pipeline, max_obj_count, texture_id);
    // add to cache
    let pipe = RPipeline {
      pipe: pipeline,
      objects: Vec::new(),
      max_obj_count,
      bind_group0: bind_group,
    };
    self.pipelines.push(pipe);
    id
  }

  fn add_bind_group(&self, pipeline: &RenderPipeline, max_obj_count: usize, texture_id: Option<usize>) -> RBindGroup {
    let min_stride = self.limits.min_uniform_buffer_offset_alignment;
    // create mvp buffer
    let mvp_buffer = self.device.create_buffer(&BufferDescriptor {
      label: Some("mvp-uniform-buffer"),
      size: min_stride as u64 * max_obj_count as u64,
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    // create texture
    let texture_view: TextureView;
    let texture_size = Extent3d {
      width: 10,
      height: 10,
      depth_or_array_layers: 1,
    };
    let ftexture = self.device.create_texture(&TextureDescriptor {
      label: Some("input-texture"),
      size: texture_size,
      sample_count: 1,
      mip_level_count: 1,
      dimension: TextureDimension::D2,
      format: TextureFormat::Rgba8Unorm,
      usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
      view_formats: &[]
    });
    if let Some(tx_id) = texture_id {
      texture_view = self.textures[tx_id].create_view(&TextureViewDescriptor::default());
    } else {
      texture_view = ftexture.create_view(&TextureViewDescriptor::default());
    }
    // create sampler
    let sampler = self.device.create_sampler(&SamplerDescriptor {
      label: Some("texture-sampler"),
      address_mode_u: AddressMode::ClampToEdge,
      address_mode_v: AddressMode::ClampToEdge,
      address_mode_w: AddressMode::ClampToEdge,
      mag_filter: FilterMode::Linear,
      min_filter: FilterMode::Nearest,
      mipmap_filter: FilterMode::Nearest,
      ..Default::default()
    });
    // create bind group
    let mvp_size = NonZeroU64::new(192); // 4 bytes * 4 rows * 4 columns * 3 matrices
    let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
      label: Some("bind-group-0"),
      layout: &pipeline.get_bind_group_layout(0),
      entries: &[
        BindGroupEntry {
          binding: 0,
          resource: BindingResource::Buffer(BufferBinding {
            buffer: &mvp_buffer, offset: 0, size: mvp_size
          })
        },
        BindGroupEntry {
          binding: 1,
          resource: BindingResource::TextureView(&texture_view)
        },
        BindGroupEntry {
          binding: 2,
          resource: BindingResource::Sampler(&sampler)
        },
      ]
    });

    return RBindGroup {
      base: bind_group,
      entries: vec![mvp_buffer]
    }
  }

  pub fn add_object(&mut self, pipeline_id: RPipelineId, v_data: &Vec<RVertex>) -> RObjectId {
    let pipe = &mut self.pipelines[pipeline_id];
    let id = pipe.objects.len();

    // create vertex buffer
    let vlen = v_data.len();
    let mut vertices: Vec<f32> = vec![0.0; vlen * 3];
    let mut uvs: Vec<f32> = vec![0.0; vlen * 2];
    let mut normals: Vec<f32> = vec![0.0; vlen * 3];
    for i in 0..vlen {
      vertices[i*3] = v_data[i].position[0];
      vertices[i*3+1] = v_data[i].position[1];
      vertices[i*3+2] = v_data[i].position[2];

      uvs[i*2] = v_data[i].uv[0];
      uvs[i*2+1] = v_data[i].uv[1];

      normals[i*3] = v_data[i].normal[0];
      normals[i*3+1] = v_data[i].normal[1];
      normals[i*3+2] = v_data[i].normal[2];
    }
    let vert_buffer = self.device.create_buffer(&BufferDescriptor { 
      label: Some("vertex-buffer"), 
      size: (vlen * 3 * 4) as u64, 
      usage: BufferUsages::VERTEX | BufferUsages::COPY_DST, 
      mapped_at_creation: false
    });
    self.queue.write_buffer(&vert_buffer, 0, bytemuck::cast_slice(&vertices));
    let uv_buffer = self.device.create_buffer(&BufferDescriptor { 
      label: Some("uv-buffer"), 
      size: (vlen * 2 * 4) as u64, 
      usage: BufferUsages::VERTEX | BufferUsages::COPY_DST, 
      mapped_at_creation: false
    });
    self.queue.write_buffer(&uv_buffer, 0, bytemuck::cast_slice(&uvs));
    let normal_buffer = self.device.create_buffer(&BufferDescriptor { 
      label: Some("normal-buffer"), 
      size: (vlen * 3 * 4) as u64, 
      usage: BufferUsages::VERTEX | BufferUsages::COPY_DST, 
      mapped_at_creation: false
    });
    self.queue.write_buffer(&normal_buffer, 0, bytemuck::cast_slice(&normals));

    // save to cache
    let obj = RObject {
      visible: true,
      vertex_buffer: vert_buffer,
      uv_buffer,
      normal_buffer,
      vertex_count: vlen,
      pipe_index: id
    };
    pipe.objects.push(obj);
    self.update_object(
      pipeline_id,
      id,
      &[0.0, 0.0, 0.0],
      &[0.0, 0.0, 1.0],
      0.0,
      &[1.0, 1.0, 1.0],
      true
    );
    id
  }

  pub fn update_object(
    &mut self,
    pipeline_id: RPipelineId,
    object_id: RObjectId,
    translate: &[f32; 3],
    rotate_axis: &[f32; 3],
    rotate_deg: f32,
    scale: &[f32; 3],
    visible: bool,
    // camera: RCamera,
  ) {
    let pipe = &mut self.pipelines[pipeline_id];
    let obj = &mut pipe.objects[object_id];

    obj.visible = visible;
    // model matrix
    let model_t = Mat4::translate(translate[0], translate[1], translate[2]);
    let model_r = Mat4::rotate(rotate_axis, rotate_deg);
    let model_s = Mat4::scale(scale[0], scale[1], scale[2]);
    let model = Mat4::multiply(&model_t, &Mat4::multiply(&model_r, &model_s));
    // view matrix
    let view = Mat4::identity();
    // projection matrix
    let w2 = (self.config.width / 2) as f32;
    let h2 = (self.config.height / 2) as f32;
    let proj = Mat4::ortho(-w2, w2, -h2, h2, 0.0, 1000.0);
    // merge together
    let mut mvp: [f32; 16 * 3] = [0.0; 16 * 3];
    for i in 0..(16 * 3) {
      if i < 16 { mvp[i] = model[i]; }
      else if i < 32 { mvp[i] = view[i - 16]; }
      else { mvp[i] = proj[i - 32]; }
    }
    let stride = self.limits.min_uniform_buffer_offset_alignment;
    self.queue.write_buffer(
      &pipe.bind_group0.entries[0], 
      (stride * obj.pipe_index as u32) as u64, 
      bytemuck::cast_slice(&mvp)
    );

  }

  pub fn render(&mut self, pipeline_ids: &[usize], target_id: Option<usize>) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_texture()?;
    let view = self.msaa.create_view(&TextureViewDescriptor::default());
    let target = match target_id {
      Some(id) => {
        let tx = &self.textures[id];
        tx.create_view(&TextureViewDescriptor::default())
      },
      None => output.texture.create_view(&TextureViewDescriptor::default())
    };
    let mut encoder = self.device.create_command_encoder(
      &wgpu::CommandEncoderDescriptor { label: Some("render-encoder") }
    );
    {
      // new context so ownership of encoder is released after pass finishes
      let mut _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("render-pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: Some(&target),
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.01, g: 0.02, b: 0.05, a: 1.0 }),
            store: wgpu::StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
      });
      // add objects to render
      for p_id in pipeline_ids {
        let pipeline = &self.pipelines[*p_id];
        for obj in &pipeline.objects {
          if !obj.visible { continue; }
          let stride = self.limits.min_uniform_buffer_offset_alignment * obj.pipe_index as u32;
          _pass.set_pipeline(&pipeline.pipe);
          _pass.set_vertex_buffer(0, obj.vertex_buffer.slice(0..));
          _pass.set_vertex_buffer(1, obj.uv_buffer.slice(0..));
          _pass.set_vertex_buffer(2, obj.normal_buffer.slice(0..));
          _pass.set_bind_group(0, &pipeline.bind_group0.base, &[stride]);
          _pass.draw(Range{ start:0, end:obj.vertex_count as u32 }, Range{ start:0, end:1 });
        }
      }
    }

    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
  }
}