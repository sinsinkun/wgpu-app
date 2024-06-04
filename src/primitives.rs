#![allow(dead_code)]

use crate::wgpu_root::{Renderer, RVertex};
use crate::lin_alg::PI;

pub struct Shape {
  pub id: (usize, usize),
  pub position: [f32; 3],
  pub rotate_axis: [f32; 3],
  pub rotate_deg: f32,
  pub scale: [f32; 3],
  pub v_index: Option<Vec<f32>>
}
impl Shape {
  pub fn new(renderer: &mut Renderer, pipe_id: usize, vertex_data: Vec<RVertex>) -> Self {
    let id = renderer.add_object(pipe_id, vertex_data);
    Self {
      id,
      position: [0.0, 0.0, 0.0],
      rotate_axis: [0.0, 0.0, 1.0],
      rotate_deg: 0.0,
      scale: [1.0, 1.0, 1.0],
      v_index: None
    }
  }
}

// note: uv_y is inverted
pub struct Primitives;
impl Primitives {
  pub fn cube(width: f32, height: f32, depth: f32) -> Vec<RVertex> {
    let w = width /2.0;
    let h = height / 2.0;
    let d = depth / 2.0;
    vec![
      // face top
      RVertex { position: [ w,-h, d], uv: [1.0,0.0], normal: [0.0,1.0,0.0] },
      RVertex { position: [ w,-h,-d], uv: [1.0,1.0], normal: [0.0,1.0,0.0] },
      RVertex { position: [-w,-h,-d], uv: [0.0,1.0], normal: [0.0,1.0,0.0] },
      RVertex { position: [-w,-h,-d], uv: [0.0,1.0], normal: [0.0,1.0,0.0] },
      RVertex { position: [-w,-h, d], uv: [0.0,0.0], normal: [0.0,1.0,0.0] },
      RVertex { position: [ w,-h, d], uv: [1.0,0.0], normal: [0.0,1.0,0.0] },
      // face bottom
      RVertex { position: [ w, h,-d], uv: [1.0,0.0], normal: [0.0,-1.0,0.0] },
      RVertex { position: [ w, h, d], uv: [1.0,1.0], normal: [0.0,-1.0,0.0] },
      RVertex { position: [-w, h, d], uv: [0.0,1.0], normal: [0.0,-1.0,0.0] },
      RVertex { position: [-w, h, d], uv: [0.0,1.0], normal: [0.0,-1.0,0.0] },
      RVertex { position: [-w, h,-d], uv: [0.0,0.0], normal: [0.0,-1.0,0.0] },
      RVertex { position: [ w, h,-d], uv: [1.0,0.0], normal: [0.0,-1.0,0.0] },
      // face left
      RVertex { position: [-w, h, d], uv: [1.0,0.0], normal: [-1.0,0.0,0.0] },
      RVertex { position: [-w,-h, d], uv: [1.0,1.0], normal: [-1.0,0.0,0.0] },
      RVertex { position: [-w,-h,-d], uv: [0.0,1.0], normal: [-1.0,0.0,0.0] },
      RVertex { position: [-w,-h,-d], uv: [0.0,1.0], normal: [-1.0,0.0,0.0] },
      RVertex { position: [-w, h,-d], uv: [0.0,0.0], normal: [-1.0,0.0,0.0] },
      RVertex { position: [-w, h, d], uv: [1.0,0.0], normal: [-1.0,0.0,0.0] },
      // face right
      RVertex { position: [ w, h,-d], uv: [1.0,0.0], normal: [1.0,0.0,0.0] },
      RVertex { position: [ w,-h,-d], uv: [1.0,1.0], normal: [1.0,0.0,0.0] },
      RVertex { position: [ w,-h, d], uv: [0.0,1.0], normal: [1.0,0.0,0.0] },
      RVertex { position: [ w,-h, d], uv: [0.0,1.0], normal: [1.0,0.0,0.0] },
      RVertex { position: [ w, h, d], uv: [0.0,0.0], normal: [1.0,0.0,0.0] },
      RVertex { position: [ w, h,-d], uv: [1.0,0.0], normal: [1.0,0.0,0.0] },
      // face back
      RVertex { position: [-w, h,-d], uv: [0.0,1.0], normal: [0.0,0.0,-1.0] },
      RVertex { position: [-w,-h,-d], uv: [0.0,0.0], normal: [0.0,0.0,-1.0] },
      RVertex { position: [ w,-h,-d], uv: [1.0,0.0], normal: [0.0,0.0,-1.0] },
      RVertex { position: [ w,-h,-d], uv: [1.0,0.0], normal: [0.0,0.0,-1.0] },
      RVertex { position: [ w, h,-d], uv: [1.0,1.0], normal: [0.0,0.0,-1.0] },
      RVertex { position: [-w, h,-d], uv: [0.0,1.0], normal: [0.0,0.0,-1.0] },
      // face front
      RVertex { position: [ w, h, d], uv: [1.0,0.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [ w,-h, d], uv: [1.0,1.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [-w,-h, d], uv: [0.0,1.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [-w,-h, d], uv: [0.0,1.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [-w, h, d], uv: [0.0,0.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [ w, h, d], uv: [1.0,0.0], normal: [0.0,0.0,1.0] },
    ]
  }
  pub fn rect(width: f32, height: f32, z_index: f32) -> Vec<RVertex> {
    let w = width / 2.0;
    let h = height / 2.0;
    vec![
      RVertex { position: [-w, -h, z_index], uv: [0.0,1.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [w, -h, z_index], uv: [1.0,1.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [w, h, z_index], uv: [1.0,0.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [w, h, z_index], uv: [1.0,0.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [-w, h, z_index], uv: [0.0,0.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [-w, -h, z_index], uv: [0.0,1.0], normal: [0.0,0.0,1.0] },
    ]
  }
  pub fn reg_polygon(radius:f32, sides:u32, z_index:f32) -> Vec<RVertex> {
    let mut v: Vec<RVertex> = vec![];
    let da = 2.0 * PI / sides as f32;

    // build polygon
    let mut x0 = 1.0;
    let mut y0 = 0.0;
    for _ in 0..sides {
      let x1 = f32::cos(da) * x0 - f32::sin(da) * y0;
      let y1 = f32::cos(da) * y0 + f32::sin(da) * x0;
      // build slice
      let p1 = [x0 * radius, y0 * radius, z_index];
      let p2 = [x1 * radius, y1 * radius, z_index];
      let p3 = [0.0, 0.0, z_index];
      let u1 = [(1.0 + x0)/2.0, 1.0 - (1.0 + y0)/2.0];
      let u2 = [(1.0 + x1)/2.0, 1.0 - (1.0 + y1)/2.0];
      let u3 = [0.5, 0.5];
      // build arrays
      v.push(RVertex{ position:p1, uv:u1, normal:[0.0, 0.0, 1.0] });
      v.push(RVertex{ position:p2, uv:u2, normal:[0.0, 0.0, 1.0] });
      v.push(RVertex{ position:p3, uv:u3, normal:[0.0, 0.0, 1.0] });
      // prepare next slice
      x0 = x1;
      y0 = y1;
    }
    
    v
  }
  pub fn flip_uv_y(input: &mut Vec<RVertex>) {
    for v in input {
      v.uv[1] = 1.0 - v.uv[1];
    }
  }
}