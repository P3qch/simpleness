mod cpu;
mod memory;
mod ppu;
use std::{cell::RefCell, rc::Rc};

use cpu::olc6502::Olc6502;


// fn main() {
//     let mut bus = memory::bus::Bus::new();

//     let rom_content = std::fs::read("roms/nestest.nes").expect("Failed to read ROM file");
//     let mapper = memory::mapper::parse_rom(rom_content);

//     bus.set_mapper(Rc::new(RefCell::new(mapper)));

//     let mut cpu = Olc6502::new(bus);

//     cpu.reset();

//     loop {
//         // Emulation loop

//         cpu.tick();
//     }
// }

use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WIDTH: u32 = 256;
const HEIGHT: u32 = 240;

fn main() {
    let mut bus = memory::bus::Bus::new();

    let rom_content = std::fs::read("roms/donkey kong.nes").expect("Failed to read ROM file");
    let mapper = memory::mapper::parse_rom(rom_content);

    bus.set_mapper(Rc::new(RefCell::new(mapper)));

    let mut cpu = Olc6502::new(bus);

    cpu.reset();

       let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("NES framebuffer")
        .with_inner_size(LogicalSize::new(WIDTH * 2, HEIGHT * 2))
        .build(&event_loop).unwrap();

    let surface_texture =
        SurfaceTexture::new(WIDTH * 2, HEIGHT * 2, &window);
    let mut pixels = Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap();


    event_loop.run(move |event, _, mut control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::MainEventsCleared => {
                while !cpu.bus.ppu.frame_ready() {
                    cpu.tick();
                }
                window.request_redraw();
            }

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    pixels.resize_surface(size.width, size.height).unwrap();
                }
                _ => {}
            },

            Event::RedrawRequested(_) => {
                let frame = pixels.frame_mut();

                let framebuffer = cpu.bus.ppu.get_pixel_buffer();
                frame.copy_from_slice(framebuffer);
                if pixels.render().is_err() {
                    *control_flow = ControlFlow::Exit;
                }
            }

            _ => {}
        }
    });
}