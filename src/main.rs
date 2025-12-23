// mod cpu;
// mod memory;
// mod ppu;
// use std::{cell::RefCell, rc::Rc};

// use cpu::olc6502::Olc6502;

// use pixels::{Pixels, SurfaceTexture};
// use winit::{
//     dpi::LogicalSize,
//     event::{Event, WindowEvent},
//     event_loop::{ControlFlow, EventLoop},
//     window::WindowBuilder,
// };

// const WIDTH: u32 = 256;
// const HEIGHT: u32 = 240;

// fn main() {
//     let mut bus = memory::bus::Bus::new();

//     let rom_content = std::fs::read("roms/donkey kong.nes").expect("Failed to read ROM file");
//     let mapper = memory::mapper::parse_rom(rom_content);

//     bus.set_mapper(Rc::new(RefCell::new(mapper)));

//     let mut cpu = Olc6502::new(bus);

//     cpu.reset();

//        let event_loop = EventLoop::new();

//     let window = WindowBuilder::new()
//         .with_title("NES framebuffer")
//         .with_inner_size(LogicalSize::new(WIDTH * 2, HEIGHT * 2))
//         .build(&event_loop).unwrap();

//     let surface_texture =
//         SurfaceTexture::new(WIDTH * 2, HEIGHT * 2, &window);
//     let mut pixels = Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap();


//     event_loop.run(move |event, _, mut control_flow| {
//         *control_flow = ControlFlow::Poll;

//         match event {
//             Event::MainEventsCleared => {
//                 while !cpu.bus.ppu.frame_ready() {
//                     cpu.tick();
//                 }
//                 window.request_redraw();
//             }

//             Event::WindowEvent { event, .. } => match event {
//                 WindowEvent::CloseRequested => {
//                     *control_flow = ControlFlow::Exit;
//                 }
//                 WindowEvent::Resized(size) => {
//                     pixels.resize_surface(size.width, size.height).unwrap();
//                 }
//                 _ => {}
//             },

//             Event::RedrawRequested(_) => {
//                 let frame = pixels.frame_mut();

//                 let framebuffer = cpu.bus.ppu.get_pixel_buffer();
//                 frame.copy_from_slice(framebuffer);
//                 if pixels.render().is_err() {
//                     *control_flow = ControlFlow::Exit;
//                 }
//             }

//             _ => {}
//         }
//     });
// }
mod cpu;
mod memory;
mod ppu;

use std::{cell::RefCell, rc::Rc};

use cpu::olc6502::Olc6502;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler, dpi::LogicalSize, event::WindowEvent, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, window::WindowId
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
            .with_title("NES framebuffer")
            .with_inner_size(LogicalSize::new(WIDTH * 2, HEIGHT * 2));

        let window = event_loop.create_window(attrs).unwrap();
        let window_id = window.id();
        self.window_id = Some(window_id);

        let surface_texture = SurfaceTexture::new(WIDTH * 2, HEIGHT * 2, window);
        let pixels = Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap();
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

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.window_id.is_some() {
            self.tick_frame();
            self.redraw();
        }
    }
}

fn main() {
    let mut bus = memory::bus::Bus::new();
    let rom_content = std::fs::read("roms/donkey kong.nes").unwrap();
    let mapper = memory::mapper::parse_rom(rom_content);
    bus.set_mapper(Rc::new(RefCell::new(mapper)));

    let mut cpu = Olc6502::new(bus);
    cpu.reset();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = NesApp::new(cpu);

    event_loop.run_app(&mut app).unwrap();
}