use std::{fs, time, path::Path};

use crate::wgpu_root::{RCamera, RObjectUpdate, RPipelineId, RPipelineSetup, RTextureId, Renderer};
use crate::primitives::{Primitives, Shape};
use crate::input_mapper::InputHandler;

pub struct AppEventLoop<'a> {
  renderer: Renderer<'a>,
  pub input_handler: InputHandler,
  frame: u32, // max value: ~4,295,000,000
  last_frame_time: time::Instant,
  new_frame_time: time::Instant,
  pipes: Vec<RPipelineId>,
  textures: Vec<RTextureId>,
  shapes: Vec<Shape>,
  camera: RCamera,
  screen_center: (f32, f32),
}

impl<'a> AppEventLoop<'a> {
  pub fn new(wgpu: Renderer<'a>, window_size: &(f32, f32)) -> Self {
    let mut cam = RCamera::new_persp(60.0, 1.0, 1000.0);
    cam.position = [0.0, 0.0, 200.0];
    let input_handler = InputHandler::new();

    Self{
      renderer: wgpu,
      input_handler,
      shapes: vec![],
      frame: 0,
      last_frame_time: time::Instant::now(),
      new_frame_time: time::Instant::now(),
      camera: cam,
      screen_center: (window_size.0 / 2.0, window_size.1 / 2.0),
      pipes: Vec::new(),
      textures: Vec::new(),
    }
  }

  // initialize app objects
  pub fn init(&mut self) {
    // initialize pipeline
    let texture1 = self.renderer.add_texture(1200, 1200, Some(Path::new("assets/test_uv_map.png")), false);
    let texture4 = self.renderer.add_texture(800, 800, None, false);
    let texture2 = self.renderer.add_texture(
      (self.screen_center.0 * 2.0) as u32,
      (self.screen_center.1 * 2.0) as u32,
      None,
      true
    );
    let pipe1 = self.renderer.add_pipeline(RPipelineSetup {
      texture1_id: Some(texture1),
      texture2_id: Some(texture4),
      ..Default::default()
    });
    let pipe2 = match fs::read_to_string("assets/test.wgsl") {
      Ok(str) => { 
        self.renderer.add_pipeline(RPipelineSetup {
          shader: &str,
          max_obj_count: 1,
          texture1_id: Some(texture2),
          ..Default::default()
        })
      }
      Err(..) => {
        println!("Err: Could not find shader");
        self.renderer.add_pipeline(RPipelineSetup {
          max_obj_count: 1, 
          texture1_id: Some(texture2),
          ..Default::default()
        })
      }
    };
    // initialize text pipeline
    // self.renderer.load_font("assets/retro_computer.ttf");
    let (texture3, pipe3) = self.renderer.add_overlay_pipeline();
    self.renderer.render_str_on_texture(texture4, "Wordy", 200.0, [255, 0, 0], [40, 450], 10);

    // initialize objects
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

    // store ids
    self.pipes.push(pipe1);
    self.pipes.push(pipe2);
    self.pipes.push(pipe3);
    self.textures.push(texture1);
    self.textures.push(texture2);
    self.textures.push(texture3);
    self.textures.push(texture4);
    self.shapes.push(cube1);
    self.shapes.push(cube2);
    self.shapes.push(cube3);
    self.shapes.push(rect);
  }

  // update logic (synchronous with render loop)
  pub fn update(&mut self) {
    // logic updates
    let input_cache = self.input_handler.output();
    self.camera.position[0] += input_cache.move_x;
    self.camera.look_at[0] += 0.9 * input_cache.move_x;
    self.camera.position[1] += input_cache.move_y;
    self.camera.look_at[1] += 0.9 * input_cache.move_y;
    self.camera.position[2] += input_cache.move_z;
    
    // render logic updates
    for obj in &mut self.shapes {
      if obj.id.0 == 1 {
        obj.position = [-self.screen_center.0 * 0.75, self.screen_center.1 * 0.75, 0.0];
        obj.scale = [self.screen_center.0, self.screen_center.1, 1.0];
        self.renderer.update_object(RObjectUpdate::from_shape(obj, None));
      } else {
        obj.rotate_deg = self.frame as f32;
        self.renderer.update_object(RObjectUpdate::from_shape(obj, Some(&self.camera)));
      }
    }
  }

  // call render
  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    self.frame += 1;
    self.last_frame_time = self.new_frame_time;
    self.new_frame_time = time::Instant::now();
    let delta_t = self.new_frame_time - self.last_frame_time;
    let fps = (1.0 / delta_t.as_secs_f32()) as u32;
    let fps_txt = "FPS: ".to_owned() + &fps.to_string();
    // find bottom left corner
    let y_max = (self.screen_center.1 * 2.0) as u32;

    // render cubes onto texture
    self.renderer.render_texture(&self.pipes[0..1], self.textures[1], Some([0.0, 0.0, 0.0, 0.0]));
    // render text onto texture
    self.renderer.render_texture(&[], self.textures[2], Some([0.0, 0.0, 0.0, 0.0])); // clears texture background
    self.renderer.render_str_on_texture(self.textures[2], &fps_txt, 20.0, [0, 255, 0], [5, y_max - 10], 1);
    self.renderer.render_str_on_texture(self.textures[2], "Camera controls: WASD, EQ", 18.0, [50, 50, 255], [5, y_max - 30], 1);
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
    self.renderer.update_texture_size(self.textures[1], Some(self.pipes[1]), width, height);
    self.renderer.update_texture_size(self.textures[2], Some(self.pipes[2]), width, height);
    self.update();
  }
}