#![allow(dead_code)]
use super::{Renderer, RTextureId, RPipelineId, RObjectId, RVertex, RVertexAnim};

// helper for defining object transform data
pub struct Shape {
  pub id: RObjectId,
  pub position: [f32; 3],
  pub rotate_axis: [f32; 3],
  pub rotate_deg: f32,
  pub scale: [f32; 3],
  pub visible: bool,
  pub v_index: Option<Vec<f32>>,
  pub anim_transforms: Vec<[f32; 16]>,
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
      v_index: None,
      anim_transforms: Vec::new(),
    }
  }
  pub fn new_anim(renderer: &mut Renderer, pipeline_id: RPipelineId, vertex_data: Vec<RVertexAnim>, index_data: Option<Vec<u32>>) -> Self {
    let mut setup = RObjectSetup {
      pipeline_id,
      anim_vertex_data: vertex_data,
      vertex_type: RObjectSetup::VERTEX_TYPE_ANIM,
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
      v_index: None,
      anim_transforms: Vec::new(),
    }
  }
}

// helper for defining camera/view matrix
#[derive(Debug)]
pub struct RCamera {
  pub cam_type: u8,
  pub position: [f32; 3],
  pub look_at: [f32; 3],
  pub up: [f32; 3],
  pub fov_y: f32,
  pub near: f32,
  pub far: f32,
}
impl RCamera {
  pub const ORTHOGRAPHIC: u8 = 1;
  pub const PERSPECTIVE: u8 = 2;
  pub fn new_ortho(near: f32, far: f32) -> Self {
    Self {
      cam_type: RCamera::ORTHOGRAPHIC,
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
      cam_type: RCamera::PERSPECTIVE,
      position: [0.0, 0.0, 1.0],
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
pub struct RUniformSetup {
  pub bind_slot: u32,
  pub visibility: u8,
  pub size_in_bytes: u32,
}
impl RUniformSetup {
  pub const VISIBILITY_VERTEX: u8 = 1;
  pub const VISIBILITY_FRAGMENT: u8 = 2;
  pub const VISIBILITY_BOTH: u8 = 0;
}
#[derive(Debug)]
pub struct RPipelineSetup<'a> {
  pub shader: &'a str,
  pub max_obj_count: usize,
  pub texture1_id: Option<RTextureId>,
  pub texture2_id: Option<RTextureId>,
  pub cull_mode: u8,
  pub poly_mode: u8,
  pub vertex_fn: &'a str,
  pub fragment_fn: &'a str,
  pub uniforms: Vec<RUniformSetup>,
  pub vertex_type: u8,
  pub max_joints_count: u32,
}
impl Default for RPipelineSetup<'_> {
  fn default() -> Self {
      RPipelineSetup {
        shader: include_str!("../embed_assets/base.wgsl"),
        max_obj_count: 10,
        texture1_id: None,
        texture2_id: None,
        cull_mode: RPipelineSetup::CULL_MODE_NONE,
        poly_mode: RPipelineSetup::POLY_MODE_TRI,
        vertex_fn: "vertexMain",
        fragment_fn: "fragmentMain",
        uniforms: Vec::new(),
        vertex_type: RPipelineSetup::VERTEX_TYPE_STATIC,
        max_joints_count: 0,
      }
  }
}
impl RPipelineSetup<'_> {
  // cull mode constants
  pub const CULL_MODE_NONE: u8 = 0;
  pub const CULL_MODE_BACK: u8 = 1;
  pub const CULL_MODE_FRONT: u8 = 2;
  // vertex type constants
  pub const VERTEX_TYPE_STATIC: u8 = 0;
  pub const VERTEX_TYPE_ANIM: u8 = 1;
  // polygon mode constants
  pub const POLY_MODE_TRI: u8 = 0;
  pub const POLY_MODE_LINE: u8 = 1;
  pub const POLY_MODE_POINT: u8 = 2;
}

// helper for building new render object
#[derive(Debug)]
pub struct RObjectSetup {
  pub pipeline_id: RPipelineId,
  pub vertex_data: Vec<RVertex>,
  pub instances: u32,
  pub indices: Vec<u32>,
  pub vertex_type: u8,
  pub anim_vertex_data: Vec<RVertexAnim>,
}
impl Default for RObjectSetup {
  fn default() -> Self {
    RObjectSetup  {
      pipeline_id: RPipelineId(0),
      vertex_data: Vec::new(),
      indices: Vec::new(),
      instances: 1,
      anim_vertex_data: Vec::new(),
      vertex_type: RObjectSetup::VERTEX_TYPE_STATIC,
    }
  }
}
impl RObjectSetup {
  pub const VERTEX_TYPE_STATIC: u8 = 0;
  pub const VERTEX_TYPE_ANIM: u8 = 1;
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
  pub uniforms: Vec<&'a [u8]>,
  pub anim_transforms: Vec<[f32; 16]>,
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
      uniforms: Vec::new(),
      anim_transforms: Vec::new(),
    }
  }
}
impl<'a> RObjectUpdate<'a> {
  pub fn from_shape(shape: &'a Shape) -> Self {
    RObjectUpdate {
      object_id: shape.id,
      translate: &shape.position,
      rotate_axis: &shape.rotate_axis,
      rotate_deg: shape.rotate_deg,
      scale: &shape.scale,
      visible: shape.visible,
      camera: None,
      uniforms: Vec::new(),
      anim_transforms: Vec::new(),
    }
  }
  pub fn with_camera(mut self, camera: &'a RCamera) -> Self {
    self.camera = Some(camera);
    self
  }
  pub fn with_uniforms(mut self, uniforms: Vec<&'a [u8]>) -> Self {
    self.uniforms = uniforms;
    self
  }
  pub fn with_anim(mut self, transforms: Vec<[f32; 16]>) -> Self {
    self.anim_transforms = transforms;
    self
  }
}
