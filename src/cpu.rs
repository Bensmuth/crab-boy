use std::process::exit;

pub struct Registers{
    a: u8, // accumulator
    f: u8, // flag

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

    // registers can be combined as AF, BC, DE, HL
    pub fn get_af(&self) -> u16 {
        ((self.a as u16) << 8 | (self.f as u16)) // << is bitshifting and | performs or on bit values
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


pub struct Operation{
    prefix: u8,
    opcode: u8,
    operand1: u8,
    operand2: u8
}

impl Operation{
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
        match memory.memory[self.registers.pc as usize] {
            0x31 => { //LD SP 
                self.registers.sp = (memory.memory[(self.registers.pc + 2) as usize] as u16) << 8 | (memory.memory[(self.registers.pc + 1) as usize] as u16);
                self.registers.pc += 2;
            },
            0xaf => //XOR A
            _ => {
                println!("Unimplemented instruction {:x}", memory.memory[self.registers.pc as usize]);
                exit(0);
            }
        }
        self.registers.pc += 1;
        return memory
    }
}