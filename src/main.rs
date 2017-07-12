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
            pc: 0xFFFC, // start reading instructions from byte 0x8000
            p: 0x34,
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
                print!("CPY #{:0>2x}\n",self.cpu_memory[(self.pc+1) as usize]);
                if self.y >= self.cpu_memory[(self.pc+0x0001) as usize] { self.set_bitflag(0,true); }
                else { self.set_bitflag(0,false); } // set carry flag

                if self.y == self.cpu_memory[(self.pc+0x0001) as usize] { self.set_bitflag(1,true); }
                else { self.set_bitflag(1,false); } // set zero flag

                if check_bit(self.cpu_memory[(self.pc+0x0001) as usize], 7) { self.set_bitflag(7,true); }
                else { self.set_bitflag(7,false); } // set negative flag

                self.pc+=0x0002; // next instruction
            },
            // SLO - Shift Left OR accumulator
            // This instruction shift left one bit in memory, then ORs the accumulator with the memory address and sets the negative, zero and carry flags as appropriate.
            // (Immediate)
            0x07 => {
                print!("SLO #{:0>2x}\n",self.cpu_memory[(self.pc+1) as usize]);
                if self.cpu_memory[(self.pc+0x0001) as usize] >= 0b10000000 { self.set_bitflag(0,true); }
                else { self.set_bitflag(0,false); } // set carry flag

                let addr = self.cpu_memory[(self.pc+0x0001) as usize] << 1;
                self.a = self.a | addr;

                if self.a == 0x00 { self.set_bitflag(1,true); }
                else { self.set_bitflag(1,false); } // set zero flag

                if check_bit(addr, 7) { self.set_bitflag(7,true); }
                else { self.set_bitflag(7,false); } // set negative flag

                self.pc+=0x0002; // next instruction
            },
            // SLO - Shift Left OR accumulator
            // This instruction shift left one bit in memory, then ORs the accumulator with the memory address and sets the negative, zero and carry flags as appropriate.
            // (Absolute)
            0x0f => {
                print!("SLO #{:0>2x}{:0>2x}\n",self.cpu_memory[(self.pc+2) as usize],self.cpu_memory[(self.pc+1) as usize]);
                let mut addr = two_u8_to_u6(self.cpu_memory[(self.pc+2) as usize], self.cpu_memory[(self.pc+1) as usize]);

                if addr >= 0b1000000000000000 { self.set_bitflag(0,true); }
                else { self.set_bitflag(0,false); } // set carry flag

                addr = addr << 1;
                self.a = self.a | (addr as u8);

                if self.a == 0x00 { self.set_bitflag(1,true); }
                else { self.set_bitflag(1,false); } // set zero flag

                if check_bit(addr as u8, 7) { self.set_bitflag(7,true); }
                else { self.set_bitflag(7,false); } // set negative flag

                self.pc+=0x0003; // next instruction
            },
            // SLO - Shift Left OR accumulator
            // This instruction shift left one bit in memory, then ORs the accumulator with the memory address and sets the negative, zero and carry flags as appropriate.
            // (Indirect, X)
            0x03 => {
                print!("SLO ({:0>2x},x)\n",self.cpu_memory[(self.pc+1) as usize]);
                let mut addr = self.cpu_memory[(self.pc+1) as usize];
                        addr = self.cpu_memory[(addr+self.x) as usize];

                if addr >= 0b10000000 { self.set_bitflag(0,true); }
                else { self.set_bitflag(0,false); } // set carry flag

                let addr = addr << 1;
                self.a = self.a | addr;

                if self.a == 0x00 { self.set_bitflag(1,true); }
                else { self.set_bitflag(1,false); } // set zero flag

                if check_bit(addr, 7) { self.set_bitflag(7,true); }
                else { self.set_bitflag(7,false); } // set negative flag

                self.pc+=0x0002; // next instruction
            },
            // ORA - OR Accumulator
            // Performs a bitwise OR with the Accumulator
            // (Indirect, X)
            0x01 => {
                print!("ORA ({:0>2x},x)\n",self.cpu_memory[(self.pc+1) as usize]);
                let mut addr = self.cpu_memory[(self.pc+1) as usize];
                        addr = self.cpu_memory[(addr+self.x) as usize];

                self.a = self.a | addr;

                if self.a == 0x00 { self.set_bitflag(1,true); }
                else { self.set_bitflag(1,false); } // set zero flag

                if check_bit(addr, 7) { self.set_bitflag(7,true); }
                else { self.set_bitflag(7,false); } // set negative flag

                self.pc+=0x0002; // next instruction
            },
            // ORA - OR Accumulator
            // Performs a bitwise OR with the Accumulator
            // (Indirect, Y)
            0x11 => {
                print!("ORA ({:0>2x},y)\n",self.cpu_memory[(self.pc+1) as usize]);
                let mut addr = self.cpu_memory[(self.pc+1) as usize];
                        addr = self.cpu_memory[(addr+self.x) as usize];

                self.a = self.a | addr;

                if self.a == 0x00 { self.set_bitflag(1,true); }
                else { self.set_bitflag(1,false); } // set zero flag

                if check_bit(addr, 7) { self.set_bitflag(7,true); }
                else { self.set_bitflag(7,false); } // set negative flag

                self.pc+=0x0002; // next instruction
            },
            // HLT - Halt
            // Stop Processor Counter
            // (Implied)
            0x02 => {
                print!("HLT\n");
                //wait(1);
                self.pc+=0x0001;
            },
            // DOP - Double NOP
            // No significance. PC moves 3 bytes forward
            // (Absolute)
            0x04 => {
                print!("DOP ${:0>2x}\n",self.cpu_memory[self.pc as usize+1]);
                self.pc+=0x0002; // next instruction
            },
            // TOP - Triple NOP
            // No significance. PC moves 3 bytes forward
            // (Absolute)
            0x0c => {
                print!("TOP ${:0>2x}{:0>2x}\n",self.cpu_memory[self.pc as usize+1],self.cpu_memory[self.pc as usize+2]);
                self.pc+=0x0003; // next instruction
            },
            // Default
            _ => {
                print!("${:0>2x}\n",self.cpu_memory[self.pc as usize]);
                wait(10);
                self.pc+=0x0001; // next instruction
            }
        }
    }

    fn push(){

    }

    pub fn run(&mut self) {
        loop {
            print!("[0x{:0>4x}] a: ${:0>2x}, x: ${:0>2x}, y: ${:0>2x}, p: {:0>8b}, op: ", self.pc, self.a, self.x, self.y, self.p);
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

fn two_u8_to_u6(a:u8,b:u8) -> u16 {
    let mut nb:u16 = 0;
    nb = (nb | (a as u16)) << 8;
    nb = nb | b as u16;
    return(nb);
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

fn wait(t:i32){
    let mut child = Command::new("sleep").arg(t.to_string()).spawn().unwrap();
    let _result = child.wait().unwrap();
}
