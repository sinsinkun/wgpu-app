#![allow(dead_code)]

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
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
pub struct MouseCache {
  left: InputState,
  right: InputState,
  middle: InputState,
  back: InputState,
  forward: InputState,
  scroll: f32, // + for up, - for down
  position_can_update: bool,
  last_position: PhysicalPosition<f64>
}

#[derive(Debug)]
pub struct InputCache {
  pub move_x: f32,
  pub move_y: f32,
  pub move_z: f32,
  retain: bool
}

// middleware for handling inputs
// note: input processing is asynchronous with render loop
#[derive(Debug)]
pub struct InputHandler {
  pub key_binds: Vec<(PhysicalKey, InputAction)>,
  pub mouse_cache: MouseCache,
  pub input_cache: InputCache,
}

impl InputHandler {
  pub fn new() -> Self {
    let key_binds = vec![
      (PhysicalKey::Code(KeyCode::KeyQ), InputAction::Up),
      (PhysicalKey::Code(KeyCode::KeyE), InputAction::Down),
      (PhysicalKey::Code(KeyCode::KeyA), InputAction::Left),
      (PhysicalKey::Code(KeyCode::KeyD), InputAction::Right),
      (PhysicalKey::Code(KeyCode::KeyW), InputAction::Fwd),
      (PhysicalKey::Code(KeyCode::KeyS), InputAction::Bkwd),
    ];
    let mouse_cache = MouseCache {
      left: InputState::None,
      right: InputState::None,
      middle: InputState::None,
      back: InputState::None,
      forward: InputState::None,
      scroll: 0.0,
      position_can_update: true,
      last_position: PhysicalPosition { x: 0.0, y: 0.0 }
    };
    let input_cache = InputCache {
      move_x: 0.0,
      move_y: 0.0,
      move_z: 0.0,
      retain: false,
    };

    InputHandler {
      key_binds,
      mouse_cache,
      input_cache,
    }
  }

  pub fn remap_input(&mut self, action: InputAction, key: PhysicalKey) {
    for (k, a) in &mut self.key_binds {
      if *a == action { *k = key }
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
            if key_state == InputState::Press { self.input_cache.move_y += 5.0 }
            if key_state == InputState::Release { self.input_cache.move_y -= 5.0 }
          }
          InputAction::Down => {
            if key_state == InputState::Press { self.input_cache.move_y += -5.0 }
            if key_state == InputState::Release { self.input_cache.move_y -= -5.0 }
          }
          InputAction::Left => {
            if key_state == InputState::Press { self.input_cache.move_x += -5.0 }
            if key_state == InputState::Release { self.input_cache.move_x -= -5.0 }
          }
          InputAction::Right => {
            if key_state == InputState::Press { self.input_cache.move_x += 5.0 }
            if key_state == InputState::Release { self.input_cache.move_x -= 5.0 }
          }
          InputAction::Fwd => {
            if key_state == InputState::Press { self.input_cache.move_z += -5.0 }
            if key_state == InputState::Release { self.input_cache.move_z -= -5.0 }
          }
          InputAction::Bkwd => {
            if key_state == InputState::Press { self.input_cache.move_z += 5.0 }
            if key_state == InputState::Release { self.input_cache.move_z -= 5.0 }
          }
        }
        self.input_cache.retain = true;
        break
      }
    }
  }

  pub fn winit_mouse_event(&mut self, btn: MouseButton, state: ElementState) {
    match btn {
      MouseButton::Left => {
        if state == ElementState::Pressed { self.mouse_cache.left = InputState::Press }
        else if state == ElementState::Released { self.mouse_cache.left = InputState::Release }
      }
      MouseButton::Right => {
        if state == ElementState::Pressed { self.mouse_cache.right = InputState::Press }
        else if state == ElementState::Released { self.mouse_cache.right = InputState::Release }
      }
      MouseButton::Middle => {
        if state == ElementState::Pressed { self.mouse_cache.middle = InputState::Press }
        else if state == ElementState::Released { self.mouse_cache.middle = InputState::Release }
      }
      MouseButton::Forward => {
        if state == ElementState::Pressed { self.mouse_cache.forward = InputState::Press }
        else if state == ElementState::Released { self.mouse_cache.forward = InputState::Release }
      }
      MouseButton::Back => {
        if state == ElementState::Pressed { self.mouse_cache.back = InputState::Press }
        else if state == ElementState::Released { self.mouse_cache.back = InputState::Release }
      }
      _ => ()
    }
  }

  pub fn winit_mouse_wheel_event(&mut self, delta: MouseScrollDelta) {
    match delta {
      MouseScrollDelta::LineDelta(_x, y) => {
        self.mouse_cache.scroll = y;
        self.input_cache.move_z = -8.0 * y;
        self.input_cache.retain = false;
      }
      _ => ()
    }
  }

  pub fn winit_cursor_event(&mut self, position: PhysicalPosition<f64>) {
    if self.mouse_cache.position_can_update {
      let delta_x: f64 = position.x - self.mouse_cache.last_position.x;
      let delta_y: f64 = position.y - self.mouse_cache.last_position.y;

      if self.mouse_cache.left == InputState::Hold {
        self.input_cache.move_x = -0.4 * delta_x as f32;
        self.input_cache.move_y = 0.4 * delta_y as f32;
        self.input_cache.retain = false;
      }
  
      // update last position
      self.mouse_cache.last_position = position;
      self.mouse_cache.position_can_update = false;
    }
  }

  pub fn cleanup_cache(&mut self) {
    // clean up mouse cache
    if self.mouse_cache.left == InputState::Press { self.mouse_cache.left = InputState::Hold }
    else if self.mouse_cache.left == InputState::Release { self.mouse_cache.left = InputState::None }
    if self.mouse_cache.right == InputState::Press { self.mouse_cache.right = InputState::Hold }
    else if self.mouse_cache.right == InputState::Release { self.mouse_cache.right = InputState::None }
    if self.mouse_cache.middle == InputState::Press { self.mouse_cache.middle = InputState::Hold }
    else if self.mouse_cache.middle == InputState::Release { self.mouse_cache.middle = InputState::None }
    if self.mouse_cache.back == InputState::Press { self.mouse_cache.back = InputState::Hold }
    else if self.mouse_cache.back == InputState::Release { self.mouse_cache.back = InputState::None }
    if self.mouse_cache.forward == InputState::Press { self.mouse_cache.forward = InputState::Hold }
    else if self.mouse_cache.forward == InputState::Release { self.mouse_cache.forward = InputState::None }
    self.mouse_cache.scroll = 0.0;
    self.mouse_cache.position_can_update = true;
    // clean up input cache
    if !self.input_cache.retain {
      self.input_cache.move_x = 0.0;
      self.input_cache.move_y = 0.0;
      self.input_cache.move_z = 0.0;
    }
  }

  pub fn output(&self) -> &InputCache {
    &self.input_cache
  }
}