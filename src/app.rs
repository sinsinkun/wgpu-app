use std::sync::Arc;
use winit::window::Window;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{Key, NamedKey};
use winit::dpi::PhysicalSize;

use crate::wgpu_root::{RCamera, RPipelineId, Renderer};
use crate::primitives::{Primitives, Shape};

pub struct AppEventLoop<'a> {
  renderer: Renderer<'a>,
  frame: u32, // max value: ~4,295,000,000
  shapes: Vec<Shape>,
  camera: RCamera,
  screen_center: (f32, f32),
}

impl AppEventLoop<'_> {
  pub fn new(window: Arc<Window>, window_size: &(f32, f32)) -> Self {
    let wgpu = pollster::block_on(Renderer::new(window.clone()));
    let mut cam = RCamera::new_persp(60.0, 1.0, 1000.0);
    cam.position = [0.0, 0.0, 200.0];

    Self {
      renderer: wgpu,
      shapes: vec![],
      frame: 0,
      camera: cam,
      screen_center: (window_size.0 / 2.0, window_size.1 / 2.0),
    }
  }

  // initialize app objects
  pub fn init(&mut self) {
    // initialize pipeline
    let shader1 = wgpu::ShaderSource::Wgsl(include_str!("base.wgsl").into());
    let pipe1: RPipelineId = self.renderer.add_pipeline(shader1, 10, None, None);

    let cube_data1 = Primitives::cube(50.0, 50.0, 50.0);
    let cube_data2 = Primitives::cube(60.0, 60.0, 60.0);
    let cube_data3 = Primitives::cube(80.0, 80.0, 80.0);
    let cube1 = Shape::new(&mut self.renderer, pipe1, cube_data1);
    let mut cube2 = Shape::new(&mut self.renderer, pipe1, cube_data2);
    cube2.position = [60.0, 60.0, 0.0];
    cube2.rotate_axis = [1.0, 0.5, 0.0];
    let mut cube3 = Shape::new(&mut self.renderer, pipe1, cube_data3);
    cube3.position = [-60.0, -30.0, 0.0];
    cube3.rotate_axis = [0.0, 0.5, 1.0];

    self.shapes.push(cube1);
    self.shapes.push(cube2);
    self.shapes.push(cube3);
  }

  // handle inputs
  pub fn input(&mut self, event: &WindowEvent, request_redraw: &mut bool) -> bool {
    match event {
      WindowEvent::KeyboardInput { 
        event: KeyEvent {
          logical_key: key,
          state,
          ..
        },
        ..
      } => {
        *request_redraw = true;
        match key.as_ref() {
          // rotate camera
          Key::Character("w") => {
            if state == &ElementState::Pressed {
              self.camera.position[2] -= 5.0;
            }
          }
          Key::Character("s") => {
            if state == &ElementState::Pressed {
              self.camera.position[2] += 5.0;
            }
          }
          Key::Character("a") => {
            if state == &ElementState::Pressed {
              self.camera.position[0] -= 5.0;
            }
          }
          Key::Character("d") => {
            if state == &ElementState::Pressed {
              self.camera.position[0] += 5.0;
            }
          }
          Key::Named(NamedKey::Space) => {
            if state == &ElementState::Pressed {
              self.camera.position[1] += 5.0;
            }
          }
          Key::Character("c") => {
            if state == &ElementState::Pressed {
              self.camera.position[1] -= 5.0;
            }
          }
          Key::Character("r") => {
            if state == &ElementState::Pressed {
              println!("reset");
              self.camera.position = [0.0, 0.0, 200.0];
              self.camera.rotate_deg = 0.0;
            }
          }
          // catch all
          _ => ()
        };
        true
      }
      WindowEvent::CursorMoved { position:_, .. } => {
        // to-do: mouse based camera rotation
        true
      },
      _ => true,
    }
  }

  // update logic
  pub fn update(&mut self) {
    for obj in &mut self.shapes {
      obj.rotate_deg = self.frame as f32;
      self.renderer.update_object(
        obj.pipe_id,
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

  // call render
  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    self.frame += 1;
    match self.renderer.render(&[0], None) {
      Ok(_) => Ok(()),
      // Reconfigure the surface if lost
      Err(wgpu::SurfaceError::Lost) => {
        self.renderer.resize_canvas(self.renderer.size);
        self.update();
        Ok(())
      }
      // The system is out of memory, we should probably quit
      Err(wgpu::SurfaceError::OutOfMemory) => Err(wgpu::SurfaceError::OutOfMemory),
      // All other errors (Outdated, Timeout) should be resolved by the next frame
      Err(e) => {
        eprintln!("{:?}", e);
        Ok(())
      }
    }
  }

  // resize event
  pub fn resize(&mut self, physical_size: PhysicalSize<u32>) {
    self.renderer.resize_canvas(physical_size);
    self.screen_center = (physical_size.width as f32 / 2.0, physical_size.height as f32 / 2.0);
    self.update();
  }
}