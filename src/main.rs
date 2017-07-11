use std::env;
use std::fs::File;
use std::io::prelude::*;


// Nintendo Entertainment System Emulator
// Author: Kyron Taylor
// ==
// Notes:
// + three general purpose registers, a, x and y.
// + little endian arch
// + 148 instructions
// + 8-bit stack pointer
// + 16-bit program counter

pub struct NESEmulator {

    // GPR (general purpose registers)
    a: u8, // accumulator
    x: u8, // index register
    y: u8, // index register

    // SP (stack pointer)
    sp: u8,

    // PC (program counter)
    pc: u16,

    // Processor Flags
    p: u8, // Negative, oVerflow, ss, Decimal, Interupt, Zero, Carry

    // Memory
    ram: [u8; 2047], // 2kb of console memory

    // Clock Speed
    clock: f32

}

// implimentation
impl NESEmulator {
    // initializes registers
    fn new() -> NESEmulator {
        return NESEmulator {
            a: 0x0000,
            x: 0x0000,
            y: 0x0000,
            sp: 0x0000,
            pc: 0xC000,
            p: 0b00000000,
            ram: [0u8; 2047],
            clock: 1.79
        }
    }
}

fn main(){
    let args: Vec<String> = env::args().collect();
    if(args.len() > 1){
        println!("Opening ROM: {}",&args[1]);

        let file = File::open(&args[1]).expect("ERROR: File not found");
    }
    else{
        println!("Please specify a ROM")
    }
    return();
}
