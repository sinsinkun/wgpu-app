use std::sync::Arc;
use std::thread;
use std::time;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

mod wgpu_root;
mod primitives;
mod lin_alg;
mod app;
use app::AppEventLoop;

// constants
const WAIT_TIME: time::Duration = time::Duration::from_millis(1000);
const POLL_SLEEP_TIME: time::Duration = time::Duration::from_millis(20);

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
}

impl Default for ControlFlowApp<'_> {
	fn default() -> Self {
		ControlFlowApp {
			mode: Mode::Wait,
			request_redraw: false, // toggle true to refresh by default
			wait_cancelled: false,
			close_requested: false,
			window: None,
			app_event_loop: None,
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

		// divert init actions to app container
		let mut app_base = AppEventLoop::new(window.clone());
		app_base.init();
		self.app_event_loop = Some(app_base);
	}

	fn window_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		_window_id: WindowId,
		event: WindowEvent,
	) {
		if let Some(app_base) = &mut self.app_event_loop {
			if app_base.input(&event, &mut self.request_redraw) {
				match event {
					WindowEvent::CloseRequested => {
						self.close_requested = true;
					}
					WindowEvent::KeyboardInput {
						event: KeyEvent {
							logical_key: key,
							state: ElementState::Pressed,
							..
						},
						..
					} => {
						match key.as_ref() {
							// WARNING: Consider using `key_without_modifiers()` if available on your platform.
							// See the `key_binding` example
							Key::Named(NamedKey::F1) => {
								self.mode = Mode::Wait;
								println!("mode: {:?}", self.mode);
							},
							Key::Named(NamedKey::F2) => {
								self.mode = Mode::WaitUntil;
								println!("mode: {:?}", self.mode);
							},
							Key::Named(NamedKey::F3) => {
								self.mode = Mode::Poll;
								println!("mode: {:?}", self.mode);
							},
							Key::Named(NamedKey::Space) => {
								self.request_redraw = !self.request_redraw;
								println!("request_redraw: {}", self.request_redraw);
							},
							Key::Named(NamedKey::Escape) => {
								self.close_requested = true;
							}
							_ => (),
						}
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
							app_base.resize(physical_size);
						}
					}
					_ => (),
				}
			}
		}
	}

	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		if self.request_redraw && !self.wait_cancelled && !self.close_requested {
			if let Some(app_base) = &mut self.app_event_loop {
				app_base.update();
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
