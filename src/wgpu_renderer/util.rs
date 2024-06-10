#![allow(dead_code)]
use crate::wgpu_renderer::{Renderer, RTextureId, RPipelineId, RObjectId, RVertex};

// helper for defining object transform data
pub struct Shape {
  pub id: RObjectId,
  pub position: [f32; 3],
  pub rotate_axis: [f32; 3],
  pub rotate_deg: f32,
  pub scale: [f32; 3],
  pub visible: bool,
  pub v_index: Option<Vec<f32>>
}
impl Shape {
  pub fn new(renderer: &mut Renderer, pipeline_id: RPipelineId, vertex_data: Vec<RVertex>, index_data: Option<Vec<u32>>) -> Self {
    let mut setup = RObjectSetup {
      pipeline_id,
      vertex_data,
      ..Default::default()
    };
    if let Some(indices) = index_data {
      setup.indices = indices;
    }
    let id = renderer.add_object(setup);
    Self {
      id,
      position: [0.0, 0.0, 0.0],
      rotate_axis: [0.0, 0.0, 1.0],
      rotate_deg: 0.0,
      scale: [1.0, 1.0, 1.0],
      visible: true,
      v_index: None
    }
  }
}

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

// helper for building new pipeline
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
  pub texture1_id: Option<RTextureId>,
  pub texture2_id: Option<RTextureId>,
  pub cull_mode: RCullMode,
  pub vertex_fn: &'a str,
  pub fragment_fn: &'a str,
  // pub uniforms: Vec<RUniformSetup>,
}
impl Default for RPipelineSetup<'_> {
  fn default() -> Self {
      RPipelineSetup {
        shader: include_str!("../embed_assets/base.wgsl"),
        max_obj_count: 10,
        texture1_id: None,
        texture2_id: None,
        cull_mode: RCullMode::None,
        vertex_fn: "vertexMain",
        fragment_fn: "fragmentMain",
        // uniforms: Vec::new(),
      }
  }
}

// helper for building new render object
#[derive(Debug)]
pub struct RObjectSetup {
  pub pipeline_id: RPipelineId,
  pub vertex_data: Vec<RVertex>,
  pub instances: u32,
  pub indices: Vec<u32>
}
impl Default for RObjectSetup {
  fn default() -> Self {
    RObjectSetup  {
      pipeline_id: RPipelineId(0),
      vertex_data: Vec::new(),
      indices: Vec::new(),
      instances: 1,
    }
  }
}

// helper for updating render object
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
      object_id: RObjectId(0, 0),
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
      visible: shape.visible,
      camera,
    }
  }
}
