#![allow(dead_code)]

use std::{fs, str::Split};

use super::RVertex;

#[derive(Debug, PartialEq)]
pub enum ModelError {
  FileError,
  DataError
}

#[derive(Debug)]
enum ObjDataType {
  None,
  Vertex,
  UV,
  Normal,
  Index
}

#[derive(Debug)]
pub struct ModelLoader;
impl ModelLoader {
  pub fn load_obj(file_path: &str) -> Result<Vec<RVertex>, ModelError> {
    let data: String = fs::read_to_string(file_path).map_err(|_| ModelError::FileError )?;
    let data_arr: Split<&str> = data.split("\n");

    // collections
    let mut raw_verts: Vec<[f32; 3]> = Vec::new();
    let mut raw_uvs: Vec<[f32; 2]> = Vec::new();
    let mut raw_normals: Vec<[f32; 3]> = Vec::new();
    let mut output: Vec<RVertex> = Vec::new();

    for str in data_arr {
      let otype = if str.starts_with("v ") { ObjDataType::Vertex }
      else if str.starts_with("vt ") { ObjDataType::UV }
      else if str.starts_with("vn ") { ObjDataType::Normal }
      else if str.starts_with("f ") { ObjDataType::Index }
      else { ObjDataType::None };

      let str_arr: Split<&str> = str.split(" ");
      match otype {
        ObjDataType::None => { continue; }
        ObjDataType::Vertex => {
          let mut v: [f32; 3] = [0.0, 0.0, 0.0];
          for (i, x) in str_arr.enumerate() {
            if i == 0 { continue; }
            let n: f32 = x.parse::<f32>().map_err(|_| ModelError::DataError)?;
            v[i-1] = n;
          }
          raw_verts.push(v);
        }
        ObjDataType::UV => {
          let mut v: [f32; 2] = [0.0, 0.0];
          for (i, x) in str_arr.enumerate() {
            if i == 0 { continue; }
            let n: f32 = x.parse::<f32>().map_err(|_| ModelError::DataError)?;
            v[i-1] = n;
          }
          raw_uvs.push(v);
        }
        ObjDataType::Normal => {
          let mut v: [f32; 3] = [0.0, 0.0, 0.0];
          for (i, x) in str_arr.enumerate() {
            if i == 0 { continue; }
            let n: f32 = x.parse::<f32>().map_err(|_| ModelError::DataError)?;
            v[i-1] = n;
          }
          raw_normals.push(v);
        }
        ObjDataType::Index => {
          let mut v1: Option<RVertex> = None;
          let mut v3: Option<RVertex> = None;
          for (i, x) in str_arr.enumerate() {
            if i == 1 { 
              let v = ModelLoader::obj_index_parse(x, &raw_verts, &raw_uvs, &raw_normals)?;
              v1 = Some(v.clone());
              output.push(v);
            } else if i == 2 {
              let v = ModelLoader::obj_index_parse(x, &raw_verts, &raw_uvs, &raw_normals)?;
              output.push(v);
            } else if i == 3 {
              let v = ModelLoader::obj_index_parse(x, &raw_verts, &raw_uvs, &raw_normals)?;
              v3 = Some(v.clone());
              output.push(v);
            } else if i == 4 {
              let v = ModelLoader::obj_index_parse(x, &raw_verts, &raw_uvs, &raw_normals)?;
              output.push(v3.unwrap());
              output.push(v);
              output.push(v1.unwrap());
            }
          }
        }
      }
    }

    Ok(output)
  }

  fn obj_index_parse(
    str: &str,
    raw_verts: &Vec<[f32; 3]>,
    raw_uvs: &Vec<[f32;2]>,
    raw_normals: &Vec<[f32;3]>
  ) -> Result<RVertex, ModelError> {
    let str_arr = str.split("/");
    let mut o = RVertex { 
      position: [0.0, 0.0, 0.0],
      uv: [0.0, 0.0],
      normal: [0.0, 0.0, 0.0]
    };

    for (i, s) in str_arr.enumerate() {
      let n: usize = s.parse::<usize>().map_err(|_| ModelError::DataError)?;
      if i == 0 { o.position = raw_verts[n - 1]; }
      else if i == 1 { o.uv = raw_uvs[n - 1]; }
      else if i == 2 { o.normal = raw_normals[n - 1]; }
    }

    Ok(o)
  }

  // pub fn load_gltf() {

  // }

  // pub fn load_gltf_mesh() {

  // }
}

#[cfg(test)]
mod model_loader_tests {
  use super::*;
  
  #[test]
  fn load_obj() {
    let o = ModelLoader::load_obj("assets/monkey.obj");
    assert_ne!(o, Err(ModelError::FileError));
    assert_ne!(o, Err(ModelError::DataError));
  }
}