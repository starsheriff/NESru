const MEM_SIZE: usize = 0xFFFF;

pub struct Memory {
    mem: [u8; MEM_SIZE],
}

impl Memory {
    pub fn new() -> Memory {
        Memory { mem: [0; MEM_SIZE] }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        // TODO: mirroring? Is it necessary to emulate?
        self.mem[addr as usize] = val;
    }

    /// Write a range in memory with a common value. The range is inclusice,
    /// meaning both first and last are written.
    pub fn write_range(&mut self, first: usize, last: usize, val: u8) {
        for x in (first..last) {
            self.mem[x] = val;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mem_write_range() {
        let mut mem = Memory::new();
        mem.write_range(0x00FF, 0x0200, 0x04);
        for x in (0x00FF..0x0200) {
            assert_eq!(mem.read(x), 0x04);
        }
    }
}
