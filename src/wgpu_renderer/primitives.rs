#![allow(dead_code)]

use crate::wgpu_renderer::{RVertex, PI};

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
  pub fn cube_indexed(width: f32, height: f32, depth: f32) -> (Vec<RVertex>, Vec<u32>) {
    let w = width /2.0;
    let h = height / 2.0;
    let d = depth / 2.0;
    let a = vec![
      // face top
      RVertex { position: [ w,-h, d], uv: [1.0,0.0], normal: [0.0,1.0,0.0] },
      RVertex { position: [ w,-h,-d], uv: [1.0,1.0], normal: [0.0,1.0,0.0] },
      RVertex { position: [-w,-h,-d], uv: [0.0,1.0], normal: [0.0,1.0,0.0] },
      RVertex { position: [-w,-h, d], uv: [0.0,0.0], normal: [0.0,1.0,0.0] },
      // face bottom
      RVertex { position: [ w, h,-d], uv: [1.0,0.0], normal: [0.0,-1.0,0.0] },
      RVertex { position: [ w, h, d], uv: [1.0,1.0], normal: [0.0,-1.0,0.0] },
      RVertex { position: [-w, h, d], uv: [0.0,1.0], normal: [0.0,-1.0,0.0] },
      RVertex { position: [-w, h,-d], uv: [0.0,0.0], normal: [0.0,-1.0,0.0] },
      // face left
      RVertex { position: [-w, h, d], uv: [1.0,0.0], normal: [-1.0,0.0,0.0] },
      RVertex { position: [-w,-h, d], uv: [1.0,1.0], normal: [-1.0,0.0,0.0] },
      RVertex { position: [-w,-h,-d], uv: [0.0,1.0], normal: [-1.0,0.0,0.0] },
      RVertex { position: [-w, h,-d], uv: [0.0,0.0], normal: [-1.0,0.0,0.0] },
      // face right
      RVertex { position: [ w, h,-d], uv: [1.0,0.0], normal: [1.0,0.0,0.0] },
      RVertex { position: [ w,-h,-d], uv: [1.0,1.0], normal: [1.0,0.0,0.0] },
      RVertex { position: [ w,-h, d], uv: [0.0,1.0], normal: [1.0,0.0,0.0] },
      RVertex { position: [ w, h, d], uv: [0.0,0.0], normal: [1.0,0.0,0.0] },
      // face back
      RVertex { position: [-w, h,-d], uv: [0.0,1.0], normal: [0.0,0.0,-1.0] },
      RVertex { position: [-w,-h,-d], uv: [0.0,0.0], normal: [0.0,0.0,-1.0] },
      RVertex { position: [ w,-h,-d], uv: [1.0,0.0], normal: [0.0,0.0,-1.0] },
      RVertex { position: [ w, h,-d], uv: [1.0,1.0], normal: [0.0,0.0,-1.0] },
      // face front
      RVertex { position: [ w, h, d], uv: [1.0,0.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [ w,-h, d], uv: [1.0,1.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [-w,-h, d], uv: [0.0,1.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [-w, h, d], uv: [0.0,0.0], normal: [0.0,0.0,1.0] },
    ];
    let b = vec![
      0,1,2,2,3,0, // top
      4,5,6,6,7,4, // bottom
      8,9,10,10,11,8, // left
      12,13,14,14,15,12, // right
      16,17,18,18,19,16, // back
      20,21,22,22,23,20, // front
    ];
    (a, b)
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
  pub fn rect_indexed(width: f32, height: f32, z_index: f32) -> (Vec<RVertex>, Vec<u32>) {
    let w = width / 2.0;
    let h = height / 2.0;
    let a = vec![
      RVertex { position: [-w, -h, z_index], uv: [0.0,1.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [w, -h, z_index], uv: [1.0,1.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [w, h, z_index], uv: [1.0,0.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [-w, h, z_index], uv: [0.0,0.0], normal: [0.0,0.0,1.0] },
    ];
    let b = vec![0,1,2,2,3,0];
    (a, b)
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