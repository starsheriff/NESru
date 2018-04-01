use cpu::status_register::StatusRegister;
use cpu::utils;
use cpu::opinfo::{OpInfo, OP_INFO};

use memory::Memory;

type MemoryAddress = u16;
type PageCrossed = bool;

pub struct OpResponse {
    bytes_consumed: usize,
    cycles_spent: usize,
}

#[derive(Copy, Clone, Debug)]
pub enum AddressingMode {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndexedIndirect,
    IndirectIndexed,
}

pub struct CPU {
    accumulator: u8,
    stack_pointer: u8,
    program_counter: u16,
    index_x: u8,
    index_y: u8,
    status_register: StatusRegister,

    /// count the total amount of cycles spent
    cycles: usize,
    /// delay the cpu for a specific amount of cycles
    delay_cycles: u8,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            accumulator: 0,
            stack_pointer: 0,
            program_counter: 0,
            index_x: 0,
            index_y: 0,
            status_register: StatusRegister::new(),

            delay_cycles: 0,
            cycles: 0,
        }
    }

    pub fn powerup(&mut self, mem: &mut Memory) {
        self.status_register.set_all(0x34);
        self.accumulator = 0;
        self.index_x = 0;
        self.index_y = 0;
        self.stack_pointer = 0xFD;

        // Remaining tasks: set memory
        // LSFR = 0x00
        mem.write(0x4017, 0x00);
        mem.write_range(0x4000, 0x400F, 0x00);
    }

    pub fn reset(&mut self, mem: &mut Memory) {
        self.stack_pointer -= 3;
        self.status_register.interrupt_disable = true;

        // Remaining tasks: set memory
        mem.write(0x4015, 0x00);
    }

    fn carry_flag(&self) -> u8 {
        self.status_register.carry_flag as u8
    }

    /// Update the zero flag of the cpus status register. This is a common call
    /// in many instructions.
    fn update_zero_flag(&mut self) {
        if self.accumulator == 0x00 {
            self.status_register.zero_flag = true;
        }
    }

    fn update_negative_flag(&mut self) {
        // TODO: implement
        if self.accumulator >> 7 & 0x01 == 0x01 {
            self.status_register.negative_flag = true;
        } else {
            self.status_register.negative_flag = false;
        }
    }

    /// Get the memory address stored at (actually behind) the current program
    /// counter.
    ///
    /// This method will increment the cpu cycles spent if it detects a page
    /// crossing.
    ///
    /// 1. All instructions have only one argument, so there is only one memory
    ///    address to read per instruction.
    /// 2. Depending on the addressing mode, either one or two bytes have to be
    ///    read from memory.
    ///
    /// # Bugs/TODOs
    ///
    /// 1. detect page crossings. Where?
    ///     * Which modes?
    ///
    fn get_address(&self, mem: &Memory, mode: AddressingMode) -> Option<u16> {
        // TODO: detect page crossings!
        use cpu::cpu::AddressingMode::*;

        match mode {
            Absolute => {
                let a = mem.read(self.program_counter + 1) as u16;
                let b = mem.read(self.program_counter + 2) as u16;

                Some((b << 8) + a)
            }
            AbsoluteX => {
                // TODO: detect page crossing
                let a = mem.read(self.program_counter + 1) as u16;
                let b = mem.read(self.program_counter + 2) as u16;
                let c = (b << 8) + a;

                Some(c + self.index_x as u16)
            }
            AbsoluteY => {
                // TODO: detect page crossing
                let a = mem.read(self.program_counter + 1) as u16;
                let b = mem.read(self.program_counter + 2) as u16;
                let c = (b << 8) + a;

                Some(c + self.index_y as u16)
            }
            Accumulator => None,
            Implicit => None,
            Immediate => Some(self.program_counter + 1),
            IndexedIndirect => {
                let a = mem.read(self.program_counter + 1) as u16;
                let b = a.wrapping_add(self.index_x as u16);
                let c = mem.read(b) as u16;
                let d = mem.read(b + 1) as u16;
                let e = (d << 8) + c;

                Some(e)
            }
            IndirectIndexed => {
                // TODO: detect page crossing
                let a = mem.read(self.program_counter + 1) as u16;
                let b = mem.read(a) as u16;
                let c = mem.read(a + 1) as u16;
                let d = (c << 8) + b;
                let e = d + self.index_y as u16;

                Some(e)
            }
            Relative => Some(self.program_counter + 1),
            ZeroPage => Some(mem.read(self.program_counter + 1) as u16),
            ZeroPageX => {
                let a = mem.read(self.program_counter + 1);
                let b = a.wrapping_add(self.index_x);

                Some(b as u16)
            }
            ZeroPageY => {
                let a = mem.read(self.program_counter + 1);
                let b = a.wrapping_add(self.index_y);

                Some(b as u16)
            }
        }
    }

    fn read16(&self, mem: &Memory, addr: u16) -> u16 {
        let msb = mem.read(addr) as u16;
        let lsb = mem.read(addr + 1) as u16;

        (msb << 8) + lsb
    }

    fn step(&mut self, mem: &mut Memory) {
        // TODO: really required?
        self.cycles += 1;

        // TODO: really required? Should be handled by caller of `step`
        // delay cycles has higher priority than interrupts
        if (self.delay_cycles > 0) {
            self.delay_cycles -= 1;
            return;
        }

        // TODO: check for interrupts
        //

        self.execute_next(mem);
    }

    /// Executes the next instruction stored at the program_counters address.
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn execute_next(&mut self, mem: &mut Memory) {
        use cpu::cpu::AddressingMode::*;

        let op_response = match self.program_counter {
            // ADC
            0x69 => self.adc(mem, &OpInfo{mode: Immediate, bytes: 2, cycles: 2}),
            0x65 => self.adc(mem, &OpInfo{mode: ZeroPage,  bytes: 2, cycles: 3}),
            0x75 => self.adc(mem, &OpInfo{mode: ZeroPageX, bytes: 2, cycles: 4}),
            0x6D => self.adc(mem, &OpInfo{mode: Absolute,  bytes: 3, cycles: 4}),
            0x7D => self.adc(mem, &OpInfo{mode: AbsoluteX, bytes: 3, cycles: 4}),
            0x79 => self.adc(mem, &OpInfo{mode: AbsoluteY, bytes: 3, cycles: 4}),
            0x61 => self.adc(mem, &OpInfo{mode: IndexedIndirect, bytes: 2, cycles: 6}),
            0x71 => self.adc(mem, &OpInfo{mode: IndirectIndexed, bytes: 2, cycles: 5}),

            // AND
            0x29 => self.and(mem, AddressingMode::Immediate),
            0x25 => self.and(mem, AddressingMode::ZeroPage),
            0x35 => self.and(mem, AddressingMode::ZeroPageX),
            0x2D => self.and(mem, AddressingMode::ZeroPageY),
            0x3D => self.and(mem, AddressingMode::AbsoluteX),
            0x39 => self.and(mem, AddressingMode::AbsoluteY),
            0x21 => self.and(mem, AddressingMode::IndexedIndirect),
            0x31 => self.and(mem, AddressingMode::IndirectIndexed),

            // ASL
            0x0A => self.asl(mem, AddressingMode::Accumulator),
            0x06 => self.asl(mem, AddressingMode::ZeroPage),
            0x16 => self.asl(mem, AddressingMode::ZeroPageX),
            0x0E => self.asl(mem, AddressingMode::Absolute),
            0x1E => self.asl(mem, AddressingMode::AbsoluteX),

            // BCC
            0x90 => self.bcc(mem, &OpInfo{mode: Relative, bytes: 2, cycles: 2}),

            // BCS
            0xB0 => self.bcs(mem, &OpInfo{mode: Relative, bytes: 2, cycles: 2}),

            // BEQ (branch if equal)
            0xF0 => self.beq(mem, &OpInfo{mode: Relative, bytes: 2, cycles: 2}),

            // BIT (bit test)
            0x24 => self.bit(mem, AddressingMode::ZeroPage),
            0x2C => self.bit(mem, AddressingMode::Absolute),

            // TODO: more remaining optcodes
            _ => panic!("not implemented"),
        };

        self.cycles += op_response.cycles_spent;
    }

    /// CPU instruction: ADC (add with carry)
    ///
    /// This instruction adds the contents of a memory location to the
    /// accumulator together with the carry bit. If overflow occurs the carry
    /// bit is set, this enables multiple byte addition to be performed.
    ///
    /// Affected Registers:
    /// - zero flag: set if accumulator is zero
    /// - negative flag: set if bit 7 (highest bit) is set
    /// - overflow flag: set if sign bit is incorrect
    fn adc(&mut self, mem: &mut Memory, opi: &OpInfo) -> OpResponse {
        let addr = self.get_address(mem, opi.mode).unwrap();
        let m = mem.read(addr);

        let a = self.accumulator;
        let c = self.carry_flag();

        // this might overflow
        self.accumulator = a + m + c;

        // set carry flag
        if a as usize + m as usize + c as usize > 0xFF {
            self.status_register.carry_flag = true;
        } else {
            self.status_register.carry_flag = false;
        }

        self.status_register.overflow_flag = utils::calculate_overflow_bit(a, m, self.accumulator);

        self.update_zero_flag();
        self.update_negative_flag();

        // TODO: fix return value
        OpResponse {
            bytes_consumed: opi.bytes,
            cycles_spent: opi.cycles,
        }
    }

    /// CPU instruction: AND (logical AND)
    ///
    /// A logical AND is performed, bit by bit, on the accumulator contents
    /// using the contents of a byte of memory.
    fn and(&mut self, mem: &mut Memory, mode: AddressingMode) -> OpResponse {
        let addr = self.get_address(mem, mode).unwrap();
        let m = mem.read(addr);
        let a = self.accumulator;

        self.accumulator = a & m;

        self.update_zero_flag();
        self.update_negative_flag();

        // TODO: fix return value
        OpResponse {
            bytes_consumed: 2,
            cycles_spent: 2,
        }
    }

    fn asl(&mut self, mem: &mut Memory, mode: AddressingMode) -> OpResponse {
        match mode {
            accumulator => {
                let (v, c) = self.accumulator.overflowing_shl(1);
                self.accumulator = v;
                self.status_register.carry_flag = c;
            }
            _ => {
                let addr = self.get_address(mem, mode).unwrap();
                let m = mem.read(addr);
                let (v, c) = m.overflowing_shl(1);
                self.status_register.carry_flag = c;
                //self.write_mem(val);
            }
        }

        self.update_zero_flag();
        self.update_negative_flag();

        // TODO: fix return value
        OpResponse {
            bytes_consumed: 2,
            cycles_spent: 2,
        }
    }

    pub fn bcc(&mut self, mem: &mut Memory, opi: &OpInfo) -> OpResponse {
        // TODO
        panic!("not implemented");
    }

    fn bcs(&mut self, mem: &mut Memory, opi: &OpInfo) -> OpResponse {
        // TODO
        panic!("not implemented");
    }

    fn beq(&mut self, mem: &mut Memory, opi: &OpInfo) -> OpResponse {
        // TODO
        panic!("not implemented");
    }

    fn bit(&mut self, mem: &mut Memory, mode: AddressingMode) -> OpResponse {
        // TODO
        panic!("not implemented");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_powerup_state() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);

        assert_eq!(cpu.accumulator, 0);
        assert_eq!(cpu.index_x, 0);
        assert_eq!(cpu.index_y, 0);
        assert_eq!(cpu.stack_pointer, 0xFD);

        assert_eq!(0x34, 52);
        assert_eq!(cpu.status_register.unused_bit, true);
        assert_eq!(cpu.status_register.break_command, true);
        assert_eq!(cpu.status_register.interrupt_disable, true);

        assert_eq!(cpu.status_register.carry_flag, false);
        assert_eq!(cpu.status_register.zero_flag, false);
        assert_eq!(cpu.status_register.decimal_mode, false);
        assert_eq!(cpu.status_register.overflow_flag, false);
        assert_eq!(cpu.status_register.negative_flag, false);

        // memory
        assert_eq!(mem.read(0x4017), 0);
    }

    #[test]
    fn cpu_reset_state() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();
        cpu.powerup(&mut mem);

        cpu.status_register.interrupt_disable = false;
        let sp_before = cpu.stack_pointer;
        println!("{}", sp_before);

        cpu.reset(&mut mem);

        assert_eq!(cpu.stack_pointer, sp_before - 3);
        assert_eq!(cpu.status_register.interrupt_disable, true);
        assert_eq!(mem.read(0x4015), 0);
        // TODO: test remaining memory addresses
    }

    #[test]
    fn cpu_delay_cycles() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);

        cpu.delay_cycles = 3;
        cpu.step(&mut mem);
        assert_eq!(cpu.cycles, 1);
        cpu.step(&mut mem);
        assert_eq!(cpu.cycles, 2);
        cpu.step(&mut mem);
        assert_eq!(cpu.cycles, 3);
    }

    #[test]
    fn cpu_set_negative_true() {
        let mut cpu = CPU::new();

        cpu.accumulator = 0xFF;
        assert_eq!(cpu.status_register.negative_flag, false);
        cpu.update_negative_flag();
        assert_eq!(cpu.status_register.negative_flag, true);
    }

    #[test]
    fn cpu_set_negative_false() {
        let mut cpu = CPU::new();

        cpu.accumulator = 0x00;
        assert_eq!(cpu.status_register.negative_flag, false);
        cpu.update_negative_flag();
        assert_eq!(cpu.status_register.negative_flag, false);
    }

    #[test]
    fn test_optcode_0x69() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);
        cpu.program_counter = 0x69;
        cpu.step(&mut mem);
    }

    #[should_panic]
    #[test]
    fn test_optcode_0x90() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);
        cpu.program_counter = 0x90;
        cpu.step(&mut mem);
    }

    #[test]
    fn test_addressing_absolute() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);
        cpu.program_counter = 0x0000;

        mem.write(0x0001, 0xFF);
        mem.write(0x0002, 0xAA);

        let result = cpu.get_address(&mut mem, AddressingMode::Absolute);
        let expected = Some(0xAAFF);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_addressing_absolute_x() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);
        cpu.program_counter = 0x0000;
        cpu.index_x = 0x02;

        mem.write(0x0001, 0xBB);
        mem.write(0x0002, 0xAA);

        let result = cpu.get_address(&mut mem, AddressingMode::AbsoluteX);
        let expected = Some(0xAABD);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_addressing_absolute_y() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);
        cpu.program_counter = 0x0000;
        cpu.index_y = 0x04;

        mem.write(0x0001, 0xBB);
        mem.write(0x0002, 0xAA);

        let result = cpu.get_address(&mut mem, AddressingMode::AbsoluteY);
        let expected = Some(0xAABF);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_addressing_indexed_indirect() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);
        cpu.program_counter = 0x0000;
        cpu.index_x = 0x04;

        mem.write(0x0001, 0xA0);
        mem.write(0x00A4, 0xF0);
        mem.write(0x00A5, 0xDA);

        let expected = Some(0xDAF0);
        let result = cpu.get_address(&mut mem, AddressingMode::IndexedIndirect);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_addressing_indirect_indexed() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);
        cpu.program_counter = 0x0000;
        cpu.index_y = 0x04;

        mem.write(0x0001, 0xA0);
        mem.write(0x00A0, 0xF0);
        mem.write(0x00A1, 0xDA);

        let expected = Some(0xDAF0 + 0x04);
        let result = cpu.get_address(&mut mem, AddressingMode::IndirectIndexed);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_read16() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);
        mem.write(0x0004, 0xAA);
        mem.write(0x0005, 0xCC);

        let result = cpu.read16(&mem, 0x0004);
        let expected = 0xAACC;
        assert_eq!(result, expected);
    }
}
