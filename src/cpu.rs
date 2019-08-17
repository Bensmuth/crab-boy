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

    pub fn tick(&mut self){

        self.registers.pc += 1
    }
}