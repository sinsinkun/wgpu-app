use std::{fs, path::Path};

use crate::wgpu_root::{RCamera, RPipelineId, Renderer};
use crate::primitives::{Primitives, Shape};

// input handling helper
#[derive(Debug, Clone, PartialEq)]
pub enum InputState {
  None, Press, Hold, Release
}
pub enum InputKey {
  Up, Down, Left, Right, Fwd, Bkwd,
}

#[derive(Debug)]
pub struct InputCache {
  move_x: i32,
  move_y: i32,
  move_z: i32,
}
impl Default for InputCache {
  fn default() -> Self {
    Self {
      move_x: 0,
      move_y: 0,
      move_z: 0,
    }  
  }
}

pub struct AppEventLoop<'a> {
  renderer: Renderer<'a>,
  frame: u32, // max value: ~4,295,000,000
  shapes: Vec<Shape>,
  camera: RCamera,
  screen_center: (f32, f32),
  pub input_cache: InputCache,
}

impl<'a> AppEventLoop<'a> {
  pub fn new(wgpu: Renderer<'a>, window_size: &(f32, f32)) -> Self {
    let mut cam = RCamera::new_persp(60.0, 1.0, 1000.0);
    cam.position = [0.0, 0.0, 200.0];

    Self{
      renderer: wgpu,
      shapes: vec![],
      frame: 0,
      camera: cam,
      screen_center: (window_size.0 / 2.0, window_size.1 / 2.0),
      input_cache: InputCache::default(),
    }
  }

  // initialize app objects
  pub fn init(&mut self) {
    // initialize pipeline
    let texture1 = self.renderer.add_texture(10, 10, Some(Path::new("assets/test_uv_map.png")), false);
    let texture2 = self.renderer.add_texture(
      (self.screen_center.0 * 2.0) as u32,
      (self.screen_center.1 * 2.0) as u32,
      None,
      true
    );
    let pipe1: RPipelineId = self.renderer.add_pipeline(None, 10, Some(texture1), None);
    let pipe2: RPipelineId = match fs::read_to_string("assets/test.wgsl") {
      Ok(str) => { self.renderer.add_pipeline(Some(&str), 1, Some(texture2), None) }
      Err(..) => {
        println!("Could not find shader");
        self.renderer.add_pipeline(None, 1, Some(texture2), None)
      }
    };

    let cube_data1 = Primitives::cube(50.0, 50.0, 50.0);
    let cube_data2 = Primitives::cube(20.0, 20.0, 60.0);
    let cube_data3 = Primitives::cube(80.0, 80.0, 80.0);
    let cube1 = Shape::new(&mut self.renderer, pipe1, cube_data1);
    let mut cube2 = Shape::new(&mut self.renderer, pipe1, cube_data2);
    cube2.position = [60.0, 0.0, 0.0];
    cube2.rotate_axis = [1.0, 0.5, 0.0];
    let mut cube3 = Shape::new(&mut self.renderer, pipe1, cube_data3);
    cube3.position = [-60.0, 0.0, 0.0];
    cube3.rotate_axis = [0.0, 0.5, 1.0];

    let rect_data = Primitives::rect(0.5, 0.5, 0.0);
    let rect = Shape::new(&mut self.renderer, pipe2, rect_data);

    self.shapes.push(cube1);
    self.shapes.push(cube2);
    self.shapes.push(cube3);
    self.shapes.push(rect);
  }

  // handle inputs (asynchronous with render loop)
  pub fn input(&mut self, key: InputKey, state: InputState) {
    match key {
      InputKey::Up => { 
        if state == InputState::Press { self.input_cache.move_y += 1 }
        if state == InputState::Release { self.input_cache.move_y -= 1 }
      }
      InputKey::Down => {
        if state == InputState::Press { self.input_cache.move_y += -1 }
        if state == InputState::Release { self.input_cache.move_y -= -1 }
      }
      InputKey::Left => {
        if state == InputState::Press { self.input_cache.move_x += -1 }
        if state == InputState::Release { self.input_cache.move_x -= -1 }
      }
      InputKey::Right => {
        if state == InputState::Press { self.input_cache.move_x += 1 }
        if state == InputState::Release { self.input_cache.move_x -= 1 }
      }
      InputKey::Fwd => {
        if state == InputState::Press { self.input_cache.move_z += -1 }
        if state == InputState::Release { self.input_cache.move_z -= -1 }
      }
      InputKey::Bkwd => {
        if state == InputState::Press { self.input_cache.move_z += 1 }
        if state == InputState::Release { self.input_cache.move_z -= 1 }
      }
    }
  }

  // update logic (synchronous with render loop)
  pub fn update(&mut self) {
    // logic updates
    self.camera.position[0] += self.input_cache.move_x as f32 * 5.0;
    self.camera.position[1] += self.input_cache.move_y as f32 * 5.0;
    self.camera.position[2] += self.input_cache.move_z as f32 * 5.0;
    // render logic updates
    for obj in &mut self.shapes {
      if obj.id.0 == 1 {
        obj.position = [-self.screen_center.0 * 0.75, -self.screen_center.1 * 0.75, 0.0];
        obj.scale = [self.screen_center.0, self.screen_center.1, 1.0];
        self.renderer.update_object(
          obj.id,
          &obj.position,
          &obj.rotate_axis,
          obj.rotate_deg,
          &obj.scale,
          true,
          None,
        );
      } else {
        obj.rotate_deg = self.frame as f32;
        self.renderer.update_object(
          obj.id,
          &obj.position,
          &obj.rotate_axis,
          obj.rotate_deg,
          &obj.scale,
          true,
          Some(&self.camera)
        );
      }
    }
  }

  // call render
  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    self.frame += 1;
    self.renderer.render_texture(&[0], 1);
    match self.renderer.render(&[0, 1]) {
      Ok(_) => Ok(()),
      // Reconfigure the surface if lost
      Err(wgpu::SurfaceError::Lost) => {
        self.renderer.resize_canvas(self.renderer.config.width, self.renderer.config.height);
        self.update();
        Ok(())
      }
      // The system is out of memory, we should probably quit
      Err(wgpu::SurfaceError::OutOfMemory) => Err(wgpu::SurfaceError::OutOfMemory),
      // All other errors (Outdated, Timeout) should be resolved by the next frame
      Err(e) => {
        eprintln!("Render error: {:?}", e);
        Ok(())
      }
    }
  }

  // resize event
  pub fn resize(&mut self, width: u32, height: u32) {
    self.renderer.resize_canvas(width, height);
    self.screen_center = (width as f32 / 2.0, height as f32 / 2.0);
    self.renderer.update_texture_size(1, Some(1), width, height);
    self.update();
  }
}