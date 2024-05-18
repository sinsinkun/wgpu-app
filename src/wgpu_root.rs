#![allow(dead_code)]

use std::sync::Arc;
use std::num::NonZeroU64;

use winit::window::Window;

use wgpu::*;
use bytemuck::{Pod, Zeroable};
use crate::lin_alg::{Mat4, Vec3, PI};

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

pub type RObjectId = usize;
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
  pub rotate_axis: [f32; 3],
  pub rotate_deg: f32,
  pub fov_y: f32,
  pub near: f32,
  pub far: f32,
}
impl RCamera {
  pub fn new_ortho(near: f32, far: f32) -> Self {
    Self {
      cam_type: CameraType::Orthographic,
      position: [0.0, 0.0, 0.0],
      rotate_axis: [0.0, 0.0, 1.0],
      rotate_deg: 0.0,
      fov_y: 0.0,
      near,
      far,
    }
  }
  pub fn new_persp(fov_y: f32, near: f32, far: f32) -> Self {
    Self {
      cam_type: CameraType::Perspective,
      position: [0.0, 0.0, 0.0],
      rotate_axis: [0.0, 0.0, 1.0],
      rotate_deg: 0.0,
      fov_y,
      near,
      far,
    }
  }
  pub fn look_at(&mut self, point: &[f32; 3]) {
    // convert look_at point to quaternion
    let (axis, rad) = Vec3::look_at(&self.position, point);
    self.rotate_axis = axis;
    self.rotate_deg = rad * 180.0 / PI;
  }
}

// -- PRIMARY RENDERER INTERFACE --
#[derive(Debug)]
pub struct Renderer<'a> {
  surface: wgpu::Surface<'a>,
  surface_format: wgpu::TextureFormat,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  msaa: wgpu::Texture,
  zbuffer: wgpu::Texture,
  limits: wgpu::Limits,
  pub default_cam: RCamera,
  pub clear_color: wgpu::Color,
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

    // create default camera setup
    let mut default_cam = RCamera::new_ortho(0.0, 1000.0);
    default_cam.position = [0.0, 0.0, -100.0];

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
      clear_color: Color { r: 0.01, g: 0.01, b: 0.02, a: 1.0 },
      default_cam
    };
  }

  pub fn resize_canvas(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);

      let texture_size = wgpu::Extent3d {
        width: new_size.width,
        height: new_size.height,
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

  pub fn _add_texture(&mut self, width: u32, height: u32, texture_data: Option<&[u8]>) -> RTextureId {
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

  pub fn _update_texture() {
    todo!()
  }

  pub fn _update_texture_size() {
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
        buffers: &[VertexBufferLayout {
          array_stride: std::mem::size_of::<RVertex>() as BufferAddress,
          step_mode: VertexStepMode::Vertex,
          attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3],
        }],
        compilation_options: PipelineCompilationOptions::default(),
      },
      fragment: Some(FragmentState{
        module: &shader_mod,
        entry_point: "fragmentMain",
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
    self.update_object(
      pipeline_id,
      id,
      &[0.0, 0.0, 0.0],
      &[0.0, 0.0, 1.0],
      0.0,
      &[1.0, 1.0, 1.0],
      true,
      None,
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
    camera: Option<&RCamera>,
  ) {
    let pipe = &mut self.pipelines[pipeline_id];
    let obj = &mut pipe.objects[object_id];
    let cam = match camera {
      Some(c) => c,
      None => &self.default_cam
    };

    obj.visible = visible;
    // model matrix
    let model_t = Mat4::translate(translate[0], translate[1], translate[2]);
    let model_r = Mat4::rotate(rotate_axis, rotate_deg);
    let model_s = Mat4::scale(scale[0], scale[1], scale[2]);
    let model = Mat4::multiply(&model_t, &Mat4::multiply(&model_r, &model_s));
    // view matrix
    let view_t = Mat4::translate(-cam.position[0], -cam.position[1], -cam.position[2]);
    let view_r = Mat4::rotate(&cam.rotate_axis, -cam.rotate_deg);
    let view = Mat4::multiply(&view_r, &view_t);
    // projection matrix
    let w2 = (self.config.width / 2) as f32;
    let h2 = (self.config.height / 2) as f32;
    let proj = match cam.cam_type {
      CameraType::Orthographic => Mat4::ortho(-w2, w2, -h2, h2, cam.near, cam.far),
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