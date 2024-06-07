use std::sync::Arc;
use std::thread;
use std::time;

use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, KeyEvent, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{PhysicalKey, KeyCode};
use winit::window::{Window, WindowId, CursorGrabMode};

mod wgpu_root;
mod wgpu_text;
mod primitives;
mod lin_alg;
mod app;
mod input_mapper;

use wgpu_root::Renderer;
use app::AppEventLoop;

// constants
const WAIT_TIME: time::Duration = time::Duration::from_millis(1000);
const POLL_SLEEP_TIME: time::Duration = time::Duration::from_millis(5);

// definitions for winit window
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
	#[default]
	Wait,
	WaitUntil,
	Poll,
}

struct ControlFlowApp<'a> {
	mode: Mode,
	request_redraw: bool,
	wait_cancelled: bool,
	close_requested: bool,
	window: Option<Arc<Window>>,
	app_event_loop: Option<AppEventLoop<'a>>,
	window_size: (f32, f32),
}

impl Default for ControlFlowApp<'_> {
	fn default() -> Self {
		ControlFlowApp {
			mode: Mode::Poll,
			request_redraw: true, // toggle true to refresh by default
			wait_cancelled: false,
			close_requested: false,
			window: None,
			app_event_loop: None,
			window_size: (0.0, 0.0)
		}
	}
}

impl ApplicationHandler for ControlFlowApp<'_> {
	fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
		self.wait_cancelled = match cause {
			StartCause::WaitCancelled { .. } => self.mode == Mode::WaitUntil,
			_ => false,
		}
	}

	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let window_attributes = Window::default_attributes()
			.with_min_inner_size(PhysicalSize::new(400.0, 300.0))
			.with_title("Wgpu-rs");
		let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
		self.window = Some(window.clone());
		self.window_size = window.inner_size().into();

		// divert init actions to app container
		let wgpu = pollster::block_on(Renderer::new(window.clone()));
		let mut app_base = AppEventLoop::new(wgpu, &self.window_size);
		app_base.init();
		self.app_event_loop = Some(app_base);
	}

	fn window_event(&mut self, _event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
		match event {
			WindowEvent::CloseRequested => {
				self.close_requested = true;
			}
			WindowEvent::KeyboardInput { event: KeyEvent { physical_key: key, state, repeat, .. }, .. } => {
				// perform app input handling first
				if let Some(app_base) = &mut self.app_event_loop {
					app_base.input_handler.winit_kb_event(&key, &state, repeat);
					self.request_redraw = true;
				}
				// perform window related input handling
				match key {
					PhysicalKey::Code(KeyCode::F1) => {
						if state == ElementState::Pressed {
							self.mode = Mode::Wait;
							println!("mode: {:?}", self.mode);
						}
					}
					PhysicalKey::Code(KeyCode::F2) => {
						if state == ElementState::Pressed {
							self.mode = Mode::WaitUntil;
							println!("mode: {:?}", self.mode);
						}
					}
					PhysicalKey::Code(KeyCode::F3) => {
						if state == ElementState::Pressed {
							self.mode = Mode::Poll;
							println!("mode: {:?}", self.mode);
						}
					}
					PhysicalKey::Code(KeyCode::Space) => {
						if state == ElementState::Pressed {
							self.request_redraw = !self.request_redraw;
							// println!("request_redraw: {}", self.request_redraw);
						}
					}
					PhysicalKey::Code(KeyCode::Escape) => {
						if state == ElementState::Pressed {
							self.close_requested = true;
						}
					}
					PhysicalKey::Code(KeyCode::AltLeft) => {
						if let Some(win) = &self.window {
							let x = self.window_size.0 / 2.0;
							let y = self.window_size.1 / 2.0;
							if state == ElementState::Pressed && !repeat {
								println!("lock cursor");
								win.set_cursor_grab(CursorGrabMode::Confined).unwrap();
								win.set_cursor_position(PhysicalPosition{ x, y }).unwrap();
								// win.set_cursor_visible(false);
							} else if state == ElementState::Released {
								println!("unlock cursor");
								win.set_cursor_grab(CursorGrabMode::None).unwrap();
								// win.set_cursor_visible(true);
							} else {
								win.set_cursor_position(PhysicalPosition{ x, y }).unwrap();
							}
						}
					}
					_ => ()
				}
			}
			WindowEvent::CursorMoved { position, .. } => {
				// perform app input handling
				if let Some(app_base) = &mut self.app_event_loop {
					app_base.input_handler.winit_cursor_event(position);
					self.request_redraw = true;
				}
			}
			WindowEvent::MouseInput { state, button, .. } => {
				// perform app input handling
				if let Some(app_base) = &mut self.app_event_loop {
					app_base.input_handler.winit_mouse_event(button, state);
					self.request_redraw = true;
				}
			}
			WindowEvent::MouseWheel { delta, .. } => {
				// perform app input handling
				if let Some(app_base) = &mut self.app_event_loop {
					app_base.input_handler.winit_mouse_wheel_event(delta);
					self.request_redraw = true;
				}
			}
			WindowEvent::CursorEntered {..} => {
				// todo
			}
			WindowEvent::CursorLeft {..} => {
				// todo
			}
			WindowEvent::RedrawRequested => {
				let window = self.window.as_ref().unwrap();
				if let Some(app_base) = &mut self.app_event_loop {
					window.pre_present_notify();
					match app_base.render() {
						Ok(_) => (),
						// pass out-of-memory error out to winit
						Err(wgpu::SurfaceError::OutOfMemory) => self.close_requested = true,
						Err(e) => eprintln!("{:?}", e),
					}
				}
			}
			WindowEvent::Resized(physical_size) => {
				if let Some(app_base) = &mut self.app_event_loop {
					app_base.resize(physical_size.width, physical_size.height);
					self.window_size = physical_size.into();
				}
			}
			_ => (),
		}
	}

	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		if self.request_redraw && !self.wait_cancelled && !self.close_requested {
			if let Some(app_base) = &mut self.app_event_loop {
				app_base.update();
				app_base.input_handler.cleanup_cache();
			}
			self.window.as_ref().unwrap().request_redraw();
		}

		match self.mode {
			Mode::Wait => {
				event_loop.set_control_flow(ControlFlow::Wait);
				self.request_redraw = false;
			}
			Mode::WaitUntil => {
				if !self.wait_cancelled {
					event_loop.set_control_flow(
						ControlFlow::WaitUntil(time::Instant::now() + WAIT_TIME)
					);
				}
			}
			Mode::Poll => {
				thread::sleep(POLL_SLEEP_TIME);
				event_loop.set_control_flow(ControlFlow::Poll);
			}
		};

		if self.close_requested {
			event_loop.exit();
		}
	}

	fn suspended(&mut self, event_loop: &ActiveEventLoop) {
		println!("Suspended window");
		let _ = event_loop;
	}
}

// entry point
pub fn main() {
	env_logger::init();
	let event_loop = EventLoop::new().unwrap();
	let mut app = ControlFlowApp::default();
	let _ = event_loop.run_app(&mut app);
}
