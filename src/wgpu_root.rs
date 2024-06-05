#![allow(dead_code)]

use std::{fs, path::Path, sync::Arc, num::NonZeroU64};

use winit::window::Window;
use image::{io::Reader as ImageReader, DynamicImage, GenericImageView};

use wgpu::*;
use bytemuck::{Pod, Zeroable};
use crate::lin_alg::Mat4;
use crate::primitives::Shape;
use crate::wgpu_text::{draw_str, RStringInputs};
use crate::primitives::Primitives;

// -- FUNCTION INPUT STRUCTS --
#[derive(Debug)]
pub enum RUniformVisibility { Vertex, Fragment, Both }
#[derive(Debug)]
pub struct RUniformSetup {
  pub bind_slot: u32,
  pub visibility: RUniformVisibility,
  pub size_in_bytes: u32,
}
#[derive(Debug)]
pub enum RCullMode { None, Front, Back }
#[derive(Debug)]
pub struct RPipelineSetup<'a> {
  pub shader: &'a str,
  pub max_obj_count: usize,
  pub texture_id: Option<usize>,
  pub cull_mode: RCullMode,
  pub vertex_fn: &'a str,
  pub fragment_fn: &'a str,
  // pub uniforms: Vec<RUniformSetup>,
}
impl Default for RPipelineSetup<'_> {
  fn default() -> Self {
      RPipelineSetup {
        shader: include_str!("base.wgsl"),
        max_obj_count: 10,
        texture_id: None,
        cull_mode: RCullMode::None,
        vertex_fn: "vertexMain",
        fragment_fn: "fragmentMain",
        // uniforms: Vec::new(),
      }
  }
}

#[derive(Debug)]
pub struct RObjectUpdate<'a> {
  pub object_id: RObjectId,
  pub translate: &'a [f32; 3],
  pub rotate_axis: &'a [f32; 3],
  pub rotate_deg: f32,
  pub scale: &'a [f32; 3],
  pub visible: bool,
  pub camera: Option<&'a RCamera>,
}
impl Default for RObjectUpdate<'_> {
  fn default() -> Self {
    RObjectUpdate {
      object_id: (0, 0),
      translate: &[0.0, 0.0, 0.0],
      rotate_axis: &[0.0, 0.0, 1.0],
      rotate_deg: 0.0,
      scale: &[1.0, 1.0, 1.0],
      visible: true,
      camera: None,
    }
  }
}
impl<'a> RObjectUpdate<'a> {
  pub fn from_shape(shape: &'a Shape, camera: Option<&'a RCamera>) -> Self {
    RObjectUpdate {
      object_id: shape.id,
      translate: &shape.position,
      rotate_axis: &shape.rotate_axis,
      rotate_deg: shape.rotate_deg,
      scale: &shape.scale,
      visible: true,
      camera,
    }
  }
}

// -- HELPER STRUCTS --
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct RVertex {
  pub position: [f32; 3],
  pub uv: [f32; 2],
  pub normal: [f32; 3],
}

#[derive(Debug)]
pub struct RObject {
  pub visible: bool,
  v_buffer: wgpu::Buffer,
  v_count: usize,
  pipe_index: usize,
}

#[derive(Debug)]
pub struct RBindGroup {
  base: wgpu::BindGroup,
  entries: Vec<wgpu::Buffer>,
}

#[derive(Debug)]
pub struct RPipeline {
  pipe: wgpu::RenderPipeline,
  objects: Vec<RObject>,
  max_obj_count: usize,
  bind_group0: RBindGroup,
  // bind_group1: Option<RBindGroup>,
  // bind_group2: Option<RBindGroup>,
  // bind_group3: Option<RBindGroup>,
}

#[derive(Debug)]
pub struct RTextPipeline {
  pipe: wgpu::RenderPipeline,
  objects: Vec<RObject>,
  max_obj_count: usize,
  bind_group0: RBindGroup,
  font_data: Vec<u8>,
  output: wgpu::Texture,
}

pub type RObjectId = (usize, usize);
pub type RPipelineId = usize;
pub type RTextureId = usize;

// helper for defining camera/view matrix
#[derive(Debug)]
pub enum CameraType {
  Orthographic,
  Perspective,
}

// helper for defining camera/view matrix
#[derive(Debug)]
pub struct RCamera {
  pub cam_type: CameraType,
  pub position: [f32; 3],
  pub look_at: [f32; 3],
  pub up: [f32; 3],
  pub fov_y: f32,
  pub near: f32,
  pub far: f32,
}
impl RCamera {
  pub fn new_ortho(near: f32, far: f32) -> Self {
    Self {
      cam_type: CameraType::Orthographic,
      position: [0.0, 0.0, 100.0],
      look_at: [0.0, 0.0, 0.0],
      up: [0.0, 1.0, 0.0],
      fov_y: 0.0,
      near,
      far,
    }
  }
  pub fn new_persp(fov_y: f32, near: f32, far: f32) -> Self {
    Self {
      cam_type: CameraType::Perspective,
      position: [0.0, 0.0, 100.0],
      look_at: [0.0, 0.0, 0.0],
      up: [0.0, 1.0, 0.0],
      fov_y,
      near,
      far,
    }
  }
}

// -- PRIMARY RENDERER INTERFACE --
#[derive(Debug)]
pub struct Renderer<'a> {
  surface: wgpu::Surface<'a>,
  surface_format: wgpu::TextureFormat,
  device: wgpu::Device,
  queue: wgpu::Queue,
  pub config: wgpu::SurfaceConfiguration,
  msaa: wgpu::Texture,
  zbuffer: wgpu::Texture,
  limits: wgpu::Limits,
  pub default_cam: RCamera,
  pub clear_color: wgpu::Color,
  pub pipelines: Vec<RPipeline>,
  pub textures: Vec<wgpu::Texture>,
  font_cache: Option<Vec<u8>>,
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

    // create default camera setup
    let default_cam = RCamera::new_ortho(0.0, 1000.0);

    return Self {
      surface,
      surface_format,
      device,
      queue,
      config,
      pipelines: Vec::new(),
      textures: Vec::new(),
      msaa,
      zbuffer,
      limits: Limits::default(),
      clear_color: Color { r: 0.01, g: 0.01, b: 0.02, a: 1.0 },
      default_cam,
      font_cache: None,
    };
  }

  pub fn resize_canvas(&mut self, width: u32, height: u32) {
    if width > 0 && height > 0 {
      self.config.width = width;
      self.config.height = height;
      self.surface.configure(&self.device, &self.config);

      let texture_size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      };

      // remake msaa texture
      let msaa = self.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("msaa-texture"),
        size: texture_size,
        sample_count: 4,
        mip_level_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: self.surface_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[]
      });
      self.msaa.destroy();
      self.msaa = msaa;

      // remake zbuffer texture
      let zbuffer = self.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zbuffer-texture"),
        size: texture_size,
        sample_count: 4,
        mip_level_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth24Plus,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[]
      });
      self.zbuffer.destroy();
      self.zbuffer = zbuffer;
    }
  }

  pub fn set_clear_color(&mut self, r: f64, g: f64, b:f64, a:f64) {
    self.clear_color.r = r;
    self.clear_color.g = g;
    self.clear_color.b = b;
    self.clear_color.a = a;
  }

  pub fn add_texture(&mut self, width: u32, height: u32, texture_path: Option<&Path>, use_device_format: bool) -> RTextureId {
    let id = self.textures.len();
    let mut texture_size = Extent3d { width, height, depth_or_array_layers: 1 };
    let mut texture_data: Option<DynamicImage> = None;

    // modify texture size/data based on file data
    if let Some(str) = texture_path {
      match ImageReader::open(str) {
        Ok(img_file) => match img_file.decode() {
          Ok(img_data) => {
            texture_size.width = img_data.dimensions().0;
            texture_size.height = img_data.dimensions().1;
            texture_data = Some(img_data);
          }
          Err(..) => {
            eprintln!("Err: Could not decode image file");
          }
        }
        Err(..) => {
          eprintln!("Err: Could not open image file");
        }
      };
    }

    // create texture
    let tex_format = if use_device_format { self.surface_format } 
    else { TextureFormat::Rgba8Unorm };
    let texture = self.device.create_texture(&TextureDescriptor {
      label: Some("input-texture"),
      size: texture_size,
      sample_count: 1,
      mip_level_count: 1,
      dimension: TextureDimension::D2,
      format: tex_format,
      usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
      view_formats: &[]
    });
    if let Some(img) = texture_data {
      // copy image into texture
      self.queue.write_texture(
        ImageCopyTexture {
          texture: &texture,
          mip_level: 0,
          origin: Origin3d::ZERO,
          aspect: TextureAspect::All,
        }, 
        &img.to_rgba8(),
        ImageDataLayout {
          offset: 0,
          bytes_per_row: Some(4 * texture_size.width),
          rows_per_image: Some(texture_size.height),
        },
        texture_size
      );
    }
    // add to cache
    self.textures.push(texture);
    id
  }

  pub fn update_texture(&mut self, texture_id: usize, texture_path: &Path) {
    let texture = &mut self.textures[texture_id];
    match ImageReader::open(texture_path) {
      Ok(img_file) => match img_file.decode() {
        Ok(img_data) => {
          // get data from image file
          let rgba8 = img_data.to_rgba8();
          let dimensions = img_data.dimensions();
          let texture_size = Extent3d { 
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1
          };
          // write to texture
          self.queue.write_texture(
            ImageCopyTexture {
              texture: &texture,
              mip_level: 0,
              origin: Origin3d::ZERO,
              aspect: TextureAspect::All,
            },
            &rgba8,
            ImageDataLayout {
              offset: 0,
              bytes_per_row: Some(4 * dimensions.0),
              rows_per_image: Some(dimensions.1),
            },
            texture_size
          );
        }
        Err(..) => {
          eprintln!("Err: Could not decode image file");
        }
      }
      Err(..) => {
        eprintln!("Err: Could not open image file");
      }
    }
  }

  pub fn update_texture_size(&mut self, texture_id: usize, pipeline_id: Option<usize>, width: u32, height: u32) {
    let old_texture = &mut self.textures[texture_id];

    // make new texture
    let texture_size = Extent3d { width, height, depth_or_array_layers: 1 };
    let new_texture = self.device.create_texture(&TextureDescriptor {
      label: Some("input-texture"),
      size: texture_size,
      sample_count: 1,
      mip_level_count: 1,
      dimension: TextureDimension::D2,
      format: old_texture.format(),
      usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
      view_formats: &[]
    });
    old_texture.destroy();
    self.textures[texture_id] = new_texture;

    // update bind group
    if let Some(p_id) = pipeline_id {
      let new_bind_id = {
        let pipeline = &self.pipelines[p_id];
        let pipe = &pipeline.pipe;
        self.add_bind_group0(pipe, pipeline.max_obj_count, Some(texture_id))
      };
      let pipeline = &mut self.pipelines[p_id];
      pipeline.bind_group0 = new_bind_id;
    }
  }

  pub fn add_pipeline(&mut self, setup: RPipelineSetup) -> RPipelineId {
    let id: usize = self.pipelines.len();

    // translate cullmode
    let cull_mode: Option<Face> = match setup.cull_mode {
      RCullMode::None => None,
      RCullMode::Back => Some(Face::Back),
      RCullMode::Front => Some(Face::Front)
    };

    // build render pipeline
    let shader_mod = self.device.create_shader_module(ShaderModuleDescriptor {
      label: Some("shader-module"),
      source: ShaderSource::Wgsl(setup.shader.into()),
    });
    let bind_group0_layout = self.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
      bind_group_layouts: &[&bind_group0_layout],
      push_constant_ranges: &[]
    });
    let pipeline = self.device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("render-pipeline"),
      layout: Some(&pipeline_layout),
      vertex: VertexState {
        module: &shader_mod,
        entry_point: setup.vertex_fn,
        buffers: &[VertexBufferLayout {
          array_stride: std::mem::size_of::<RVertex>() as BufferAddress,
          step_mode: VertexStepMode::Vertex,
          attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3],
        }],
        compilation_options: PipelineCompilationOptions::default(),
      },
      fragment: Some(FragmentState{
        module: &shader_mod,
        entry_point: setup.fragment_fn,
        targets: &[Some(ColorTargetState{
          format: self.surface_format,
          blend: Some(BlendState { 
            color: BlendComponent {
              operation: BlendOperation::Add,
              src_factor: BlendFactor::SrcAlpha,
              dst_factor: BlendFactor::OneMinusSrcAlpha
            },
            alpha: BlendComponent {
              operation: BlendOperation::Add,
              src_factor: BlendFactor::SrcAlpha,
              dst_factor: BlendFactor::OneMinusSrcAlpha
            }
          }),
          write_mask: ColorWrites::ALL
        })],
        compilation_options: PipelineCompilationOptions::default(),
      }),
      multisample: MultisampleState {
        count: 4,
        mask: !0,
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
        ..PrimitiveState::default()
      },
      multiview: None,
    });

    // build bind group
    let bind_group0: RBindGroup = self.add_bind_group0(&pipeline, setup.max_obj_count, setup.texture_id);
    // add to cache
    let pipe = RPipeline {
      pipe: pipeline,
      objects: Vec::new(),
      max_obj_count: setup.max_obj_count,
      bind_group0,
    };
    self.pipelines.push(pipe);
    id
  }

  fn add_bind_group0(&self, pipeline: &RenderPipeline, max_obj_count: usize, texture_id: Option<usize>) -> RBindGroup {
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
  #[cfg(never)] // unused method
  fn add_bind_group1(&self, pipeline: &RenderPipeline, max_obj_count: usize, uniforms: Vec<RUniformSetup>) -> RBindGroup {
    let min_stride = self.limits.min_uniform_buffer_offset_alignment;
    let mut bind_entries: Vec<Buffer> = Vec::new();
    let mut bind_desc: Vec<BindGroupEntry> = Vec::new();
    for i in 0..uniforms.len() {
      let size = min_stride * max_obj_count as u32;
      let label = "custom-uniform".to_owned() + &i.to_string();
      let entry = self.device.create_buffer(&BufferDescriptor { 
        label: Some(&label),
        size: size as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false 
      });
      bind_entries.push(entry);
    }
    for (i, u) in uniforms.iter().enumerate() {
      let desc = BindGroupEntry {
        binding: i as u32,
        resource: BindingResource::Buffer(BufferBinding {
          buffer: &bind_entries[i], offset: 0, size: NonZeroU64::new(u.size_in_bytes as u64)
        })
      };
      bind_desc.push(desc);
    }
    let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
      label: Some("bind-group-1"),
      layout: &pipeline.get_bind_group_layout(1),
      entries: &bind_desc
    });

    return RBindGroup {
      base: bind_group,
      entries: bind_entries
    }
  }

  pub fn add_text_pipeline(&mut self) -> (RTextureId, RPipelineId) {
    // build full screen texture
    let texture_id = self.add_texture(self.config.width, self.config.height, None, true);
    // build render pipeline
    let pipeline_id = self.add_pipeline(RPipelineSetup {
      shader: include_str!("text.wgsl"),
      texture_id: Some(texture_id),
      ..Default::default()
    });
    // build object
    let rect_data = Primitives::rect(2.0, 2.0, 0.0);
    let _rect = Shape::new(self, pipeline_id, rect_data);
    // output fields
    (texture_id, pipeline_id)
  }

  pub fn add_object(&mut self, pipeline_id: RPipelineId, v_data: Vec<RVertex>) -> RObjectId {
    let pipe = &mut self.pipelines[pipeline_id];
    let id = pipe.objects.len();

    // create vertex buffer
    let vlen = v_data.len();
    let v_buffer = self.device.create_buffer(&BufferDescriptor {
      label: Some("vertex-buffer"),
      size: (std::mem::size_of::<RVertex>() * vlen) as u64,
      usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
      mapped_at_creation: false
    });
    self.queue.write_buffer(&v_buffer, 0, bytemuck::cast_slice(&v_data.as_slice()));

    // save to cache
    let obj = RObject {
      visible: true,
      v_buffer,
      v_count: vlen,
      pipe_index: id
    };
    pipe.objects.push(obj);
    self.update_object(RObjectUpdate{ object_id: (pipeline_id, id), ..Default::default()});
    (pipeline_id, id)
  }

  pub fn update_object(&mut self, update: RObjectUpdate) {
    let pipe = &mut self.pipelines[update.object_id.0];
    let obj = &mut pipe.objects[update.object_id.1];
    let cam = match update.camera {
      Some(c) => c,
      None => &self.default_cam
    };

    obj.visible = update.visible;
    // model matrix
    let model_t = Mat4::translate(update.translate[0], update.translate[1], update.translate[2]);
    let model_r = Mat4::rotate(&update.rotate_axis, update.rotate_deg);
    let model_s = Mat4::scale(update.scale[0], update.scale[1], update.scale[2]);
    let model = Mat4::multiply(&model_t, &Mat4::multiply(&model_s, &model_r));
    // view matrix
    let view_t = Mat4::translate(-cam.position[0], -cam.position[1], -cam.position[2]);
    let view_r = Mat4::view_rot(&cam.position, &cam.look_at, &cam.up);
    let view = Mat4::multiply(&view_r, &view_t);
    // projection matrix
    let w2 = (self.config.width / 2) as f32;
    let h2 = (self.config.height / 2) as f32;
    let proj = match cam.cam_type {
      CameraType::Orthographic => Mat4::ortho(-w2, w2, h2, -h2, cam.near, cam.far),
      CameraType::Perspective => Mat4::perspective(cam.fov_y, w2/h2, cam.near, cam.far)
    };
    // merge together
    let mut mvp: [f32; 48] = [0.0; 48]; // 16 * 3 = 48
    for i in 0..48 {
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

  pub fn render_texture(&mut self, pipeline_ids: &[usize], target_id: usize) {
    let view = self.msaa.create_view(&TextureViewDescriptor::default());
    let tx = &self.textures[target_id];
    let target = tx.create_view(&TextureViewDescriptor::default());
    let zbuffer_view = self.zbuffer.create_view(&TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(
      &wgpu::CommandEncoderDescriptor { label: Some("render-texture-encoder") }
    );
    {
      // new context so ownership of encoder is released after pass finishes
      let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("render-pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
          view: &view,
          resolve_target: Some(&target),
          ops: Operations {
            load: LoadOp::Clear(self.clear_color),
            store: StoreOp::Store,
          },
        })],
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
          view: &zbuffer_view,
          depth_ops: Some(Operations {
            load: LoadOp::Clear(1.0),
            store: StoreOp::Store
          }),
          stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
      });
      // add objects to render
      for p_id in pipeline_ids {
        let pipeline = &self.pipelines[*p_id];
        for obj in &pipeline.objects {
          if !obj.visible { continue; }
          let stride = self.limits.min_uniform_buffer_offset_alignment * obj.pipe_index as u32;
          pass.set_pipeline(&pipeline.pipe);
          pass.set_vertex_buffer(0, obj.v_buffer.slice(..));
          pass.set_bind_group(0, &pipeline.bind_group0.base, &[stride]);
          pass.draw(0..(obj.v_count as u32), 0..1);
        }
      }
    }
    self.queue.submit(std::iter::once(encoder.finish()));
  }

  pub fn render_str_on_texture(&mut self, texture_id: usize, input: &str, size:f32, color: [u8; 3], top_left: [u32; 2]) {
    let texture = &mut self.textures[texture_id];
    // fetch font data
    if self.font_cache.is_none() {
      let f = fs::read("assets/roboto.ttf").unwrap();
      self.font_cache = Some(f);
    }
    let font_data = self.font_cache.as_ref().unwrap();
    // draw string onto existing texture
    match draw_str(RStringInputs {
      queue: &self.queue,
      texture,
      font_data,
      string: input,
      size,
      color,
      top_left,
      char_gap: 1,
    }) {
      Ok(()) => (),
      Err(e) => {
        println!("Could not draw str: {:?}", e);
      }
    };
  }

  pub fn render(&mut self, pipeline_ids: &[usize]) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_texture()?;
    let view = self.msaa.create_view(&TextureViewDescriptor::default());
    let target = output.texture.create_view(&TextureViewDescriptor::default());
    let zbuffer_view = self.zbuffer.create_view(&TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(
      &wgpu::CommandEncoderDescriptor { label: Some("render-encoder") }
    );
    {
      // new context so ownership of encoder is released after pass finishes
      let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("render-pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
          view: &view,
          resolve_target: Some(&target),
          ops: Operations {
            load: LoadOp::Clear(self.clear_color),
            store: StoreOp::Store,
          },
        })],
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
          view: &zbuffer_view,
          depth_ops: Some(Operations {
            load: LoadOp::Clear(1.0),
            store: StoreOp::Store
          }),
          stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
      });
      // add objects to render
      for p_id in pipeline_ids {
        let pipeline = &self.pipelines[*p_id];
        for obj in &pipeline.objects {
          if !obj.visible { continue; }
          let stride = self.limits.min_uniform_buffer_offset_alignment * obj.pipe_index as u32;
          pass.set_pipeline(&pipeline.pipe);
          pass.set_vertex_buffer(0, obj.v_buffer.slice(..));
          pass.set_bind_group(0, &pipeline.bind_group0.base, &[stride]);
          pass.draw(0..(obj.v_count as u32), 0..1);
        }
      }
    }

    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
  }
}