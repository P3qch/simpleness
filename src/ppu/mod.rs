mod ppu_bus;
mod ppu_ctrl;
mod ppu_mask;
mod ppu_status;

use ppu_bus::PPUBus;
use ppu_ctrl::PPUCtrl;
use ppu_mask::PPUMask;
use ppu_status::PPUStatus;

use crate::memory::mapper::SharedMapper;

const PPUADDR: u16 = 0x2006;
const PPUDATA: u16 = 0x2007;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

pub struct PPU {
    write_latch: u8, //Toggles on each write to either PPUSCROLL or PPUADDR, indicating whether this is the first or second write. Clears on reads of PPUSTATUS. Sometimes called the 'write latch' or 'write toggle'.
    ppu_ctrl: PPUCtrl,
    ppu_mask: PPUMask,
    ppu_status: PPUStatus,
    ppu_bus: PPUBus,

    sdl_context: sdl2::Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    window_canvas: sdl2::render::Canvas<sdl2::video::Window>,
    sdl_event_pump: sdl2::EventPump,
}

impl PPU {
    pub fn new() -> Self {
        let ppu_mask = PPUMask::from_bytes([0]);
        let ppu_ctrl = PPUCtrl::from_bytes([0]);
        let ppu_status = PPUStatus::from_bytes([0]); 
        
        
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();
        Self {
            write_latch: 0,
            ppu_status: ppu_status,
            ppu_ctrl: ppu_ctrl,
            ppu_mask: ppu_mask,
            ppu_bus: PPUBus::new(),
            sdl_context: sdl_context,
            video_subsystem: video_subsystem,
            window_canvas: canvas,
            sdl_event_pump: event_pump,
        }
    }

    pub fn set_mapper(&mut self, mapper: SharedMapper) {
        self.ppu_bus.set_mapper(mapper);
    }

    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            _ => 0,
        }
    }

    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            _ => {}
        }
    }   
    
    pub fn render_pattern_table(&mut self) {

        self.window_canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.window_canvas.clear();
        self.window_canvas.present();
        'running: loop {
            self.window_canvas.set_draw_color(Color::RGB(0, 0, 0));
            self.window_canvas.clear();
            for event in self.sdl_event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    _ => {}
                }
            }
            // The rest of the game loop goes here...

            self.window_canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }

}
