//! This module represents the NES console system.
//!
//! # Ideas
//!
//! * The main program launches a new nes instance each time a new rom is loaded.
//! * nes.run() is launched in a dedicated thread. All I/O works via shared
//!   resources. That is the framebuffer and controls as far as I can see atm.
//!
//! # Open Questions
//!
//! * once the rom is loaded, where is it put in memory?

use cpu::cpu::CPU;
use memory::Memory;

/// dummy for completeness
pub struct Cartridge {}
pub struct PPU{}
pub struct APU{}
pub struct Clock{}

pub struct Console {
    cpu: CPU,
    mem: Memory,
    // TODO: missing:
    //  car
    car: Cartridge,
    ppu: PPU,
    apu: APU,
    clk: Clock,
}

impl Console {
    pub fn new() -> Console {
        Console {
            cpu: CPU::new(),
            mem: Memory::new(),

            // dummies
            car: Cartridge{},
            ppu: PPU{},
            apu: APU{},
            clk: Clock{},
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.mem);
    }

    pub fn load_cartridge(&mut self, filepath: &str) {
        self.cpu.reset(&mut self.mem);
    }

    pub fn run(&mut self) {
        // game loop
        // TODO: timing
        loop {
            self.cpu.step(&mut self.mem);
        }
    }

    pub fn pause(&mut self) {}

    /// Store the state of the system
    pub fn save(&mut self, path: &str) {}

    pub fn load(&mut self, path: &str) {}

    pub fn poweroff(&mut self) {}
}
