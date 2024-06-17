#![allow(dead_code)]

use crate::wgpu_renderer::{RVertex, PI};

// note: uv_y is inverted
pub struct Primitives;
impl Primitives {
  // util functions
  pub fn flip_uv_y(input: &mut Vec<RVertex>) {
    for v in input {
      v.uv[1] = 1.0 - v.uv[1];
    }
  }
  // 2d primitives
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
  pub fn torus_2d(outer_radius:f32, inner_radius:f32, sides: u32, z_index:f32) -> (Vec<RVertex>, Vec<u32>) {
    let mut v: Vec<RVertex> = vec![];
    let mut idx: Vec<u32> = vec![];
    let dr = inner_radius / outer_radius;
    // build points
    for i in 0..sides {
      let theta = 2.0 * PI * (i as f32) / (sides as f32);
      let x: f32 = f32::cos(theta);
      let y: f32 = f32::sin(theta);
      let v1 = RVertex {
        position: [x * outer_radius, y * outer_radius, z_index],
        uv: [(1.0 + x)/2.0, (1.0 + y)/2.0],
        normal: [0.0,0.0,1.0]
      };
      let v2 = RVertex {
        position: [x * inner_radius, y * inner_radius, z_index],
        uv: [(1.0 + dr * x)/2.0, (1.0 + dr * y)/2.0],
        normal: [0.0,0.0,1.0]
      };
      v.push(v1);
      v.push(v2);
    }
    // build index
    for i in 0..v.len() - 2 {
      if i % 2 == 0 {
        idx.push(i as u32 + 1); idx.push(i as u32); idx.push(i as u32 + 2);
      } else {
        idx.push(i as u32); idx.push(i as u32 + 1); idx.push(i as u32 + 2);
      }
    }
    // join back to first 2 vertices
    idx.push(v.len() as u32 - 1); idx.push(v.len() as u32 - 2); idx.push(0);
    idx.push(v.len() as u32 - 1); idx.push(0); idx.push(1);

    (v, idx)
  }
  // 3d primitives
  pub fn cube(width: f32, height: f32, depth: f32) -> Vec<RVertex> {
    let w = width /2.0;
    let h = height / 2.0;
    let d = depth / 2.0;
    vec![
      // face top
      RVertex { position: [ w,-h,-d], uv: [1.0,1.0], normal: [0.0,1.0,0.0] },
      RVertex { position: [ w,-h, d], uv: [1.0,0.0], normal: [0.0,1.0,0.0] },
      RVertex { position: [-w,-h,-d], uv: [0.0,1.0], normal: [0.0,1.0,0.0] },
      RVertex { position: [-w,-h, d], uv: [0.0,0.0], normal: [0.0,1.0,0.0] },
      RVertex { position: [-w,-h,-d], uv: [0.0,1.0], normal: [0.0,1.0,0.0] },
      RVertex { position: [ w,-h, d], uv: [1.0,0.0], normal: [0.0,1.0,0.0] },
      // face bottom
      RVertex { position: [ w, h, d], uv: [1.0,1.0], normal: [0.0,-1.0,0.0] },
      RVertex { position: [ w, h,-d], uv: [1.0,0.0], normal: [0.0,-1.0,0.0] },
      RVertex { position: [-w, h, d], uv: [0.0,1.0], normal: [0.0,-1.0,0.0] },
      RVertex { position: [-w, h,-d], uv: [0.0,0.0], normal: [0.0,-1.0,0.0] },
      RVertex { position: [-w, h, d], uv: [0.0,1.0], normal: [0.0,-1.0,0.0] },
      RVertex { position: [ w, h,-d], uv: [1.0,0.0], normal: [0.0,-1.0,0.0] },
      // face left
      RVertex { position: [-w,-h, d], uv: [1.0,1.0], normal: [-1.0,0.0,0.0] },
      RVertex { position: [-w, h, d], uv: [1.0,0.0], normal: [-1.0,0.0,0.0] },
      RVertex { position: [-w,-h,-d], uv: [0.0,1.0], normal: [-1.0,0.0,0.0] },
      RVertex { position: [-w, h,-d], uv: [0.0,0.0], normal: [-1.0,0.0,0.0] },
      RVertex { position: [-w,-h,-d], uv: [0.0,1.0], normal: [-1.0,0.0,0.0] },
      RVertex { position: [-w, h, d], uv: [1.0,0.0], normal: [-1.0,0.0,0.0] },
      // face right
      RVertex { position: [ w,-h,-d], uv: [1.0,1.0], normal: [1.0,0.0,0.0] },
      RVertex { position: [ w, h,-d], uv: [1.0,0.0], normal: [1.0,0.0,0.0] },
      RVertex { position: [ w,-h, d], uv: [0.0,1.0], normal: [1.0,0.0,0.0] },
      RVertex { position: [ w, h, d], uv: [0.0,0.0], normal: [1.0,0.0,0.0] },
      RVertex { position: [ w,-h, d], uv: [0.0,1.0], normal: [1.0,0.0,0.0] },
      RVertex { position: [ w, h,-d], uv: [1.0,0.0], normal: [1.0,0.0,0.0] },
      // face back
      RVertex { position: [-w,-h,-d], uv: [0.0,0.0], normal: [0.0,0.0,-1.0] },
      RVertex { position: [-w, h,-d], uv: [0.0,1.0], normal: [0.0,0.0,-1.0] },
      RVertex { position: [ w,-h,-d], uv: [1.0,0.0], normal: [0.0,0.0,-1.0] },
      RVertex { position: [ w, h,-d], uv: [1.0,1.0], normal: [0.0,0.0,-1.0] },
      RVertex { position: [ w,-h,-d], uv: [1.0,0.0], normal: [0.0,0.0,-1.0] },
      RVertex { position: [-w, h,-d], uv: [0.0,1.0], normal: [0.0,0.0,-1.0] },
      // face front
      RVertex { position: [ w,-h, d], uv: [1.0,1.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [ w, h, d], uv: [1.0,0.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [-w,-h, d], uv: [0.0,1.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [-w, h, d], uv: [0.0,0.0], normal: [0.0,0.0,1.0] },
      RVertex { position: [-w,-h, d], uv: [0.0,1.0], normal: [0.0,0.0,1.0] },
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
      1,0,2,3,2,0, // top
      5,4,6,7,6,4, // bottom
      9,8,10,11,10,8, // left
      13,12,14,15,14,12, // right
      17,16,18,19,18,16, // back
      21,20,22,23,22,20, // front
    ];
    (a, b)
  }
  pub fn cylinder(radius: f32, height: f32, sides: u32) -> (Vec<RVertex>, Vec<u32>) {
    let mut v: Vec<RVertex> = vec![];
    let mut idx: Vec<u32> = vec![];
    let h: f32 = height / 2.0;
    // build top/bottom center
    let top_center = RVertex {
      position: [0.0, h, 0.0],
      uv: [0.5, 0.5],
      normal: [0.0, 1.0, 0.0]
    };
    let bot_center = RVertex {
      position: [0.0, -h, 0.0],
      uv: [0.5, 0.5],
      normal: [0.0, 1.0, 0.0]
    };
    v.push(top_center);
    v.push(bot_center);
    // build top/bottom sides
    for i in 0..sides {
      let theta: f32 = 2.0 * PI * (i as f32 / sides as f32);
      let x: f32 = f32::cos(theta);
      let z: f32 = f32::sin(theta);
      let v1 = RVertex {
        position: [x * radius, h, z * radius],
        uv: [(1.0 + x) / 2.0, (1.0 + z) / 2.0],
        normal: [0.0, 1.0, 0.0]
      };
      let v2 = RVertex {
        position: [x * radius, -h, z * radius],
        uv: [(1.0 + x) / 2.0, (1.0 - z) / 2.0],
        normal: [0.0, -1.0, 0.0]
      };
      v.push(v1);
      v.push(v2);
    }
    // generate indexing
    for i in 2..v.len() - 2 {
      if i % 2 == 0 {
        // top
        idx.push(i as u32); idx.push(0); idx.push(i as u32 + 2);
      } else {
        // bottom
        idx.push(i as u32); idx.push(i as u32 + 2); idx.push(1);
      }
    }
    idx.push(v.len() as u32 - 2); idx.push(0); idx.push(2);
    idx.push(v.len() as u32 - 1); idx.push(3); idx.push(1);

    // build sides
    let new0 = v.len();
    for i in 0..sides + 1 {
      let theta: f32 = 2.0 * PI * (i as f32 / sides as f32);
      let x: f32 = f32::cos(theta);
      let z: f32 = f32::sin(theta);
      let v1 = RVertex {
        position: [x * radius, h, z * radius],
        uv: [(i as f32 / sides as f32), 1.0],
        normal: [x, 0.0, z]
      };
      let v2 = RVertex {
        position: [x * radius, -h, z * radius],
        uv: [(i as f32 / sides as f32), 0.0],
        normal: [x, 0.0, z]
      };
      v.push(v1);
      v.push(v2);
    }
    // generate indexing
    for i in new0..v.len() - 2 {
      if i % 2 == 0 {
        idx.push(i as u32 + 1); idx.push(i as u32); idx.push(i as u32 + 2);
      } else {
        idx.push(i as u32); idx.push(i as u32 + 1); idx.push(i as u32 + 2);
      }
    }

    (v, idx)
  }
  pub fn tube(outer_radius: f32, inner_radius: f32, height: f32, sides: u32) -> (Vec<RVertex>, Vec<u32>) {
    let mut v: Vec<RVertex> = vec![];
    let mut idx: Vec<u32> = vec![];
    let dr: f32 = inner_radius / outer_radius;
    let h: f32 = height / 2.0;

    // build top/bottom
    for i in 0..sides {
      let theta = 2.0 * PI * (i as f32) / (sides as f32);
      let x: f32 = f32::cos(theta);
      let z: f32 = f32::sin(theta);
      let v1 = RVertex {
        position: [x * outer_radius, h, z * outer_radius],
        uv: [(1.0 + x)/2.0, (1.0 + z)/2.0],
        normal: [0.0, 1.0, 0.0]
      };
      let v2 = RVertex {
        position: [x * outer_radius, -h, z * outer_radius],
        uv: [(1.0 + x)/2.0, (1.0 - z)/2.0],
        normal: [0.0, -1.0, 0.0]
      };
      let v3 = RVertex {
        position: [x * inner_radius, h, z * inner_radius],
        uv: [(1.0 + dr * x)/2.0, (1.0 + dr * z)/2.0],
        normal: [0.0, 1.0, 0.0]
      };
      let v4 = RVertex {
        position: [x * inner_radius, -h, z * inner_radius],
        uv: [(1.0 + dr * x)/2.0, (1.0 - dr * z)/2.0],
        normal: [0.0, -1.0, 0.0]
      };
      v.push(v1); v.push(v2); v.push(v3); v.push(v4);
    }
    // generate indexing
    for i in (0..v.len() - 5).step_by(2) {
      if i % 4 == 0 {
        idx.push(i as u32); idx.push(i as u32 + 2); idx.push(i as u32 + 4);
        idx.push(i as u32 + 3); idx.push(i as u32 + 1); idx.push(i as u32 + 5);
      } else {
        idx.push(i as u32 + 2); idx.push(i as u32); idx.push(i as u32 + 4);
        idx.push(i as u32 + 1); idx.push(i as u32 + 3); idx.push(i as u32 + 5);
      }
    }
    // join back to first 2 vertices
    idx.push(v.len() as u32 - 4); idx.push(v.len() as u32 - 2); idx.push(0);
    idx.push(0); idx.push(v.len() as u32 - 2); idx.push(2);
    idx.push(v.len() as u32 - 1); idx.push(v.len() as u32 - 3); idx.push(1);
    idx.push(v.len() as u32 - 1); idx.push(1); idx.push(3);

    // build sides
    let new0 = v.len();
    for i in 0..sides+1 {
      let theta = 2.0 * PI * (i as f32) / (sides as f32);
      let x: f32 = f32::cos(theta);
      let z: f32 = f32::sin(theta);
      let v1 = RVertex {
        position: [x * outer_radius, h, z * outer_radius],
        uv: [(i as f32) / (sides as f32), 1.0],
        normal: [x, 0.0, z]
      };
      let v2 = RVertex {
        position: [x * inner_radius, h, z * inner_radius],
        uv: [(i as f32) / (sides as f32), 1.0],
        normal: [x, 0.0, z]
      };
      let v3 = RVertex {
        position: [x * outer_radius, -h, z * outer_radius],
        uv: [(i as f32) / (sides as f32), 0.0],
        normal: [x, 0.0, z]
      };
      let v4 = RVertex {
        position: [x * inner_radius, -h, z * inner_radius],
        uv: [(i as f32) / (sides as f32), 0.0],
        normal: [x, 0.0, z]
      };
      v.push(v1); v.push(v2); v.push(v3); v.push(v4);
    }
    for i in (new0..v.len() - 4).step_by(2) {
      if i % 4 == 0 {
        idx.push(i as u32 + 2); idx.push(i as u32); idx.push(i as u32 + 4);
        idx.push(i as u32 + 1); idx.push(i as u32 + 3); idx.push(i as u32 + 5);
      } else {
        idx.push(i as u32); idx.push(i as u32 + 2); idx.push(i as u32 + 4);
        idx.push(i as u32 + 3); idx.push(i as u32 + 1); idx.push(i as u32 + 5);
      }
    }

    (v, idx)
  }
  pub fn cone(radius: f32, height: f32, sides: u32) -> (Vec<RVertex>, Vec<u32>) {
    let mut v: Vec<RVertex> = vec![];
    let mut idx: Vec<u32> = vec![];

    // build top
    let v0 = RVertex {
      position: [0.0, height, 0.0],
      uv: [0.5, 1.0],
      normal: [0.0, 1.0, 0.0]
    };
    v.push(v0);
    // build sides
    for i in 0..sides+1 {
      let theta = 2.0 * PI * (i as f32) / (sides as f32);
      let x: f32 = f32::cos(theta);
      let z: f32 = f32::sin(theta);
      let v1 = RVertex {
        position: [x * radius, 0.0, z * radius],
        uv: [(i as f32) / (sides as f32), 0.0],
        normal: [x, 0.0, z]
      };
      v.push(v1);
    }
    // generate index
    for i in 1..v.len() - 1 {
      idx.push(i as u32 + 1); idx.push(i as u32); idx.push(0);
    }
    // build bottom center
    let v0 = RVertex {
      position: [0.0, 0.0, 0.0],
      uv: [0.5, 0.5],
      normal: [0.0, -1.0, 0.0]
    };
    v.push(v0);
    // build bottom face
    let new0 = v.len();
    for i in 0..sides {
      let theta = 2.0 * PI * (i as f32) / (sides as f32);
      let x: f32 = f32::cos(theta);
      let z: f32 = f32::sin(theta);
      let v1 = RVertex {
        position: [x * radius, 0.0, z * radius],
        uv: [(1.0 + x)/2.0, (1.0 - z)/2.0],
        normal: [0.0, -1.0, 0.0]
      };
      v.push(v1);
    }
    // generate index
    for i in new0..v.len() {
      idx.push(i as u32); idx.push(i as u32 + 1); idx.push(new0 as u32 - 1);
    }
    idx.push(v.len() as u32 - 1); idx.push(new0 as u32); idx.push(new0 as u32 - 1);

    (v, idx)
  }
  pub fn sphere(radius: f32, sides: u32, slices: u32) -> (Vec<RVertex>, Vec<u32>) {
    let mut v: Vec<RVertex> = vec![];
    let mut idx: Vec<u32> = vec![];

    // add top point
    let v0 = RVertex {
      position: [0.0, radius, 0.0],
      uv: [0.5, 0.5],
      normal: [0.0, 1.0, 0.0]
    };
    v.push(v0);
    // add points per slice
    for i in 0..slices - 1 {
      let phi: f32 = PI * (i + 1) as f32 / slices as f32;
      for j in 0..sides {
        let theta: f32 = 2.0 * PI * j as f32 / sides as f32;
        let x = f32::sin(phi) * f32::cos(theta);
        let y = f32::cos(phi);
        let z = f32::sin(phi) * f32::sin(theta);
        let v1 = RVertex {
          position: [x * radius, y * radius, z * radius],
          uv: [(1.0 + x)/2.0, (1.0 + z)/2.0],
          normal: [x, y, z]
        };
        v.push(v1);
      }
    }
    // add bottom point
    let v0 = RVertex {
      position: [0.0, -radius, 0.0],
      uv: [0.5, 0.5],
      normal: [0.0, -1.0, 0.0]
    };
    v.push(v0);
    // generate top/bottom index
    for i in 0..sides {
      let mut i0: u32 = i + 1;
      let mut i1: u32 = (i + 1) % sides + 1;
      idx.push(0); idx.push(i1); idx.push(i0);
      i0 = i + sides * (slices - 2) + 1;
      i1 = (i + 1) % sides + sides * (slices - 2) + 1;
      idx.push(v.len() as u32 - 1); idx.push(i0); idx.push(i1);
    }
    // generate slice indices
    for j in 0..slices - 2 {
      let j0: u32 = j * sides + 1;
      let j1: u32 = (j + 1) * sides + 1;
      for i in 0..sides {
        let i0: u32 = j0 + i;
        let i1: u32 = j0 + (i + 1) % sides;
        let i2: u32 = j1 + (i + 1) % sides;
        let i3: u32 = j1 + i;
        idx.push(i0); idx.push(i1); idx.push(i2);
        idx.push(i2); idx.push(i3); idx.push(i0);
      }
    }

    (v, idx)
  }
  pub fn hemisphere(radius: f32, sides: u32, slices: u32) -> (Vec<RVertex>, Vec<u32>) {
    let mut v: Vec<RVertex> = vec![];
    let mut idx: Vec<u32> = vec![];

    // add top point
    let v0 = RVertex {
      position: [0.0, radius, 0.0],
      uv: [0.5, 0.5],
      normal: [0.0, 1.0, 0.0]
    };
    v.push(v0);
    // generate points per slice
    for i in 0..slices {
      let phi: f32 = PI * (i + 1) as f32 / (2 * slices) as f32;
      for j in 0..sides {
        let theta: f32 = 2.0 * PI * j as f32 / sides as f32;
        let x = f32::sin(phi) * f32::cos(theta);
        let y = f32::cos(phi);
        let z = f32::sin(phi) * f32::sin(theta);
        let v1 = RVertex {
          position: [x * radius, y * radius, z * radius],
          uv: [(1.0 + x)/2.0, (1.0 + z)/2.0],
          normal: [x, y, z]
        };
        v.push(v1);
      }
    }
    // generate top index
    for i in 0..sides {
      let i0 = i + 1;
      let i1 = (i + 1) % sides + 1;
      idx.push(0); idx.push(i1); idx.push(i0);
    }
    // generate slice indices
    for j in 0..slices-1 {
      let j0 = j * sides + 1;
      let j1 = (j + 1) * sides + 1;
      for i in 0..sides {
        let i0: u32 = j0 + i;
        let i1: u32 = j0 + (i + 1) % sides;
        let i2: u32 = j1 + (i + 1) % sides;
        let i3: u32 = j1 + i;
        idx.push(i0); idx.push(i1); idx.push(i2);
        idx.push(i2); idx.push(i3); idx.push(i0);
      }
    }
    // generate bottom face
    let new0: u32 = v.len() as u32;
    for i in 0..sides {
      let theta: f32 = 2.0 * PI * i as f32 / sides as f32;
      let x = f32::cos(theta);
      let z = f32::sin(theta);
      let v1 = RVertex {
        position: [x * radius, 0.0, z * radius],
        uv: [(1.0 + x)/2.0, (1.0 - z)/2.0],
        normal: [0.0, -1.0, 0.0]
      };
      v.push(v1);
    }
    // add bottom point
    let v0 = RVertex {
      position: [0.0, 0.0, 0.0],
      uv: [0.5, 0.5],
      normal: [0.0, -1.0, 0.0]
    };
    v.push(v0);
    let c: u32 = (v.len() - 1) as u32;
    // generate index
    for i in 0..sides-1 {
      idx.push(c); idx.push(new0 + i); idx.push(new0 + i + 1);
    }
    idx.push(c); idx.push(c - 1); idx.push(new0);

    (v, idx)
  }
}