#![allow(internal_features)]
#![feature(cold_path)]
#![feature(nonzero_internals)]

pub mod graphics;
pub mod utils;

use std::num::NonZero;

use tracing_subscriber::layer::SubscriberExt;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

pub struct WindowContext {
    pub window: winit::window::Window,
    pub hwnd: NonZero<isize>,
}

pub struct Application {
    pub wnd_ctx: Option<WindowContext>,
}

impl winit::application::ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = winit::window::Window::default_attributes()
            .with_title("Hello, Window!")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600));

        let window = event_loop.create_window(window_attributes).unwrap();
        window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .expect("Failet to lock cursor");
        window.set_cursor_visible(false);

        let Ok(RawWindowHandle::Win32(hwnd)) = window.window_handle().map(|h| h.as_raw()) else {
            unreachable!()
        };
        let hwnd = hwnd.hwnd;

        self.wnd_ctx = Some(WindowContext { window, hwnd });
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::Focused(_) => {}
            winit::event::WindowEvent::KeyboardInput { event, .. } => match event.state {
                winit::event::ElementState::Pressed => {
                    if event.physical_key == winit::keyboard::KeyCode::Escape {
                        event_loop.exit();
                    }
                }
                winit::event::ElementState::Released => {}
            },
            winit::event::WindowEvent::MouseInput { state, .. } => match state {
                winit::event::ElementState::Pressed => {}
                winit::event::ElementState::Released => {}
            },
            winit::event::WindowEvent::Resized(_size) => {}
            winit::event::WindowEvent::RedrawRequested => {}
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            _ => (),
        }
    }

    #[allow(clippy::single_match)]
    fn device_event(
        &mut self,
        _: &winit::event_loop::ActiveEventLoop,
        _: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        match event {
            winit::event::DeviceEvent::MouseMotion { .. } => {}
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &winit::event_loop::ActiveEventLoop) {
        if let Some(context) = self.wnd_ctx.as_ref() {
            context.window.request_redraw();
        }
    }
}

fn main() {
    let console_log = tracing_subscriber::fmt::Layer::new()
        .with_ansi(true)
        .with_writer(std::io::stdout);
    let subscriber = tracing_subscriber::registry().with(console_log);
    let _ = tracing::subscriber::set_global_default(subscriber);

    let event_loop = winit::event_loop::EventLoop::new().expect("failed to create event loop");

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = Application { wnd_ctx: None };
    event_loop.run_app(&mut app).expect("failed to run app");
}
