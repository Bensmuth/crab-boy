use std::process::exit;

pub enum Flag {
    // Zero Flag. This bit is set when the result of a math operationis zero or two values match when using the CP
    // instruction.
    Z = 0b1000_0000,
    // Subtract Flag. This bit is set if a subtraction was performed in the last math instruction.
    N = 0b0100_0000,
    // Half Carry Flag. This bit is set if a carry occurred from the lowernibble in the last math operation.
    H = 0b0010_0000,
    // Carry Flag. This bit is set if a carry occurred from the last math operation or if register A is the smaller
    // value when executing the CP instruction.
    C = 0b0001_0000,
}

impl Flag { 
    pub fn og(self) -> u8{
        self as u8 // ensures bits arent returned
    }
    pub fn bw(self) -> u8{
        !self.og()
    }
}

pub struct Registers{
    a: u8, // accumulator
    f: u8, // flag, indirectly accescable

    b: u8,
    c: u8,
    
    d: u8,
    e: u8,

    h: u8,
    l: u8,

    sp: u16, // stack pointer
    pc: u16, // program counter
    
}

impl Registers{
    pub fn new(a : u8, f : u8, b : u8, c : u8, d : u8, e : u8, h : u8, l : u8, sp : u16, pc : u16) -> Registers{
        Registers {a : a, f:f, b:b, c:c, d:d, e:e, h:h, l:l, sp:sp, pc:pc }
    }

    pub fn get_flag(&self, f:Flag) -> bool{
        (self.f & f as u8) != 0
    }

    pub fn set_flag(&mut self, f: Flag, v: bool){
        if v{
            self.f |= f.og(); // | is or operator, sets flag 
        }else {
            self.f &= f.bw(); // & is and operator, sets flag by merge techniques i think, stolen from mohanson
        }
    }

    // * registers can be combined as AF, BC, DE, HL
    pub fn get_af(&self) -> u16 {
        ((self.a as u16) << 8 | (self.f as u16)) // * << is bitshifting and | performs or on bit values
    }

    pub fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8 | (self.c as u16))
    }

    pub fn get_de(&self) -> u16 {
        ((self.d as u16) << 8 | (self.e as u16))
    }

    pub fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8 | (self.l as u16))
    }

    pub fn set_af(&mut self, input:u16) {
        self.a = (input >> 8) as u8;
        self.f = (input & 0x00f0) as u8
    }

    pub fn set_bc(&mut self, input:u16) {
        self.b = (input >> 8) as u8;
        self.c = (input & 0x00ff) as u8;
    }

    pub fn set_de(&mut self, input:u16) {
        self.d = (input >> 8) as u8;
        self.e = (input & 0x00ff) as u8;
    }

    pub fn set_hl(&mut self, input:u16) {
        self.h = (input >> 8) as u8;
        self.l = (input & 0x00ff) as u8;
    }


}


pub struct Operation{ // ? this might be redundant
    prefix: u8,
    opcode: u8,
    operand1: u8,
    operand2: u8
}

impl Operation{ // ? this might be redundant
    pub fn new(prefix : u8, opcode : u8, operand1 : u8, operand2 : u8) -> Operation{
        Operation {prefix : prefix, opcode : opcode, operand1 : operand1, operand2 : operand2}
    }
}

pub struct Cpu {
    registers: Registers,
    operation: Operation,

}

impl Cpu {
    pub fn new(registers : Registers, operation : Operation) -> Cpu{
        Cpu { registers: registers, operation : operation}
    }

    pub fn tick(&mut self, mut memory : crate::memory::Memory) -> crate::memory::Memory{
        match memory.memory[self.registers.pc as usize] { // * massive switch that implements all the cpu functions
            // ! perhaps move each instruction into its own function, will shorten code, but increased complexity
            0x31 => { //LD SP 
                self.registers.sp = (memory.memory[(self.registers.pc + 2) as usize] as u16) << 8 | (memory.memory[(self.registers.pc + 1) as usize] as u16);
                self.registers.pc += 2;
            },
            0xaf =>{ //XOR A into A, essentially sets A to 0
                self.registers.a = 0; // ! lazy method, but more efficient
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, false);
                self.registers.set_flag(Flag::Z, true)
            },
            0x21 =>{ // put value nn into HL
                self.registers.set_hl((memory.memory[(self.registers.pc + 2) as usize] as u16) << 8 | (memory.memory[(self.registers.pc + 1) as usize] as u16));
                self.registers.pc += 2;

            },
            0x32 =>{ //put value at address HL into A. Decrement HL.
                self.registers.a = memory.memory[self.registers.get_hl() as usize];
                self.registers.set_hl(self.registers.get_hl() - 1);
            },
            0xcb =>{ // swap upper and lower nibles of a
                
            },
            _ => {
                println!("Unimplemented instruction {:x}", memory.memory[self.registers.pc as usize]);
                println!("Register Dump as hex: ");
                println!("a: {:x}, f: {:x} \nb: {:x} c: {:x} \nd: {:x} e:{:x} \nh:{:x} l: {:x} \nsp: {:x} pc: {:x}", self.registers.a, self.registers.f, self.registers.b, self.registers.c, self.registers.d, self.registers.e, self.registers.h, self.registers.l, self.registers.sp, self.registers.pc);

                exit(0);
            }
        }
        if self.registers.pc == 65534{
            self.registers.pc = 0
        } else{
            self.registers.pc += 1;
        }
        return memory
    }
}