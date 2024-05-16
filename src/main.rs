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
use wgpu_root::{RPipelineId, RVertex};

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

struct ControlFlowApp<'a> {
	mode: Mode,
	request_redraw: bool,
	wait_cancelled: bool,
	close_requested: bool,
	window: Option<Arc<Window>>,
	wgpu: Option<wgpu_root::Renderer<'a>>,
}

impl Default for ControlFlowApp<'_> {
	fn default() -> Self {
		ControlFlowApp {
			mode: Mode::Wait,
			request_redraw: false, // toggle true to refresh by default
			wait_cancelled: false,
			close_requested: false,
			window: None,
			wgpu: None
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

		let state = pollster::block_on(wgpu_root::Renderer::new(window.clone()));
		self.wgpu = Some(state);
		
		// init stuff
		if let Some(wgpu) = &mut self.wgpu {
			let shader1 = wgpu::ShaderSource::Wgsl(include_str!("base.wgsl").into());
			let pipe1: RPipelineId = wgpu.add_pipeline(shader1, 10, None, None);

			let verts = vec![
				RVertex { position:[0.0, 50.0, 0.0], uv: [1.0, 0.0], normal: [0.0,0.0,1.0] },
				RVertex { position:[50.0, 50.0, 0.0], uv: [1.0, 0.0], normal: [0.0,0.0,1.0] },
				RVertex { position:[50.0, 0.0, 0.0], uv: [1.0, 0.0], normal: [0.0,0.0,1.0] },
			];

			// declare vertex
			wgpu.add_object(pipe1, &verts);
		}
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
							Key::Named(NamedKey::Space) => {
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
							match wgpu.render(&[], None) {
								Ok(_) => (),
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
