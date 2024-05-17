use std::sync::Arc;
use winit::window::Window;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::Key;
use winit::dpi::PhysicalSize;

use crate::wgpu_root::{Renderer, RVertex, RPipelineId};

pub struct AppEventLoop<'a> {
  renderer: Renderer<'a>,
  rotate: [f32; 3]
}

impl AppEventLoop<'_> {
  pub fn new(window: Arc<Window>) -> Self {
    let wgpu = pollster::block_on(Renderer::new(window.clone()));

    Self {
      renderer: wgpu,
      rotate: [0.0, 0.0, 0.0]
    }
  }

  // initialize app objects
  pub fn init(&mut self) {
    // initialize pipeline
    let shader1 = wgpu::ShaderSource::Wgsl(include_str!("base.wgsl").into());
    let pipe1: RPipelineId = self.renderer.add_pipeline(shader1, 10, None, None);

    let size: f32 = 200.0;
    let verts = vec![
      RVertex { position:[-size, size, 0.0], uv: [0.0, 1.0], normal: [0.0,-1.0,1.0] },
      RVertex { position:[size, size, 0.0], uv: [1.0, 1.0], normal: [0.0,0.0,1.0] },
      RVertex { position:[size, -size, 0.0], uv: [1.0, 0.0], normal: [-1.0,0.0,1.0] },
      RVertex { position:[size, -size, 0.0], uv: [0.0, 1.0], normal: [-1.0,0.0,1.0] },
      RVertex { position:[-size, -size, 0.0], uv: [1.0, 1.0], normal: [0.0,0.0,1.0] },
      RVertex { position:[-size, size, 0.0], uv: [1.0, 0.0], normal: [0.0,-1.0,1.0] },
    ];
    self.renderer.add_object(pipe1, &verts);
  }

  // handle inputs
  pub fn input(&mut self, event: &WindowEvent, request_redraw: &mut bool) -> bool {
    match event {
      WindowEvent::KeyboardInput { 
        event: KeyEvent {
          logical_key: key,
          state: ElementState::Pressed,
          ..
        },
        ..
      } => {
        *request_redraw = true;
        match key.as_ref() {
          Key::Character("w") => {
            self.rotate[0] = self.rotate[0] + 5.0;
            true
          }
          Key::Character("s") => {
            self.rotate[0] = self.rotate[0] - 5.0;
            true
          }
          Key::Character("a") => {
            self.rotate[1] = self.rotate[1] + 5.0;
            true
          }
          Key::Character("d") => {
            self.rotate[1] = self.rotate[1] - 5.0;
            true
          }
          Key::Character("q") => {
            self.rotate[2] = self.rotate[2] - 5.0;
            true
          }
          Key::Character("e") => {
            self.rotate[2] = self.rotate[2] + 5.0;
            true
          }
          _ => {
            let debug = key.as_ref();
            println!("Unhandled key: {debug:?}");
            true
          }
        }
      }
      #[allow(unused_variables)]
      WindowEvent::CursorMoved { device_id, position } => true,
      _ => true,
    }
  }

  // update logic
  pub fn update(&mut self) {
    self.renderer.update_object(0, 0, &[0.0, 0.0, 0.0], &self.rotate, &[1.0, 1.0, 1.0], true);
  }

  // call render
  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
    self.update();
  }
}