mod cpu;
mod joypad;
mod memory;
mod ppu;

use std::{cell::RefCell, rc::Rc};

use cpu::olc6502::Olc6502;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture, wgpu::RequestAdapterOptions};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{DeviceEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowId,
};

const WIDTH: u32 = 256;
const HEIGHT: u32 = 240;

struct NesApp<'a> {
    window_id: Option<WindowId>,
    pixels: Option<Pixels<'a>>,
    cpu: Olc6502,
}

impl<'a> NesApp<'a> {
    fn new(cpu: Olc6502) -> Self {
        Self {
            window_id: None,
            pixels: None,
            cpu,
        }
    }

    fn initialize_window(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = winit::window::Window::default_attributes()
            .with_title("Simpleness")
            .with_inner_size(LogicalSize::new(WIDTH * 2, HEIGHT * 2));

        let window = event_loop.create_window(attrs).unwrap();
        let window_id = window.id();
        self.window_id = Some(window_id);

        let surface_texture = SurfaceTexture::new(WIDTH * 2, HEIGHT * 2, window);
        let pixels = PixelsBuilder::new(WIDTH, HEIGHT, surface_texture)
            .request_adapter_options(RequestAdapterOptions {
                power_preference: pixels::wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .enable_vsync(true)
            .build()
            .unwrap();
        self.pixels = Some(pixels);
    }

    fn tick_frame(&mut self) {
        while !self.cpu.bus.ppu.frame_ready() {
            self.cpu.tick();
        }
    }

    fn redraw(&mut self) {
        if let Some(pixels) = &mut self.pixels {
            let frame = pixels.frame_mut();
            let framebuffer = self.cpu.bus.ppu.get_pixel_buffer();
            frame.copy_from_slice(framebuffer);

            pixels.render().unwrap();
        }
    }
}

impl<'a> ApplicationHandler<()> for NesApp<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create the window on first resume
        if self.window_id.is_none() {
            self.initialize_window(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if Some(window_id) == self.window_id {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(size) => {
                    if let Some(p) = &mut self.pixels {
                        p.resize_surface(size.width, size.height).unwrap();
                    }
                }
                _ => {}
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        match event {
            DeviceEvent::Key(key_event) => {
                if let PhysicalKey::Code(code) = key_event.physical_key {
                    let joypad1 = &mut self.cpu.bus.joypad1;
                    let state = if key_event.state.is_pressed() { 1 } else { 0 };
                    match code {
                        KeyCode::ArrowUp => joypad1.state.set_up(state),
                        KeyCode::ArrowDown => joypad1.state.set_down(state),
                        KeyCode::ArrowLeft => joypad1.state.set_left(state),
                        KeyCode::ArrowRight => joypad1.state.set_right(state),
                        KeyCode::KeyX => joypad1.state.set_a(state),
                        KeyCode::KeyZ => joypad1.state.set_b(state),
                        KeyCode::ShiftLeft | KeyCode::ShiftRight => joypad1.state.set_select(state),
                        KeyCode::Enter => joypad1.state.set_start(state),
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.window_id.is_some() {
            self.tick_frame();
            self.redraw();
        }
    }
}

fn main() {
    let mut bus = memory::bus::Bus::new();
    let rom_content = std::fs::read(r"roms\donkey kong.nes").unwrap();
    let mapper = memory::mapper::parse_rom(rom_content);
    bus.set_mapper(Rc::new(RefCell::new(mapper)));

    let mut cpu = Olc6502::new(bus);
    cpu.reset();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = NesApp::new(cpu);

    event_loop.run_app(&mut app).unwrap();
}
