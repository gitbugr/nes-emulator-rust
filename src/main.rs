use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;


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
    cpu_memory: [u8; 0xFFFF],

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
            pc: 0x8000, // start reading instructions from byte 0x8000
            p: 0b00000000,
            cpu_memory: [0u8; 0xFFFF],
            clock: 1.79, // US-region
            filepath: f.to_owned()
        }
    }

    pub fn load_rom(&mut self){
        let mut file = File::open(&self.filepath).expect("ERROR: File not found"); // load file
        let mut buffer = Vec::new(); // definte buffur vector

        let mut has_cartram:bool = false;
        let mut has_trainer:bool = false;
        let mut region:bool = false;
        let mut rom_mapper_type:u8 = 0;

        // read file and store bytes in buffer
        file.read_to_end(&mut buffer);

        // check header
        if b"NES" == &buffer[0..3] { println!("Found .NES Header!"); }
        else { println!("NOT .NES FILETYPE!"); return(); }

        println!("Has {:?} 16kB ROM banks!",&buffer[4]);
        println!("Has {:?} 8kB VROM banks!",&buffer[5]);

        // check has cartram
        if &buffer[6] & 0b00000010 == 0b00000010 { has_cartram=true; println!("Has on-cartridge ram!") }
        else { println!("No on-cartridge ram!"); }

        // get lower bits of ROM mapper type
        let temp_low:u8 = (&buffer[6] << 4) >> 4;
        // get higher bits of ROM mapper type
        let temp_high:u8 = &buffer[7] << 4;
        // bitwise OR to get value
        rom_mapper_type = temp_low|temp_high;
        println!("ROM Mapper Type: {:?}",rom_mapper_type);

        // check has trainer
        if &buffer[7] & 0b00000100 == 0b00000100 { has_trainer=true; println!("Has trainer section!") }
        else { println!("No trainer section!"); }

        println!("Has {:?} 8kB RAM banks!",&buffer[7]);

        if &buffer[9] & 0b00000001 == 0b00000001 { region=true; println!("Region: PAL"); }
        else { println!("Region: NTSC"); }

        println!("Loading ROM bank #1 into memory $8000 - $10000...");
        for i in 0x0000..0x8000-16{
            self.cpu_memory[0x8000+i] = buffer[16+i];
        }
        println!("Loaded!")
    }

    // Tick function
    // Reads OPCODES and executes functions
    fn tick(&mut self) {
        match self.cpu_memory[self.pc as usize]{
            // PHP - Push Processor Status
            // Pushes a copy of the status flags on to the stack.
            0x08 => {
                print!("PHP\n");
                self.cpu_memory[self.sp as usize] = self.p;
                self.sp+=0x01;
                self.pc+=0x0001;
            },
            // CPY - Compare Y Register
            // This instruction compares the contents of the Y register with another memory held value and sets the zero and carry flags as appropriate.
            // (Immediate)
            0xC0 => {
                print!("CPY #{:2x}\n",self.cpu_memory[(self.pc+0x0001) as usize]);
                if self.y >= self.cpu_memory[(self.pc+0x0001) as usize] { self.set_bitflag(0,true); }
                else { self.set_bitflag(0,false); }

                if self.y == self.cpu_memory[(self.pc+0x0001) as usize] { self.set_bitflag(1,true); }
                else { self.set_bitflag(1,false); }

                if check_bit(self.cpu_memory[(self.pc+0x0001) as usize], 7) { self.set_bitflag(7,true); }
                else { self.set_bitflag(7,false); }

                self.pc+=0x0002;
            },
            // Default
            _ => {
                print!("${:2x}\n",self.cpu_memory[self.pc as usize]);
                let mut child = Command::new("sleep").arg("10").spawn().unwrap();
                let _result = child.wait().unwrap();
                self.pc+=0x0001;
            }
        }
    }

    fn push(){

    }

    pub fn run(&mut self) {
        loop {
            print!("pc: ${:0>4x}, p: {:0>8b}, op: ", self.pc, self.p);
            self.tick();
        }

    }

    fn set_bitflag(&mut self, pos:usize, val:bool){
        let positions:[u8;8] = [
            0b00000001,
            0b00000010,
            0b00000100,
            0b00001000,
            0b00010000,
            0b00100000,
            0b01000000,
            0b10000000
        ];
        if(self.p & positions[pos] == positions[pos]) && !val{
            self.p = self.p ^ positions[pos];
        }
        else if (self.p & positions[pos] != positions[pos]) && val{
            self.p = self.p | positions[pos];
        }
    }


}

fn main(){
    // Gets ROM filename from user argument and loads it into a buffer
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        println!("--------------------------------------------");
        println!("\\  Nintendo Entertainment System Emulator  /");
        println!("/     Written by Kyron Taylor (gitbugr)    \\");
        println!("--------------------------------------------");
        println!("Starting NES Emulator with Default Values...");
        let mut emu = NESEmulator::new(&args[1]);

        println!("Opening ROM: '{}'",&args[1]); // debug
        emu.load_rom();
        emu.run();


    }
    else{
        println!("Please specify a ROM"); // no args
    }
    return();
}


fn check_bit(val:u8, pos:usize) -> bool{
    let positions:[u8;8] = [
        0b00000001,
        0b00000010,
        0b00000100,
        0b00001000,
        0b00010000,
        0b00100000,
        0b01000000,
        0b10000000
    ];
    if val & positions[pos] == positions[pos] {
        return(true);
    }
    return(false);
}
