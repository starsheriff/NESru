
struct StatusRegisterBits {

}

struct StatusRegister {
    CarryFlag: bool,
    ZeroFlag: bool,
    InterruptDisable: bool,
    DecimalMode: bool,
    BreakCommand: bool,
    OverflowFlag: bool,
}

impl StatusRegister {
    fn new() -> StatusRegister {
        StatusRegister {
            CarryFlag: false,
            ZeroFlag: false,
            InterruptDisable: false,
            DecimalMode: false,
            BreakCommand: false,
            OverflowFlag: false,
        }
    }

    fn from_u8() -> StatusRegister {
        StatusRegister::new()
    }
}

struct CPU {
    accumulator: u8,
    stack_pointer: u8,
    program_counter: u8,
    index_x: u8,
    index_y: u8,
    status_register: StatusRegister,
}

impl CPU {

    pub fn new() -> CPU {
        CPU{
            accumulator: 0,
            stack_pointer: 0,
            program_counter: 0,
            index_x: 0,
            index_y: 0,
            status_register: StatusRegister::new(),
        }
    }

    pub fn powerup(&mut self) {
    }

    fn run(&mut self) {
        loop {
            self.step();
        }
    }

    fn step(&mut self) {
    }

}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
