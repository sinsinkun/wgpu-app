use std::sync::Arc;
use std::thread;
use std::time;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

mod wgpu_root;

// constants
const WAIT_TIME: time::Duration = time::Duration::from_millis(20);
const POLL_SLEEP_TIME: time::Duration = time::Duration::from_millis(100);

// definitions for winit window
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
	#[default]
	Wait,
	WaitUntil,
	Poll,
}

#[derive(Default)]
struct ControlFlowApp<'a> {
	mode: Mode,
	request_redraw: bool,
	wait_cancelled: bool,
	close_requested: bool,
	window: Option<Arc<Window>>,
	wgpu: Option<wgpu_root::State<'a>>,
}

impl ApplicationHandler for ControlFlowApp<'_> {
	fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
		self.wait_cancelled = match cause {
			StartCause::WaitCancelled { .. } => self.mode == Mode::WaitUntil,
			_ => false,
		}
	}

	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let window_attributes = Window::default_attributes().with_title("Wgpu-rs");
		let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
		self.window = Some(window.clone());

		let state = pollster::block_on(wgpu_root::State::new(window.clone()));
		self.wgpu = Some(state);
	}

	fn window_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		_window_id: WindowId,
		event: WindowEvent,
	) {
		if let Some(wgpu) = &mut self.wgpu {
			if wgpu.input(&event) {
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
							Key::Named(NamedKey::F4) => {
								self.request_redraw = !self.request_redraw;
								println!("request_redraw: {}", self.request_redraw);
							},
							Key::Named(NamedKey::Escape) => {
								self.close_requested = true;
							}
							_ => (),
						}
					},
					WindowEvent::RedrawRequested => {
						let window = self.window.as_ref().unwrap();
						if let Some(wgpu) = &mut self.wgpu {
							wgpu.update();
							window.pre_present_notify();
							match wgpu.render() {
								Ok(_) => {},
								// Reconfigure the surface if lost
								Err(wgpu::SurfaceError::Lost) => wgpu.resize(wgpu.size),
								// The system is out of memory, we should probably quit
								Err(wgpu::SurfaceError::OutOfMemory) => self.close_requested = true,
								// All other errors (Outdated, Timeout) should be resolved by the next frame
								Err(e) => eprintln!("{:?}", e),
							}
						}
					}
					WindowEvent::Resized(physical_size) => {
						if let Some(wgpu) = &mut self.wgpu {
							wgpu.resize(physical_size);
						}
					}
					_ => (),
				}
			}
		}
	}

	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		if self.request_redraw && !self.wait_cancelled && !self.close_requested {
			self.window.as_ref().unwrap().request_redraw();
		}

		match self.mode {
			Mode::Wait => event_loop.set_control_flow(ControlFlow::Wait),
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
}

// entry point
pub fn main() {
	env_logger::init();
	let event_loop = EventLoop::new().unwrap();
	let mut app = ControlFlowApp::default();
	let _ = event_loop.run_app(&mut app);
}
