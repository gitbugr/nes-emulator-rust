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
    cpu_memory: [u8; 0x10000],

    // Clock Speed
    clock: f32,

    // File Path
    filepath: String
}

// implimentation
impl NESEmulator {
    // initializes registers
    pub fn new(f: &String) -> NESEmulator {
        return NESEmulator {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            sp: 0x00,
            pc: 0xC000, // start reading instructions from byte 0xC000
            p: 0b00000000,
            cpu_memory: [0u8; 0x10000],
            clock: 1.79, // US-region
            filepath: f.to_owned()
        }
    }

    pub fn load_rom(&self){
        let mut file = File::open(&self.filepath).expect("ERROR: File not found"); // load file
        let mut buffer = Vec::new(); // definte buffur vector
        file.read_to_end(&mut buffer); // store bytes in buffer

        if(b"NES" == &buffer[0..3]) { println!("Passed header check!"); }
        else { println!("Failed header check!"); }
    }

    // (memory mapper) memory i/o
    fn read_mem() {

    }

    fn write_mem(){

    }

    // reads instructions, calls functions, sleeps
    fn tick(&self) {

    }

    fn run() {

    }
}

fn main(){
    // Gets ROM filename from user argument and loads it into a buffer
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        println!("--------------------------------------------");
        println!("\\ Nintendo Entertainment System Emulator  /");
        println!("/    Written by Kyron Taylor (gitbugr)    \\");
        println!("--------------------------------------------");
        println!("Starting NES Emulator with Default Values...");
        let mut emu = NESEmulator::new(&args[1]);

        println!("Opening ROM: '{}'",&args[1]); // debug
        emu.load_rom();



    }
    else{
        println!("Please specify a ROM"); // no args
    }
    return();
}
