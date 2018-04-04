//! This module represents the NES console system.

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

    pub fn poweron(&mut self) {
        self.cpu.powerup(&mut self.mem);
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

    pub fn save(&mut self) {}

    pub fn poweroff(&mut self) {}
}
