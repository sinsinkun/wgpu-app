#![allow(dead_code)]

const PI: f32 = 3.14159265;

pub struct Mat4;
impl Mat4 {
  pub fn size_in_bytes() -> u32 { 4 * 4 * 4 * 3 }
  pub fn identity() -> [f32; 16] {
    [
      1.0, 0.0, 0.0, 0.0,
      0.0, 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, 0.0, 1.0
    ]
  }
  pub fn perspective(fov_y: f32, aspect_ratio: f32, near: f32, far: f32) -> [f32; 16] {
    let f = f32::tan(PI * 0.5 - 0.5 * fov_y * PI / 180.0);
    let range = 1.0 / (near - far);
    let a = f / aspect_ratio;
    let c = far * range;
    let d = near * far * range;
    [
      a, 0.0, 0.0, 0.0,
      0.0, f, 0.0, 0.0,
      0.0, 0.0, c, -1.0,
      0.0, 0.0, d, 0.0
    ]
  }
  pub fn ortho(left: f32, right: f32, top: f32, bottom: f32, near: f32, far: f32) -> [f32; 16] {
    let a = 2.0 / (right - left);
    let b = 2.0 / (top - bottom);
    let c = 1.0 / (near - far);
    let d = (right + left) / (left - right);
    let e = (top + bottom) / (bottom - top);
    let f = near / (near - far);
    [
      a, 0.0, 0.0, 0.0,
      0.0, b, 0.0, 0.0,
      0.0, 0.0, c, 0.0,
      d, e, f, 1.0
    ]
  }
  pub fn translate(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
      1.0, 0.0, 0.0, 0.0,
      0.0, 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      x, y, z, 1.0
    ]
  }
  // !-- NOT WORKING
  pub fn rotate(axis: &[f32; 3], deg: f32) -> [f32; 16] {
    // normalize axis
    let n = f32::sqrt(axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]);
    let x = axis[0] / n;
    let y = axis[1] / n;
    let z = axis[2] / n;
    // helpers
    let xx = x * x;
    let yy = y * y;
    let zz = z * z;
    let c = f32::cos(deg * PI / 180.0);
    let s = f32::sin(deg * PI / 180.0);
    let o = 1.0 - c;
    // builders
    let a = xx + (1.0 - xx) * c;
    let b = x * y * o + z * s;
    let c = x * z * o - y * s;
    let d = x * y * o - z * s;
    let e = yy + (1.0 - yy) * c;
    let f = y * z * o + x * s;
    let g = x * z * o + y * s;
    let h = y * z * o - x * s;
    let i = zz + (1.0 - zz) * c;
    [
      a, b, c, 0.0,
      d, e, f, 0.0,
      g, h, i, 0.0,
      0.0, 0.0, 0.0, 1.0
    ]
  }
  pub fn scale(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
      x, 0.0, 0.0, 0.0,
      0.0, y, 0.0, 0.0,
      0.0, 0.0, z, 0.0,
      0.0, 0.0, 0.0, 1.0
    ]
  }
  pub fn multiply(a: &[f32;16], b: &[f32;16]) -> [f32; 16] {
    let mut dst = [0.0; 16];
    let a00 = a[0];
    let a01 = a[1];
    let a02 = a[2];
    let a03 = a[3];
    let a10 = a[ 4 + 0];
    let a11 = a[ 4 + 1];
    let a12 = a[ 4 + 2];
    let a13 = a[ 4 + 3];
    let a20 = a[ 8 + 0];
    let a21 = a[ 8 + 1];
    let a22 = a[ 8 + 2];
    let a23 = a[ 8 + 3];
    let a30 = a[12 + 0];
    let a31 = a[12 + 1];
    let a32 = a[12 + 2];
    let a33 = a[12 + 3];
    let b00 = b[0];
    let b01 = b[1];
    let b02 = b[2];
    let b03 = b[3];
    let b10 = b[ 4 + 0];
    let b11 = b[ 4 + 1];
    let b12 = b[ 4 + 2];
    let b13 = b[ 4 + 3];
    let b20 = b[ 8 + 0];
    let b21 = b[ 8 + 1];
    let b22 = b[ 8 + 2];
    let b23 = b[ 8 + 3];
    let b30 = b[12 + 0];
    let b31 = b[12 + 1];
    let b32 = b[12 + 2];
    let b33 = b[12 + 3];

    dst[ 0] = a00 * b00 + a10 * b01 + a20 * b02 + a30 * b03;
    dst[ 1] = a01 * b00 + a11 * b01 + a21 * b02 + a31 * b03;
    dst[ 2] = a02 * b00 + a12 * b01 + a22 * b02 + a32 * b03;
    dst[ 3] = a03 * b00 + a13 * b01 + a23 * b02 + a33 * b03;
    dst[ 4] = a00 * b10 + a10 * b11 + a20 * b12 + a30 * b13;
    dst[ 5] = a01 * b10 + a11 * b11 + a21 * b12 + a31 * b13;
    dst[ 6] = a02 * b10 + a12 * b11 + a22 * b12 + a32 * b13;
    dst[ 7] = a03 * b10 + a13 * b11 + a23 * b12 + a33 * b13;
    dst[ 8] = a00 * b20 + a10 * b21 + a20 * b22 + a30 * b23;
    dst[ 9] = a01 * b20 + a11 * b21 + a21 * b22 + a31 * b23;
    dst[10] = a02 * b20 + a12 * b21 + a22 * b22 + a32 * b23;
    dst[11] = a03 * b20 + a13 * b21 + a23 * b22 + a33 * b23;
    dst[12] = a00 * b30 + a10 * b31 + a20 * b32 + a30 * b33;
    dst[13] = a01 * b30 + a11 * b31 + a21 * b32 + a31 * b33;
    dst[14] = a02 * b30 + a12 * b31 + a22 * b32 + a32 * b33;
    dst[15] = a03 * b30 + a13 * b31 + a23 * b32 + a33 * b33;
    dst
  }
  pub fn transpose(src: &[f32;16]) -> [f32;16] {
    let mut dst = [0.0; 16];
    for i in 0..4 {
      for j in 0..4 {
        dst[i*4 + j] = src[j*4 + i];
      }
    }
    dst
  }
}

pub struct Vec;
impl Vec {
  pub fn add_vec3(v1: &[f32; 3], v2: &[f32; 3]) -> [f32; 3] {
    [
      v1[0] + v2[0],
      v1[1] + v2[1],
      v1[2] + v2[2]
    ]
  }
  pub fn subtract_vec3(v1: &[f32; 3], v2: &[f32; 3]) -> [f32; 3] {
    [
      v1[0] - v2[0],
      v1[1] - v2[1],
      v1[2] - v2[2]
    ]
  }
  pub fn dot_vec3(v1: &[f32; 3], v2: &[f32; 3]) -> f32 {
    let mut out = v1[0] * v2[0];
    out = out + v1[1] * v2[1];
    out = out + v1[2] * v2[2];
    out
  }
  pub fn cross_vec3(v1: &[f32; 3], v2: &[f32; 3]) -> [f32; 3] {
    [
      v1[1] * v2[2] - v1[2] * v2[1],
      v1[2] * v2[0] - v1[0] * v2[2],
      v1[0] * v2[1] - v1[1] * v2[0]
    ]
  }
  pub fn normalize_vec3(v: &[f32; 3]) -> [f32; 3] {
    let n = f32::sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
    [
      v[0] / n,
      v[1] / n,
      v[2] / n
    ]
  }
}

#[cfg(test)]
mod lin_alg_tests {
  use super::*;
  #[test]
  fn mat4_ortho() {
    let o = Mat4::ortho(0.0, 200.0, 0.0, 100.0, 0.0, 1000.0);
    assert_eq!(o, [
      0.01, 0.0, 0.0, 0.0,
      0.0, -0.02, 0.0, 0.0,
      0.0, 0.0, -0.001, 0.0,
      -1.0, 1.0, 0.0, 1.0, 
    ]);
  }
  #[test]
  fn mat4_persp() {
    let o = Mat4::perspective(80.0, 1.5, 1.0, 1000.0);
    assert_eq!(o, [
      0.79450244, 0.0, 0.0, 0.0,
      0.0, 1.1917536, 0.0, 0.0,
      0.0, 0.0, -1.001001, -1.0,
      0.0, 0.0, -1.001001, 0.0, 
    ]);
  }
  #[test]
  fn mat4_rotate1() {
    let o = Mat4::rotate(&[0.0, 0.0, 1.0], 30.0);
    assert_eq!(o, [
      0.8660254, 0.5, 0.0, 0.0,
      -0.5, 0.8660254, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, 0.0, 1.0
    ]);
  }
  #[test]
  fn mat4_rotate2() {
    let o = Mat4::rotate(&[0.0, 1.0, 0.0], 45.0);
    assert_eq!(o, [
      0.70710677, 0.0, -0.70710677, 0.0,
      0.0, 1.0, 0.0, 0.0,
      0.70710677, 0.0, 0.70710677, 0.0,
      0.0, 0.0, 0.0, 1.0
    ]);
  }
  #[test]
  fn mat4_rotate3() {
    let o = Mat4::rotate(&[1.0, 0.0, 0.0], 60.0);
    assert_eq!(o, [
      1.0, 0.0, 0.0, 0.0,
      0.0, 0.5, 0.86602539, 0.0,
      0.0, -0.86602539, 0.5, 0.0,
      0.0, 0.0, 0.0, 1.0
    ]);
  }
  #[test]
  fn mat4_rotate4() {
    let o = Mat4::rotate(&[1.0, 0.0, 1.0], 90.0);
    assert_eq!(o, [
      0.5, 0.70710677, 0.5, 0.0,
      -0.70710677, 0.0, 0.70710677, 0.0,
      0.5, -0.70710677, 0.5, 0.0,
      0.0, 0.0, 0.0, 1.0
    ]);
  }
  #[test]
  fn mat4_rotate5() {
    let o = Mat4::rotate(&[0.0, 2.0, 1.0], 140.0);
    assert_eq!(o, [
      -0.76604444, 0.28746337, -0.57492673, 0.0,
      -0.287463367, 0.6467911, 0.7064178, 0.0,
      0.57492673, 0.7064178, -0.41283557, 0.0,
      0.0, 0.0, 0.0, 1.0
    ]);
  }
  #[test] #[ignore]
  fn mat4_multiply() {
    todo!();
  }
}