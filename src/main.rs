#![allow(internal_features)]
#![feature(cold_path)]
#![feature(nonzero_internals)]

pub mod graphics;
pub mod utils;

use std::{num::NonZero, sync::Arc};

use graphics::{
    context::RenderContext,
    core::{
        backend::{Api, RenderDeviceGroup},
        resource::{BufferDesc, BufferUsages, TextureDesc, TextureType, TextureUsages},
        types::Format,
    },
    DebugFlags, RenderBackend, RenderBackendSettings, RenderSystem,
};

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

    let render_system = RenderSystem::new(&[RenderBackendSettings {
        api: RenderBackend::Dx12,
        debug: DebugFlags::all(),
    }]);

    let dx_api = render_system.dx_api().unwrap();

    let gpu1 = Arc::new(dx_api.create_device(0));
    let gpu2 = Arc::new(dx_api.create_device(1));

    let devices = RenderDeviceGroup::new(gpu1, vec![gpu2]);

    let handle = render_system.create_texture_handle();
    devices.call(|d| {
        d.bind_texture(
            handle,
            TextureDesc {
                name: None,
                ty: TextureType::D2,
                width: 1,
                height: 1,
                depth: 1,
                mip_levels: 1,
                format: Format::R32,
                usage: TextureUsages::RenderTarget,
            },
            Some(bytemuck::cast_slice(&[1.0])),
        );

        d.unbind_texture(handle);
    });
    render_system.free_texture_handle(handle);

    let handle = render_system.create_texture_handle();
    devices.primary.bind_texture(
        handle,
        TextureDesc {
            name: None,
            ty: TextureType::D2,
            width: 1280,
            height: 720,
            depth: 1,
            mip_levels: 1,
            format: Format::R32,
            usage: TextureUsages::RenderTarget | TextureUsages::Shared,
        },
        None,
    );
    devices.secondaries[0].open_texture_handle(handle, &devices.primary);

    let data = [0u8, 1, 2, 3, 4, 5, 6, 7];
    let handle = render_system.create_buffer_handle();
    devices.call(|d| {
        d.bind_buffer(
            handle,
            BufferDesc {
                name: None,
                size: size_of_val(&data),
                stride: 0,
                usage: BufferUsages::Vertex,
            },
            Some(&data),
        );

        d.unbind_buffer(handle);
    });
    render_system.free_buffer_handle(handle);

    /*let event_loop = winit::event_loop::EventLoop::new().expect("failed to create event loop");

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = Application { wnd_ctx: None };
    event_loop.run_app(&mut app).expect("failed to run app");*/
}
