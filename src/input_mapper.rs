#![allow(dead_code)]

use winit::event::ElementState;
use winit::keyboard::{PhysicalKey, KeyCode};

#[derive(Debug, Clone, PartialEq)]
pub enum InputState {
  None, Press, Hold, Release
}

#[derive(Debug, PartialEq)]
pub enum InputAction {
  Up, Down, Left, Right, Fwd, Bkwd,
}

#[derive(Debug)]
pub struct InputCache {
  pub move_x: i32,
  pub move_y: i32,
  pub move_z: i32,
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

// middleware for handling inputs
// note: input processing is asynchronous with render loop
#[derive(Debug)]
pub struct InputHandler {
  pub key_binds: Vec<(PhysicalKey, InputAction)>,
  pub input_cache: InputCache,
}

impl InputHandler {
  pub fn new() -> Self {
    let key_binds = vec![
      (PhysicalKey::Code(KeyCode::KeyW), InputAction::Up),
      (PhysicalKey::Code(KeyCode::KeyS), InputAction::Down),
      (PhysicalKey::Code(KeyCode::KeyA), InputAction::Left),
      (PhysicalKey::Code(KeyCode::KeyD), InputAction::Right),
      (PhysicalKey::Code(KeyCode::KeyE), InputAction::Fwd),
      (PhysicalKey::Code(KeyCode::KeyQ), InputAction::Bkwd),
    ];

    InputHandler {
      key_binds,
      input_cache: InputCache::default(),
    }
  }

  pub fn winit_kb_event(&mut self, key: &PhysicalKey, state: &ElementState, repeat: bool) {
    let mut key_state = InputState::None;
    if state == &ElementState::Pressed && !repeat { key_state = InputState::Press }
    else if repeat { key_state = InputState::Hold }
    else if state == &ElementState::Released { key_state = InputState::Release };

    for (k, a) in &self.key_binds {
      if key == k {
        match a {
          InputAction::Up => {
            if key_state == InputState::Press { self.input_cache.move_y += 1 }
            if key_state == InputState::Release { self.input_cache.move_y -= 1 }
          }
          InputAction::Down => {
            if key_state == InputState::Press { self.input_cache.move_y += -1 }
            if key_state == InputState::Release { self.input_cache.move_y -= -1 }
          }
          InputAction::Left => {
            if key_state == InputState::Press { self.input_cache.move_x += -1 }
            if key_state == InputState::Release { self.input_cache.move_x -= -1 }
          }
          InputAction::Right => {
            if key_state == InputState::Press { self.input_cache.move_x += 1 }
            if key_state == InputState::Release { self.input_cache.move_x -= 1 }
          }
          InputAction::Fwd => {
            if key_state == InputState::Press { self.input_cache.move_z += -1 }
            if key_state == InputState::Release { self.input_cache.move_z -= -1 }
          }
          InputAction::Bkwd => {
            if key_state == InputState::Press { self.input_cache.move_z += 1 }
            if key_state == InputState::Release { self.input_cache.move_z -= 1 }
          }
        }
        break
      }
    }
  }

  pub fn winit_mouse_event(&mut self) {
    todo!()
  }

  pub fn remap_input() {
    todo!()
  }

  pub fn get_cache(&self) -> &InputCache {
    &self.input_cache
  }
}