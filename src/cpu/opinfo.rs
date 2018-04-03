use cpu::cpu::{AddressingMode, OpResponse};
use cpu::cpu::AddressingMode::*;

/// Information required to correctly execute an instruction. The cycles
/// _shall _ be  stored without conditional cycles such as extra cycles caused
/// by page crossings.
#[derive(Debug)]
pub struct OpInfo {
    pub mode: AddressingMode,
    pub cycles: usize,
    pub bytes: usize,
}

//impl OpInfo {
    //pub fn new(mode: AddressingMode, bytes: usize, cycles: usize) -> OpInfo {
        //OpInfo {
            //mode,
            //bytes,
            //cycles,
        //}
    //}
//}

/// currently not used
//pub static JUMP_TABLE: [fn(&mut CPU, &mut Memory, &OpInfo) -> OpResponse; 1] = [
    //CPU::bcc,
//];


/// bytes in OpInfo do not contain conditional cycles like they occur on page
/// crossings
///
/// Currently not in use in favour for direct encoding in cpu struct.
#[cfg_attr(rustfmt, rustfmt_skip)]
pub static OP_INFO: [OpInfo; 256] = [
    // 0x00
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    // 0x08
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    // 0x10
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    // 0x20
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    // 0x30
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    // 0x40
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    // 0x50
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    // 0x60
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    // 0x70
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    // 0x80
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    // 0x90
    OpInfo{mode: Relative, bytes: 2, cycles: 2},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    // 0xA0
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    // 0xD0
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    // 0xE0
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},

    //0xF0
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
    OpInfo{mode: Immediate, bytes: 0xFF, cycles: 0xFF},
];
