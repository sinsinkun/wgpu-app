use std::{fs, time, path::Path};
use rand::{thread_rng, Rng};

use crate::wgpu_renderer::{ModelLoader, Primitives, RCamera, RObjectUpdate, RPipelineId, RPipelineSetup, RTextureId, RUniformSetup, Renderer, Shape};
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
    cam.position = [0.0, 0.0, 200.0];
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
    // initialize pipeline for objects
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
      max_obj_count: 1000,
      cull_mode: RPipelineSetup::CULL_MODE_BACK,
      ..Default::default()
    });
    // pipeline for miniview
    let pipe2 = match fs::read_to_string("assets/miniview.wgsl") {
      Ok(str) => { 
        self.renderer.add_pipeline(RPipelineSetup {
          shader: &str,
          max_obj_count: 1,
          texture1_id: Some(texture2),
          uniforms: vec![
            RUniformSetup {
              bind_slot: 0,
              visibility: RUniformSetup::VISIBILITY_FRAGMENT,
              size_in_bytes: 8
            }
          ],
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
    self.renderer.load_font("assets/retro_computer.ttf");
    let (texture3, pipe3) = self.renderer.add_overlay_pipeline();
    self.renderer.render_str_on_texture(texture4, "Marked", 200.0, [255, 0, 0], [40, 450], 10);

    // pipeline for 3d model
    let pipe4 = self.renderer.add_pipeline(RPipelineSetup {
      max_obj_count: 10,
      cull_mode: RPipelineSetup::CULL_MODE_BACK,
      ..Default::default()
    });

    // initialize objects
    let (cube_data, cube_idx) = Primitives::hemisphere(20.0, 32, 16);
    for x in 0..10 {
      for y in 0..10 {
        for z in 0..5 {
          let rx: f32 = thread_rng().gen_range(-1.0..1.0);
          let ry: f32 = thread_rng().gen_range(-1.0..1.0);
          let rz: f32 = thread_rng().gen_range(-1.0..1.0);
          let s: f32 = thread_rng().gen_range(0.5..1.2);
          let mut cube = Shape::new(&mut self.renderer, pipe1, cube_data.clone(), Some(cube_idx.clone()));
          cube.position = [
            -270.0 + x as f32 * 60.0 + rx * 20.0,
            -270.0 + y as f32 * 60.0 + ry * 20.0,
            z as f32 * 60.0 + rz * 20.0
          ];
          cube.rotate_axis = [rx, ry, rz];
          cube.scale = [s, s, s];
          self.shapes.push(cube);
        }
      }
    }

    match ModelLoader::load_obj("assets/monkey.obj") {
      Ok(model) => {
        let mut shape = Shape::new(&mut self.renderer, pipe4, model, None);
        shape.rotate_axis = [0.0, 1.0, 0.0];
        shape.scale = [30.0, 30.0, 30.0];
        self.shapes.push(shape);
      }
      Err(e) => {
        println!("Could not load model {:?}", e);
      }
    };

    let (rect_data, rect_i) = Primitives::rect_indexed(0.5, 0.5, 0.0);
    let rect = Shape::new(&mut self.renderer, pipe2, rect_data, Some(rect_i));
    self.shapes.push(rect);

    // store ids
    self.pipes.push(pipe1);
    self.pipes.push(pipe2);
    self.pipes.push(pipe3);
    self.pipes.push(pipe4);
    self.textures.push(texture1);
    self.textures.push(texture2);
    self.textures.push(texture3);
    self.textures.push(texture4);
  }

  // update logic (asynchronous with render loop)
  pub fn update(&mut self) {
    // logic updates
    let input_cache = self.input_handler.output();
    self.camera.position[0] += input_cache.move_x;
    self.camera.look_at[0] += 0.9 * input_cache.move_x;
    self.camera.position[1] += input_cache.move_y;
    self.camera.look_at[1] += 0.9 * input_cache.move_y;
    self.camera.position[2] += input_cache.move_z;
  }

  // render logic updates (synchronous with render loop)
  pub fn pre_render(&mut self, frame_time: &time::Duration) {
    self.render_frame += 1;
    // render logic updates
    for obj in &mut self.shapes {
      if obj.id.0 == 1 {
        obj.position = [-self.screen_center.0 * 0.75, self.screen_center.1 * 0.75, 0.0];
        obj.scale = [self.screen_center.0, self.screen_center.1, 1.0];
        let win_size = vec![self.screen_center.0, self.screen_center.1];
        self.renderer.update_object(RObjectUpdate::from_shape(obj).with_uniforms(vec![bytemuck::cast_slice(&win_size)]));
      } else {
        obj.rotate_deg = self.render_frame as f32;
        self.renderer.update_object(RObjectUpdate::from_shape(obj).with_camera(&self.camera));
      }
    }

    // generate fps text
    let fps = (1.0 / frame_time.as_secs_f32()) as u32;
    let fps_txt = "FPS: ".to_owned() + &fps.to_string();
    // find bottom left corner
    let y_max = (self.screen_center.1 * 2.0) as u32;

    // render cubes onto texture
    self.renderer.render_texture(&self.pipes[0..1], self.textures[1], Some([0.1, 0.0, 0.3, 1.0]));
    // render text onto texture
    self.renderer.render_texture(&[], self.textures[2], Some([0.0, 0.0, 0.0, 0.0])); // clears existing text texture
    self.renderer.render_str_on_texture(self.textures[2], &fps_txt, 20.0, [0, 255, 0], [5, y_max - 10], 1);
    self.renderer.render_str_on_texture(self.textures[2], "Camera controls: WASD, EQ", 18.0, [50, 50, 255], [5, y_max - 30], 1);
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
    self.renderer.update_texture_size(self.textures[1], Some(self.pipes[1]), width, height);
    self.renderer.update_texture_size(self.textures[2], Some(self.pipes[2]), width, height);
    self.update();
  }
}