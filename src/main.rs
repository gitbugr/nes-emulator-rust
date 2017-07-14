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
            pc: 0x0000, // start reading instructions from byte 0x8000
            p: 0x34,
            cpu_memory: [0u8; 0x10000],
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

        let mut rom_banks:u8 = 0;

        // read file and store bytes in buffer
        file.read_to_end(&mut buffer);

        // check header
        if b"NES" == &buffer[0..3] { println!("Found .NES Header!"); }
        else { println!("NOT .NES FILETYPE!"); return(); }

        rom_banks = buffer[4];

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
        if &buffer[6] & 0b00000100 == 0b00000100 { has_trainer=true; println!("Has trainer section!") }
        else { println!("No trainer section!"); }

        println!("Has {:?} 8kB RAM banks!",&buffer[7]);

        if &buffer[9] & 0b00000001 == 0b00000001 { region=true; println!("Region: PAL"); }
        else { println!("Region: NTSC"); }

        println!("Loading ROM bank #1 into memory $8000 - $10000...");

        for bank in 0..2 {
            for i in 0x0000..0x4000 {
                self.cpu_memory[0x8000+(0x4000 * bank)+i] = buffer[16+(0x4000 * bank)+i];
            }
        }
        println!("Loaded!");
        let reset_vector:u16 = two_u8_to_u16(self.cpu_memory[0xFFFD],self.cpu_memory[0xFFFC]);
        println!("Setting pc to reset_vector: ${:0>4x}",reset_vector);
        self.pc = reset_vector;
    }

    // Tick function
    // Reads OPCODES and executes functions
    fn tick(&mut self) {
        match self.cpu_memory[self.pc as usize]{
            // PHP - Push Processor Status
            // Pushes a copy of the status flags on to the stack.
            0x08 => {
                print!("PHP\n");
                let status = self.p.to_owned();
                self.push_to_stack(status);
                self.pc+=0x0001;
            },
            // PHA - Push Accumulator
            // Pushes a copy of the accumulator on to the stack.
            0x48 => {
                print!("PHA\n");
                let acc = self.a.to_owned();
                self.push_to_stack(acc);
                self.pc+=0x0001;
            },
            // TAY - Transfer Accumulator to Y
            // Copies the current contents of the accumulator into the Y register and sets the zero and negative flags as appropriate.
            0xA8 => {
                print!("TAY\n");
                let acc = self.a.to_owned();
                self.y = acc;
                if self.y == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                if check_bit(self.y,7) { self.set_bitflag(7,true) }
                else { self.set_bitflag(7,false) } // set negative flag
                self.pc+=0x0001;
            },
            // LDA - LoaD Accumulator
            // Loads value into accumulator
            // (Immediate)
            0xA9 =>{
                print!("LDA #{:0>2x}\n",self.cpu_memory[(self.pc+1) as usize]);
                self.a = self.cpu_memory[(self.pc+1) as usize];
                if self.a == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                self.pc+=0x0002
            },
            // LDA - LoaD Accumulator
            // Loads value into accumulator
            // (Absolute)
            0xAD =>{
                print!("LDA ${:0>2x}{:0>2x}\n",self.cpu_memory[(self.pc+1) as usize],self.cpu_memory[(self.pc+2) as usize]);
                let addr = two_u8_to_u16(self.cpu_memory[(self.pc+2) as usize], self.cpu_memory[(self.pc+1) as usize]);
                self.a = self.cpu_memory[addr as usize];
                if self.a == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                self.pc+=0x0003
            },
            // LDA - LoaD Accumulator
            // Loads value into accumulator
            // (Zero Page)
            0xA5 =>{
                print!("LDA ${:0>2x}\n",self.cpu_memory[(self.pc+1) as usize]);
                let addr = self.cpu_memory[(self.pc+1) as usize];
                self.a = self.cpu_memory[addr as usize];
                if self.a == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                self.pc+=0x0002
            },
            // LDA - LoaD Accumulator
            // Loads value into accumulator
            // (Absolute, X)
            0xBD =>{
                print!("LDA (${:0>2x}{:0>2x}, x)\n",self.cpu_memory[(self.pc+1) as usize],self.cpu_memory[(self.pc+2) as usize]);
                let mut addr = two_u8_to_u16(self.cpu_memory[(self.pc+2) as usize], self.cpu_memory[(self.pc+1) as usize]);
                self.a = self.cpu_memory[addr.wrapping_add(self.x as u16) as usize];
                if self.a == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                if check_bit(self.a,7) { self.set_bitflag(7,true) }
                else { self.set_bitflag(7,false) } // set negative flag
                self.pc+=0x0003
            },
            // LDA - LoaD Accumulator
            // Loads value into accumulator
            // (Absolute, X)
            0xBD =>{
                print!("LDA (${:0>2x}{:0>2x}, x)\n",self.cpu_memory[(self.pc+1) as usize],self.cpu_memory[(self.pc+2) as usize]);
                let mut addr = two_u8_to_u16(self.cpu_memory[(self.pc+2) as usize], self.cpu_memory[(self.pc+1) as usize]);
                self.a = self.cpu_memory[addr.wrapping_add(self.y as u16) as usize];
                if self.a == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                if check_bit(self.a,7) { self.set_bitflag(7,true) }
                else { self.set_bitflag(7,false) } // set negative flag
                self.pc+=0x0003
            },
            // LDA - LoaD Accumulator
            // Loads value into accumulator
            // (Indirect, Y)
            0xB1 =>{
                print!("LDA (${:0>2x}), y\n",self.cpu_memory[(self.pc+1) as usize]);
                let mut addr = self.cpu_memory[self.pc as usize+1] as u16;
                        addr = two_u8_to_u16(self.cpu_memory[(addr as usize)+1],self.cpu_memory[addr as usize]);
                self.a = self.cpu_memory[(addr.wrapping_add(self.y as u16)) as usize];
                if self.a == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                if check_bit(self.a,7) { self.set_bitflag(7,true) }
                else { self.set_bitflag(7,false) } // set negative flag
                self.pc+=0x0002
            },
            // LDA - LoaD Accumulator
            // Loads value into accumulator
            // (Absolute, Y)
            0xB9 =>{
                print!("LDA ${:0>2x}{:0>2x}, y\n",self.cpu_memory[(self.pc+1) as usize],self.cpu_memory[(self.pc+2) as usize]);
                let mut addr = two_u8_to_u16(self.cpu_memory[(self.pc+2) as usize],self.cpu_memory[(self.pc+1) as usize]);
                self.a = self.cpu_memory[(addr.wrapping_add(self.y as u16)) as usize];
                if self.a == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                if check_bit(self.a,7) { self.set_bitflag(7,true) }
                else { self.set_bitflag(7,false) } // set negative flag
                self.pc+=0x0003
            },

            // LDX - LoaD X register
            // Loads value into x register
            // (Immediate)
            0xA2 =>{
                print!("LDX #{:0>2x}\n",self.cpu_memory[(self.pc+1) as usize]);
                self.x = self.cpu_memory[(self.pc+1) as usize];
                let x = self.x.to_owned();
                if self.x == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                if check_bit(x,7) { self.set_bitflag(7,true) }
                else { self.set_bitflag(7,false) } // set negative flag
                self.pc+=0x0002
            },
            // LDX - LoaD X register
            // Loads a byte of memory into the X register setting the zero and negative flags as appropriate.
            // (Absolute)
            0xAE =>{
                print!("LDX ${:0>2x}{:0>2x}\n",self.cpu_memory[(self.pc+1) as usize],self.cpu_memory[(self.pc+2) as usize]);
                let addr = two_u8_to_u16(self.cpu_memory[(self.pc+2) as usize],self.cpu_memory[(self.pc+1) as usize]);
                self.x = self.cpu_memory[addr as usize];
                let x = self.x.to_owned();
                if self.x == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                if check_bit(x,7) { self.set_bitflag(7,true) }
                else { self.set_bitflag(7,false) } // set negative flag
                self.pc+=0x0003
            },
            // LDY - LoaD Y register
            // Loads value into y register
            // (Immediate)
            0xA0 =>{
                print!("LDY #{:0>2x}\n",self.cpu_memory[(self.pc+1) as usize]);
                self.y = self.cpu_memory[(self.pc+1) as usize];
                if self.y == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                self.pc+=0x0002
            },
            // JMP - Jump
            // Jumps to location in memory
            // (Absolute)
            0x4C =>{
                print!("JMP ${:0>2x}{:0>2x}\n",self.cpu_memory[(self.pc+1) as usize],self.cpu_memory[(self.pc+2) as usize]);
                let mut addr = two_u8_to_u16(self.cpu_memory[(self.pc+2) as usize], self.cpu_memory[(self.pc+1) as usize]);
                self.pc = addr;
            },
            // JSR - Jump to Subroutine
            // Jump to New Location Saving Return Address
            // (Absolute)
            0x20 =>{
                print!("JSR ${:0>2x}{:0>2x}\n",self.cpu_memory[(self.pc+1) as usize],self.cpu_memory[(self.pc+2) as usize]);
                let mut addr = two_u8_to_u16(self.cpu_memory[(self.pc+2) as usize], self.cpu_memory[(self.pc+1) as usize]);
                let mut to_stack = self.pc.to_owned() + 3;
                self.push_to_stack(to_stack as u8);
                self.push_to_stack((to_stack  >> 8) as u8);
                self.pc = addr;
            },
            // RTS - Return to Subroutine
            // The RTS instruction is used at the end of a subroutine to return to the calling routine. It pulls the program counter (minus one) from the stack.
            // (Implied)
            0x60 =>{
                print!("RTS\n");
                let mut addr = two_u8_to_u16(self.pop_from_stack(),self.pop_from_stack());
                self.pc = addr;
            },
            // JSR - Jump to Subroutine
            // Jump to New Location Saving Return Address
            // (Absolute)
            0x20 =>{
                print!("JSR ${:0>2x}{:0>2x}\n",self.cpu_memory[(self.pc+1) as usize],self.cpu_memory[(self.pc+2) as usize]);
                let mut addr = two_u8_to_u16(self.cpu_memory[(self.pc+2) as usize], self.cpu_memory[(self.pc+1) as usize]);
                let mut to_stack = self.pc.to_owned();
                self.push_to_stack(to_stack as u8);
                self.push_to_stack((to_stack >> 4) as u8);
                self.pc = addr;
            },
            // AND - bitwise function
            // performs bitwise AND with accumulator
            // (Immediate)
            0x29 =>{
                print!("AND #{:0>2x}\n",self.cpu_memory[(self.pc+1) as usize]);
                self.a = self.a & self.cpu_memory[(self.pc+1) as usize];
                if self.a == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                self.pc+=0x0002
            },
            // AND - bitwise function
            // performs bitwise AND with accumulator
            // (Absolute, X)
            0x3d =>{
                print!("AND ${:0>2x}{:0>2x}\n",self.cpu_memory[(self.pc+2) as usize],self.cpu_memory[(self.pc+1) as usize]);

                let addr:u16 = two_u8_to_u16(self.cpu_memory[(self.pc+2) as usize],self.cpu_memory[(self.pc+1) as usize]).wrapping_add(self.x as u16);

                self.a = self.a & self.cpu_memory[addr as usize];

                if self.a == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                self.pc+=0x0003
            },
            // SBC -  Subtract with Carry
            // This instruction subtracts the contents of a memory location to the accumulator together with the not of the carry bit. If overflow occurs the carry bit is clear, this enables multiple byte subtraction to be performed.
            // (Immediate)
            0xE9 =>{
                print!("SBC #{:0>2x}\n",self.cpu_memory[(self.pc+1) as usize]);
                self.a = self.a & self.cpu_memory[(self.pc+1) as usize];
                if self.a == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                self.pc+=0x0002
            },
            // PLA - PuLl to Accumulator
            // Pulls byte from stack to Accumulator
            // (Implied)
            0x68 =>{
                print!("PLA\n");
                self.a = self.pop_from_stack();
                if self.a == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                if check_bit(self.a,7) { self.set_bitflag(7,true) }
                else { self.set_bitflag(7,false) } // set negative flag
                self.pc+=0x0001
            },
            // INC - INCriment Memory
            // Adds one to the value held at a specified memory location setting the zero and negative flags as appropriate.
            // (Zero Page)
            0xe6 =>{
                print!("INC ${:0>2x}\n",self.cpu_memory[self.pc as usize+1]);
                let mut addr = self.cpu_memory[self.pc as usize+1];
                self.cpu_memory[addr as usize].wrapping_add(1);
                if self.cpu_memory[addr as usize] == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                if check_bit(self.cpu_memory[addr as usize],7) { self.set_bitflag(7,true) }
                else { self.set_bitflag(7,false) } // set negative flag
                self.pc+=0x0002
            },
            // INX - INCriment X Register
            // Adds one to the X register setting the zero and negative flags as appropriate.
            // (Implied)
            0xe8 =>{
                print!("INX\n");
                self.x.wrapping_add(1);
                if self.x == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                if check_bit(self.x,7) { self.set_bitflag(7,true) }
                else { self.set_bitflag(7,false) } // set negative flag
                self.pc+=0x0001
            },
            // BIT - BIt Test
            // This instructions is used to test if one or more bits are set in a target memory location. The mask pattern in A is ANDed with the value in memory to set or clear the zero flag, but the result is not kept. Bits 7 and 6 of the value from memory are copied into the N and V flags.
            // (Absolute)
            0x2c =>{
                print!("BIT ${:0>2x}{:0>2x}\n",self.cpu_memory[self.pc as usize+1],self.cpu_memory[self.pc as usize+2]);
                let addr = two_u8_to_u16(self.cpu_memory[self.pc as usize+2],self.cpu_memory[self.pc as usize+1]);
                let val = self.cpu_memory[addr as usize];

                if val & self.a == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,false) } // set zero flag
                if check_bit(val,6) { self.set_bitflag(6,true) }
                else { self.set_bitflag(6,false) } // set overflow flag
                if check_bit(val,7) { self.set_bitflag(7,true) }
                else { self.set_bitflag(7,false) } // set negative flag
                self.pc+=0x0003
            },
            // CMP - Compare Memory and Accumulator
            // This instruction compares the contents of the accumulator with another memory held value and sets the zero and carry flags as appropriate.
            // (Immediate)
            0xC9 => {
                print!("CMP #{:0>2x}\n",self.cpu_memory[(self.pc+1) as usize]);
                let res = self.a.wrapping_sub(self.cpu_memory[(self.pc+1) as usize]);
                if res >= 0 { self.set_bitflag(0,true) }
                else { self.set_bitflag(0,true) } // set carry flag
                if res == 0 { self.set_bitflag(1,true) }
                else { self.set_bitflag(1,true) } // set zero flag
                if check_bit(res,7) { self.set_bitflag(7,true) }
                else { self.set_bitflag(7,true) } // set negative flag
                self.pc+=0x0002;
            },
            // BCC - branch if carry clear
            // branches if carry flag is clear
            // (Relative)
            0x90 => {
                print!("BCC #{:0>2x}\n",self.cpu_memory[(self.pc+1) as usize]);
                if check_bit(self.p, 0) {
                    self.pc+=0x0002;
                }
                else {
                    self.pc = self.pc.wrapping_add(self.cpu_memory[(self.pc+1) as usize] as u16 - 1)
                }
            },
            // BEQ - branch on equal
            // branches if last result was equal
            // (Relative)
            0xF0 => {
                print!("BEQ #{:0>2x}\n",self.cpu_memory[(self.pc+1) as usize]);
                if check_bit(self.p, 1) {
                    self.pc = self.pc.wrapping_add(self.cpu_memory[(self.pc+1) as usize] as u16 - 1)
                }
                else {
                    self.pc+=0x0002;
                }
            },
            // BNE - branch not equal
            // If the zero flag is clear then add the relative displacement to the program counter to cause a branch to a new location.
            // (Immediate)
            0xD0 => {
                print!("BNE #{:0>2x}\n",self.cpu_memory[(self.pc+1) as usize]);
                if !check_bit(self.p, 1) {
                    self.pc = self.pc.wrapping_add(self.cpu_memory[(self.pc+1) as usize] as u16 - 1)
                }
                else {
                    self.pc+=0x0002;
                }
            },
            // STA - STore Accumulator
            // Stores Accumulator into Memory
            // (Absolute)
            0x8d =>{
                print!("STA ${:0>2x}{:0>2x}\n",self.cpu_memory[(self.pc+1) as usize],self.cpu_memory[(self.pc+2) as usize]);
                let addr = two_u8_to_u16(self.cpu_memory[(self.pc+2) as usize], self.cpu_memory[(self.pc+1) as usize]);
                self.cpu_memory[addr as usize] = self.a;
                self.pc+=0x0003
            },
            // STA - STore Accumulator
            // Stores Accumulator into Memory
            // (Absolute, y)
            0x99 =>{
                print!("STA (${:0>2x}{:0>2x}), y\n",self.cpu_memory[(self.pc+1) as usize],self.cpu_memory[(self.pc+2) as usize]);
                let mut addr = two_u8_to_u16(self.cpu_memory[(self.pc+2) as usize], self.cpu_memory[(self.pc+1) as usize]);
                        addr = addr.wrapping_add(self.y as u16);
                self.cpu_memory[addr as usize] = self.a;
                self.pc+=0x0003
            },
            // STA - STore Accumulator
            // Stores Accumulator into Memory
            // (Absolute, y)
            0x99 =>{
                print!("STA (${:0>2x}{:0>2x}), y\n",self.cpu_memory[(self.pc+1) as usize],self.cpu_memory[(self.pc+2) as usize]);
                let mut addr = two_u8_to_u16(self.cpu_memory[(self.pc+2) as usize], self.cpu_memory[(self.pc+1) as usize]);
                        addr = addr.wrapping_add(self.y as u16);
                self.cpu_memory[addr as usize] = self.a;
                self.pc+=0x0003
            },
            // BPL - Branch on result PLus
            // Branches if Negative flag == 0
            // (Relative)
            0x10=>{
                print!("BPL ${:0>2x}\n",self.cpu_memory[(self.pc+1) as usize]);
                if check_bit(self.p,7) { self.pc = self.pc.wrapping_add(self.cpu_memory[(self.pc+1) as usize] as u16 - 1) }
                else { self.pc += 0x0002 }
            }
            // STA - STore Accumulator
            // Stores Accumulator into Memory
            // (Indirect, Y)
            0x91 =>{
                print!("STA (${:0>2x}),y\n",self.cpu_memory[(self.pc+1) as usize]);
                let zp = self.cpu_memory[(self.pc+1) as usize];
                let addr1 = self.cpu_memory[two_u8_to_u16(0,zp) as usize];
                let addr2 = self.cpu_memory[two_u8_to_u16(0,zp+1) as usize];
                self.a = self.cpu_memory[(two_u8_to_u16(addr2,addr1) + (self.y as u16)) as usize];
                self.pc+=0x0002
            },
            // STA - STore Accumulator
            // Stores Accumulator into Memory
            // (Zero Page)
            0x85 =>{
                print!("STA ${:0>2x}\n",self.cpu_memory[(self.pc+1) as usize]);
                self.cpu_memory[(self.pc+1) as usize] = self.a;
                self.pc+=0x0002
            },
            // STA - STore Accumulator
            // Stores Accumulator into Memory
            // (Absolute, X)
            0x9D =>{
                print!("STA ${:0>2x}{:0>2x}, x\n",self.cpu_memory[(self.pc+1) as usize],self.cpu_memory[(self.pc+2) as usize]);
                self.cpu_memory[two_u8_to_u16(self.cpu_memory[(self.pc+2) as usize],self.cpu_memory[(self.pc+1).wrapping_add(self.x as u16) as usize]) as usize] = self.a;
                self.pc+=0x0003
            },
            // STY - STore Y register
            // Stores Y register into Memory
            // (Zero Page)
            0x84 =>{
                print!("STY ${:0>2x}\n",self.cpu_memory[(self.pc+1) as usize]);
                let addr = two_u8_to_u16(0, self.cpu_memory[(self.pc+1) as usize]);
                self.cpu_memory[addr as usize] = self.y;
                self.pc+=0x0002
            },
            // TXS - Transfer X to Stack pointer
            // Stores value of x in memory at the stack pointer
            // (Implied)
            0x9a =>{
                print!("TXS\n");
                let x = self.x.to_owned();
                self.cpu_memory[self.sp as usize] = x;
                self.pc+=0x0001
            },
            // TXA - Transfer X to Accumulator
            // Copies the current contents of the X register into the accumulator and sets the zero and negative flags as appropriate.
            // (Implied)
            0x8a =>{
                print!("TXA\n");
                let x = self.x.to_owned();
                self.a = x;
                self.pc+=0x0001
            },
            // DEY - DEcrement Y register
            // Decrements Y register by one
            // (Implied)
            0x88 =>{
                print!("DEY\n");
                if self.y == 0 { self.y = 0xff }
                else { self.y-=1; }

                if self.y == 0 { self.set_bitflag(1,true); }
                else { self.set_bitflag(1,false); }

                if self.y >= 0x80 { self.set_bitflag(7,true); }
                else { self.set_bitflag(7,false); }

                self.pc+=0x0001
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
                let mut addr = two_u8_to_u16(self.cpu_memory[(self.pc+2) as usize], self.cpu_memory[(self.pc+1) as usize]);

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
                        addr = self.cpu_memory[(addr.wrapping_add(self.x)) as usize];

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
                        addr = self.cpu_memory[(addr.wrapping_add(self.x)) as usize];

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
                        addr = self.cpu_memory[(addr.wrapping_add(self.x)) as usize];

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
            // SEI - SEt Interupt
            // Sets Interupt Flag
            // (Implied)
            0x78 => {
                print!("SEI\n");
                self.set_bitflag(2,true);
                self.pc+=0x0001; // next instruction
            },
            // CLC - CLear Carry
            // Clears Carry Flag
            // (Implied)
            0x18 => {
                print!("CLC\n");
                self.set_bitflag(0,false);
                self.pc+=0x0001; // next instruction
            },
            // CLD - CLear Decimal
            // Clears Decimal Flag
            // (Implied)
            0xD8 => {
                print!("CLD\n");
                self.set_bitflag(4,false);
                self.pc+=0x0001; // next instruction
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
            // PLP - PuLl Processor status
            // Pulls an 8 bit value from the stack and into the processor flags. The flags will take on new states as determined by the value pulled.
            // (Implied)
            0x28 => {
                print!("PLP\n");
                self.p = self.pop_from_stack();
                self.pc+=0x0001; // next instruction
            },
            // DEC - Decrement Memory
            // Subtracts one from the value held at a specified memory location setting the zero and negative flags as appropriate.
            // (Zero Page)
            0xc6 => {
                print!("DEC ${:0>2x}\n",self.cpu_memory[self.pc as usize+1]);
                let addr = self.cpu_memory[self.pc as usize+1];
                let res = self.cpu_memory[addr as usize].wrapping_sub(1);
                if res == 0x00 { self.set_bitflag(1,true); }
                else { self.set_bitflag(1,false); } // set zero flag
                if res >= 0b10000000 { self.set_bitflag(7,true); }
                else { self.set_bitflag(7,false); } // set negative flag
                self.pc += 0x0002; // next instruction
            },
            // DEC - Decrement Memory
            // Subtracts one from the value held at a specified memory location setting the zero and negative flags as appropriate.
            // (Absolute)
            0xce => {
                print!("DEC ${:0>2x}{:0>2x}\n",self.cpu_memory[self.pc as usize+1],self.cpu_memory[self.pc as usize+2]);
                let addr = two_u8_to_u16(self.cpu_memory[self.pc as usize+2],self.cpu_memory[self.pc as usize+1]);
                let res = self.cpu_memory[addr as usize].wrapping_sub(1);
                if res == 0x00 { self.set_bitflag(1,true); }
                else { self.set_bitflag(1,false); } // set zero flag
                if res >= 0b10000000 { self.set_bitflag(7,true); }
                else { self.set_bitflag(7,false); } // set negative flag
                self.pc += 0x0003; // next instruction
            },
            // ASL - Arithmic Shift Left
            // This operation shifts all the bits of the accumulator or memory contents one bit left. Bit 0 is set to 0 and bit 7 is placed in the carry flag. The effect of this operation is to multiply the memory contents by 2 (ignoring 2's complement considerations), setting the carry if the result will not fit in 8 bits.
            // (Accumulator)
            0x0a => {
                print!("ASL A\n");
                self.a = self.a << 1;
                let temp = self.p;
                self.set_bitflag(0,check_bit(temp,7)); // set carry flag
                if self.a == 0 { self.set_bitflag(1,true); }
                else { self.set_bitflag(1,true); } // set zero flag
                if self.a >= 0b10000000 { self.set_bitflag(7,true); }
                else { self.set_bitflag(7,true); } // set negative flag
                self.pc += 1; // next instruction
            },

            // LSR - Logical Shift Right
            // Each of the bits in A or M is shift one place to the right. The bit that was in bit 0 is shifted into the carry flag. Bit 7 is set to zero.
            // (Zero Page)
            0x46 => {
                print!("LSR ${:0>2x}\n",self.cpu_memory[self.pc as usize+1]);
                let addr = self.cpu_memory[self.pc as usize+1];
                let mut val = self.cpu_memory[addr as usize];

                self.set_bitflag(0,check_bit(val,0)); // set carry flag
                val = val >> 1; // shift right 1 bit
                if val == 0 { self.set_bitflag(1,true); }
                else { self.set_bitflag(1,true); } // set zero flag
                if self.a >= 0b10000000 { self.set_bitflag(7,true); }
                else { self.set_bitflag(7,true); } // set negative flag
                self.pc += 2; // next instruction
            },
            // LSR - Logical Shift Right
            // Each of the bits in A or M is shift one place to the right. The bit that was in bit 0 is shifted into the carry flag. Bit 7 is set to zero.
            // (Absolute)
            0x4e => {
                print!("LSR ${:0>2x}\n",self.cpu_memory[self.pc as usize+1]);
                let addr = two_u8_to_u16(self.cpu_memory[self.pc as usize+2],self.cpu_memory[self.pc as usize+1]);
                let mut val = self.cpu_memory[addr as usize];

                self.set_bitflag(0,check_bit(val,0)); // set carry flag
                val = val >> 1; // shift right 1 bit
                if val == 0 { self.set_bitflag(1,true); }
                else { self.set_bitflag(1,true); } // set zero flag
                if self.a >= 0b10000000 { self.set_bitflag(7,true); }
                else { self.set_bitflag(7,true); } // set negative flag
                self.pc += 3; // next instruction
            },

            // BRK - Force Interrupt
            // The BRK instruction forces the generation of an interrupt request. The program counter and processor status are pushed on the stack then the IRQ interrupt vector at $FFFE/F is loaded into the PC and the break flag in the status set to one.
            // (Implied)
            0x00 => {
                print!("BRK\n",);

                    //TODO
                wait(3);
                self.pc += 0; // next instruction
            }
            // Default
            _ => {
                print!("${:0>2x}\n",self.cpu_memory[self.pc as usize]);
                wait(10);
                self.pc+=0x0001; // next instruction
            }
        }

    }

    fn push_to_stack(&mut self, v:u8){
        self.cpu_memory[0x01FF - self.sp as usize] = v;
        self.sp+=1; //incriment stack pointer
    }

    fn pop_from_stack(&mut self) -> u8{
        self.sp-=1; //decriment stack pointer
        return(self.cpu_memory[0x01FF - self.sp as usize]);
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

fn two_u8_to_u16(a:u8,b:u8) -> u16 {
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
