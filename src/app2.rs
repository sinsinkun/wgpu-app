use std::{fs, time};

use crate::wgpu_renderer::{RCamera, RObjectUpdate, RPipelineId, RPipelineSetup, RTextureId, RVertexAnim, Renderer, Shape};
use crate::input_mapper::InputHandler;

pub struct AppEventLoop<'a> {
  renderer: Renderer<'a>,
  pub input_handler: InputHandler,
  render_frame: u32, // max value: ~4,295,000,000
  pipes: Vec<RPipelineId>,
  textures: Vec<RTextureId>,
  shapes: Vec<Shape>,
  camera: RCamera,
  screen_center: (f32, f32),
}

impl<'a> AppEventLoop<'a> {
  pub fn new(wgpu: Renderer<'a>, window_size: &(f32, f32)) -> Self {
    let mut cam = RCamera::new_persp(60.0, 1.0, 1000.0);
    cam.position = [0.0, 0.0, 10.0];
    let input_handler = InputHandler::new();

    Self{
      renderer: wgpu,
      input_handler,
      shapes: vec![],
      render_frame: 0,
      camera: cam,
      screen_center: (window_size.0 / 2.0, window_size.1 / 2.0),
      pipes: Vec::new(),
      textures: Vec::new(),
    }
  }

  // initialize app objects
  pub fn init(&mut self) {
    // initialize text pipeline
    let (texture0, pipe0) = self.renderer.add_overlay_pipeline();
    // initialize anim pipeline
    let pipe1 = match fs::read_to_string("assets/animated.wgsl") {
      Ok(str) => {
        self.renderer.add_pipeline(RPipelineSetup {
          shader: &str,
          max_obj_count: 10,
          vertex_type: RPipelineSetup::VERTEX_TYPE_ANIM,
          max_joints_count: 1,
          ..Default::default()
        })
      }
      Err(e) => {
        println!("ERR: shader load error - {}", e.to_string());
        self.renderer.add_pipeline(RPipelineSetup {
          max_obj_count: 10,
          vertex_type: RPipelineSetup::VERTEX_TYPE_ANIM,
          max_joints_count: 1,
          ..Default::default()
        })
      }
    };

    // initialize anim object
    let obj_data: Vec<RVertexAnim> = vec![
      RVertexAnim {
        position: [-1.0, 1.0, 0.0], uv: [0.0, 1.0], normal: [0.0, 0.0, 1.0],
        joint_ids: [0, 0, 0, 0], joint_weights: [0.0, 0.0, 0.0, 0.0]
      },
      RVertexAnim {
        position: [-1.0, -1.0, 0.0], uv: [0.0, 0.0], normal: [0.0, 0.0, 1.0],
        joint_ids: [0, 0, 0, 0], joint_weights: [0.5, 0.0, 0.0, 0.0]
      },
      RVertexAnim {
        position: [1.0, 1.0, 0.0], uv: [1.0, 1.0], normal: [0.0, 0.0, 1.0],
        joint_ids: [0, 0, 0, 0], joint_weights: [1.0, 0.0, 0.0, 0.0]
      },
      RVertexAnim {
        position: [-1.0, -1.0, 0.0], uv: [0.0, 0.0], normal: [0.0, 0.0, 1.0],
        joint_ids: [0, 0, 0, 0], joint_weights: [0.5, 0.0, 0.0, 0.0]
      },
      RVertexAnim {
        position: [1.0, -1.0, 0.0], uv: [1.0, 0.0], normal: [0.0, 0.0, 1.0],
        joint_ids: [0, 0, 0, 0], joint_weights: [0.0, 0.0, 0.0, 0.0]
      },
      RVertexAnim {
        position: [1.0, 1.0, 0.0], uv: [1.0, 1.0], normal: [0.0, 0.0, 1.0],
        joint_ids: [0, 0, 0, 0], joint_weights: [1.0, 0.0, 0.0, 0.0]
      },
    ];
    let rect = Shape::new_anim(&mut self.renderer, pipe1, obj_data, None);
    self.shapes.push(rect);

    // store ids
    self.pipes.push(pipe0);
    self.pipes.push(pipe1);
    self.textures.push(texture0);
  }

  // update logic (asynchronous with render loop)
  pub fn update(&mut self) {
    // logic updates
    let input_cache = self.input_handler.output();
    self.camera.position[0] += 0.1 * input_cache.move_x;
    self.camera.look_at[0] += 0.09 * input_cache.move_x;
    self.camera.position[1] += 0.1 * input_cache.move_y;
    self.camera.look_at[1] += 0.09 * input_cache.move_y;
    self.camera.position[2] += 0.1 * input_cache.move_z;
  }

  // render logic updates (synchronous with render loop)
  pub fn pre_render(&mut self, frame_time: &time::Duration) {
    self.render_frame += 1;
    // render logic updates
    let x = f32::sin(0.02 * self.render_frame as f32);
    let y = f32::cos(0.02 * self.render_frame as f32);
    let transforms: Vec<[f32; 16]> = vec![
      [
        x, 0.0, 0.0, 0.0,
        0.0, y, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
      ]
    ];
    for obj in &mut self.shapes {
      self.renderer.update_object(RObjectUpdate::from_shape(obj)
        .with_camera(&self.camera)
        .with_anim(transforms.clone())
      );
    }

    // generate fps text
    let fps = (1.0 / frame_time.as_secs_f32()) as u32;
    let fps_txt = "FPS: ".to_owned() + &fps.to_string();
    // find bottom left corner
    let y_max = (self.screen_center.1 * 2.0) as u32;
    // render overlay text
    self.renderer.render_texture(&[], self.textures[0], Some([0.0, 0.0, 0.0, 0.0])); // clears texture background
    self.renderer.render_str_on_texture(self.textures[0], &fps_txt, 20.0, [0, 255, 0], [5, y_max - 10], 1);
  }

  // render to screen (can cause frame limiting from requesting screen surface)
  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    // render everything to screen
    match self.renderer.render(&self.pipes) {
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
    self.renderer.update_texture_size(self.textures[0], Some(self.pipes[0]), width, height);
    self.update();
  }
}