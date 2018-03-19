use cpu::status_register::StatusRegister;

use memory::Memory;

use std::fmt::Display;


struct CPU {
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

    fn step(&mut self, mem: &mut Memory) {
        self.cycles += 1;

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
    fn execute_next(&mut self, mem: &mut Memory) {
        let (bytes_consumed, cycles_spent) = match self.program_counter {
            // ADC
            0x69 => self.adc(mem, AddressingMode::Immediate),
            0x65 => self.adc(mem, AddressingMode::ZeroPage),
            0x75 => self.adc(mem, AddressingMode::ZeroPageX),
            0x6D => self.adc(mem, AddressingMode::Absolute),
            0x7D => self.adc(mem, AddressingMode::AbsoluteX),
            0x79 => self.adc(mem, AddressingMode::AbsoluteY),
            0x61 => self.adc(mem, AddressingMode::IndexedIndirect),
            0x71 => self.adc(mem, AddressingMode::IndirectIndexed),
            
            // TODO: more remaining optcodes

            _ => panic!("not implemented"),
        };

        self.cycles += cycles_spent;

    }

    fn get_address(&self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => {
                // in this addressing mode the constant is embedded directly in
                // the programs assembler. Thus the value to read is at the next
                // position in memory
                self.program_counter + 1
            }
            _ => panic!("not implemented"),
        }
    }

    /// Returns a tuple containing the address and the amount of cycles the
    /// 6502 cpu would have spent.
    fn get_address_immediate(&self) -> (u16, usize) {
        (self.program_counter + 1, 2)
    }

    //fn get_instruction_info(&self, mem: &Memory) -> InstructionInfo {
        //InstructionInfo {
            //addressing_mode: AddressingMode::Immediate,
            //cycles: 16,
            //addr: 16,
        //}
    //}

    /// TODO: types for bytes and cycles for static typing benefits/safety
    fn adc(&mut self, mem: &mut Memory, mode: AddressingMode) -> (usize, usize){
        let addr = self.get_address(AddressingMode::Immediate);

        let a = self.accumulator;
        let m = mem.read(addr);
        let c = self.carry_flag();

        // this might overflow
        self.accumulator = a + m + c;

        if a as usize + m as usize + c as usize > 0xFF {
            self.status_register.carry_flag = true;
        } else {
            self.status_register.carry_flag = false;
        }

        if a == 0 {
            self.status_register.zero_flag = true;
        }

        // TODO: set negative flag
        // TODO: set overflow flag
        (2, 2)
    }
}

enum AddressingMode {
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

struct InstructionInfo {
    addressing_mode: AddressingMode,
    cycles: usize,
    addr: u16,
}

//impl Display for CPU {
//fn fmt(&self) -> Result<(), std::fmt::Error> {
//()
//}
//}

struct Instruction<'a> {
    func: &'a fn(&mut CPU, &mut Memory),
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
    fn test_optcode_0x69() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);
        cpu.program_counter = 0x69;
        cpu.step(&mut mem);
    }
}