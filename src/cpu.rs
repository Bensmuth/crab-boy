use std::process::exit;

#[derive(PartialEq, PartialOrd, Copy, Clone)]
enum RegisterTarget{
    A, B, C, D, E, H, L, AF, BC, DE, HL, SP, PC, MemoryAdress(u16), Value(u16), UNKNOWN
}

impl From<u8> for RegisterTarget{
    fn from(orig: u8) -> Self {
        return match orig {
            0x7 => RegisterTarget::A,
            0x0 => RegisterTarget::B,
            0x1 => RegisterTarget::C,
            0x2 => RegisterTarget::D,
            0x3 => RegisterTarget::E,
            0x4 => RegisterTarget::H,
            0x5 => RegisterTarget::L,
            0x6 => RegisterTarget::HL,
            _ => RegisterTarget::UNKNOWN,
        };
    }
}


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

#[derive(Debug, Copy, Clone)]
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
        Registers {a, f, b, c, d, e, h, l, sp, pc }
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




    // * reg can be combined as AF, BC, DE, HL
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

#[derive(Debug, Copy, Clone)]
pub struct Cpu {
    reg: Registers,
    opcode: u8,
    mem: super::memory::Memory

}

impl Cpu {
    pub fn new(reg : Registers, operation : u8, mem : super::memory::Memory) -> Cpu{
        let mut new_cpu = Cpu { reg, opcode : operation, mem};
        // pseudo bios - CHECK THIS, IM NOT SURE IF IT WORKS
        new_cpu.reg.pc = 0x100;
        new_cpu.reg.set_af(0x01B0);
        new_cpu.reg.set_bc(0x0013);
        new_cpu.reg.set_de(0x00D8);
        new_cpu.reg.set_hl(0x014D);
        new_cpu.reg.sp = 0xFFFF;
        new_cpu.reg.set_flag(Flag::Z, true);
        new_cpu.reg.set_flag(Flag::H, true);
        new_cpu.reg.set_flag(Flag::C, true);

        // lcdc set to 91
        // stat 85
        // lv 00
        // cnt 28
        // ie 00
        // if e1
        // spd 0
        // rom 1
        // TODO implement all these and the ime, see https://gbdev.gg8.se/wiki/ for more info

        new_cpu


    }

    fn get_target(&self, register : RegisterTarget) -> u16{ // return u16 cause of special registers, TODO implement for all combined registers
        match register {
            RegisterTarget::A => {self.reg.a.into()}
            RegisterTarget::B => {self.reg.b.into()}
            RegisterTarget::C => {self.reg.c.into()}
            RegisterTarget::D => {self.reg.d.into()}
            RegisterTarget::E => {self.reg.e.into()}
            RegisterTarget::H => {self.reg.h.into()}
            RegisterTarget::L => {self.reg.l.into()}
            RegisterTarget::HL => {self.reg.get_hl()}
            RegisterTarget::AF => {self.reg.get_af()},
            RegisterTarget::BC => {self.reg.get_bc()},
            RegisterTarget::DE => {self.reg.get_de()},
            RegisterTarget::SP => {self.reg.sp},
            RegisterTarget::MemoryAdress(adress) => {self.mem.clone().get(adress).into()},
            RegisterTarget::Value(val) => {val},
            _ => panic!(),

        }
    }

    fn set_target(&mut self, register: RegisterTarget, value : u16) {
        match register {
            RegisterTarget::A => {self.reg.a = value as u8}
            RegisterTarget::B => {self.reg.b = value as u8}
            RegisterTarget::C => {self.reg.c = value as u8}
            RegisterTarget::D => {self.reg.d = value as u8}
            RegisterTarget::E => {self.reg.e = value as u8}
            RegisterTarget::H => {self.reg.h = value as u8}
            RegisterTarget::L => {self.reg.l = value as u8}
            RegisterTarget::HL => {self.reg.set_hl(value)}
            RegisterTarget::AF => {self.reg.set_af(value)},
            RegisterTarget::BC => {self.reg.set_bc(value)},
            RegisterTarget::DE => {self.reg.set_de(value)},
            RegisterTarget::SP => {self.reg.sp = value},
            RegisterTarget::MemoryAdress(adress) => {self.mem.set(adress, value as u8)},
            _ => panic!()

        }
    }

    pub fn get_register_debug_string(&mut self) -> String{
        format!("PC: {:x}, OpCode {:x} \nRegisters \na: {:x}, f: {:x} \nb: {:x} c: {:x} \nd: {:x} e:{:x} \nh:{:x} l: {:x} \nsp: {:x} pc: {:x}", self.reg.pc, self.mem.clone().get(self.reg.pc), self.reg.a, self.reg.f, self.reg.b, self.reg.c, self.reg.d, self.reg.e, self.reg.h, self.reg.l, self.reg.sp, self.reg.pc)
    }

    pub fn get_memory_debug(&mut self) -> super::memory::Memory {
        self.mem
    }
    
    fn pcc(&mut self) -> u8{ //program counter call, use this whenever iterating pc and wanting to get something from memory in that iteration
        let v = self.mem.get(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(1);
        v
    }

    #[inline(always)]
    fn pointer_convert(&self, target: RegisterTarget) -> RegisterTarget { // converts (hl) pointers to my method of pointers (works for other pointers)
        RegisterTarget::MemoryAdress(self.get_target(target))
    }
    #[inline(always)]
    fn get_target_hl(&mut self, target: RegisterTarget) -> u8{
        return if target == RegisterTarget::HL{
            self.get_target(RegisterTarget::MemoryAdress(self.get_target(RegisterTarget::HL))) as u8
        } else {
            self.get_target(target) as u8
        }
    }

    #[inline(always)]
    fn get_a_and_target_hl(&mut self, target : RegisterTarget) -> (u8, u8) { // okay this function is kinda pointless now but i used it for most of the alu instructions so whatever
        let reg_a_value = self.reg.a;
        let target_value = self.get_target_hl(target);
        return (reg_a_value, target_value as u8)
    }
    #[inline(always)]
    fn inc_target(&mut self, target : RegisterTarget) {
        self.set_target(target, self.get_target(target).wrapping_add(1))
    }
    #[inline(always)]
    fn dec_target(&mut self, target : RegisterTarget) {
        self.set_target(target, self.get_target(target).wrapping_sub(1))
    }

    
    // Instructions
    // Load x value into X
    fn ld_X_x(&mut self, mut X : RegisterTarget, x: RegisterTarget) {
        let target_value = self.get_target_hl(x) as u8;
        if X == RegisterTarget::HL{
            X = self.pointer_convert(RegisterTarget::HL);
        }
        self.set_target(X, target_value.into());
    }
    // ADD x register to A instructions, add u8 to A is seperate
    // Carry represents the ADC instructions
    fn add_A_x(&mut self, target: RegisterTarget, carry: bool){
        let (reg_a_value, mut target_value) = self.get_a_and_target_hl(target);
        if carry {
            target_value = target_value.wrapping_add(self.reg.get_flag(Flag::C) as u8);
        }

        let zero_check = (reg_a_value.wrapping_add(target_value)) == 0;
        let half_carry = (((reg_a_value & 0xf).wrapping_add(target_value & 0xf)) & 0x10) == 0x10; // might not be meant use a wrapping add here, who knows
        let carry_check = reg_a_value.checked_add(target_value) == None;
        self.reg.set_flag(Flag::Z,zero_check);
        self.reg.set_flag(Flag::H,half_carry);
        self.reg.set_flag(Flag::C,carry_check);
        
        self.reg.a.wrapping_add(target_value);
    }
    // SUB x register to A instructions.
    fn sub_A_x(&mut self, target: RegisterTarget, carry: bool){
        let (reg_a_value, mut target_value) = self.get_a_and_target_hl(target);
        if carry { // this can be simplified at the sacrifice of less 'readable' code
            target_value = target_value.wrapping_sub(self.reg.get_flag(Flag::C) as u8);
        }

        let zero_check = (reg_a_value.wrapping_sub(target_value)) == 0;
        let half_carry = (((reg_a_value & 0xf).wrapping_sub(target_value & 0xf)) & 0x10) == 0x10; // might not be meant use a wrapping sub here, who knows (THIS COULD BE WRONG)
        let carry_check = reg_a_value.checked_sub(target_value) == None;
        self.reg.set_flag(Flag::Z,zero_check);
        self.reg.set_flag(Flag::H,half_carry);
        self.reg.set_flag(Flag::N, true);
        self.reg.set_flag(Flag::C,carry_check);

        self.reg.a.wrapping_sub(target_value);
    }
    // AND x register with A
    fn and_A_x(&mut self, target: RegisterTarget){
        let (reg_a_value, mut target_value) = self.get_a_and_target_hl(target);

        
        self.reg.a = reg_a_value & target_value;
        self.reg.set_flag(Flag::Z, self.reg.a == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::C, false);
        self.reg.set_flag(Flag::H, true);

    }
    // XOR x register with A
    fn xor_A_x(&mut self, target: RegisterTarget){
        let (reg_a_value, mut target_value) = self.get_a_and_target_hl(target);

        self.reg.a = reg_a_value ^ target_value;
        self.reg.set_flag(Flag::Z, self.reg.a == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::C, false);
        self.reg.set_flag(Flag::H, false);
    }
    // OR x register with A
    fn or_A_x(&mut self, target: RegisterTarget){
        let (reg_a_value, mut target_value) = self.get_a_and_target_hl(target);

        self.reg.a = reg_a_value | target_value;
        self.reg.set_flag(Flag::Z, self.reg.a == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::C, false);
        self.reg.set_flag(Flag::H, false);
    }
    // Compare x register with A, essentially sub in which we throw away the result
    fn cp_A_x(&mut self, target: RegisterTarget){
        let reg_a_value = self.reg.a;
        self.sub_A_x(target, false);
        self.reg.a = reg_a_value;
    }
        
    

    pub fn tick(&mut self) -> Cpu{
        /*
        Aight so theres a little issue here, the bios runs for a bit then it
        moves into uninitialized memory and NOP slides to the end of the
        memory array causing an overflow. Its something to do with
        instruction 0x20 and 0xcb -> 0x7c. Debug the code to see what i mean.
        I have no idea how to fix this, thats for post exam Ben!
        I should probs get a gameboy debugger and compare my behavour to that of the debugger
        */
        fn nibc(unib : u8, lnib : u8) -> u16 { //combines 2 nibbles, upper nibble and lower nibble - USING THE WRONG TERMINOLOGY HERE, ITS NOT NIBBLES ITS BYTES CAUSE WE DOING A SPECIAL LOAD etc
            (lnib as u16) << 8 | unib as u16
        }
        self.opcode = self.pcc();

        match self.opcode { // * massive switch that implements all the cpu functions
            // ! perhaps move each instruction into its own function, will shorten code, but at cost of increased complexity
            0x0 => println!("NOP at {:x}", self.reg.pc), // No Operation (NOP)
            0x01 => { // LD BC, u16
                self.reg.c = self.pcc();
                self.reg.b = self.pcc();
            }
            0x02 => self.set_target(self.pointer_convert(RegisterTarget::BC), self.reg.a.into()), // LD (BC), A
            0x03 => self.inc_target(RegisterTarget::BC),
            0x04 => self.inc_target(RegisterTarget::B),
            0x05 => self.dec_target(RegisterTarget::B),
            0x06 => {
                let value = RegisterTarget::Value(self.pcc().into());
                self.ld_X_x(RegisterTarget::B, value);
            }
            0x10 => (exit(1)), // STOP
            0x11 => { // LD DE, u16
                self.reg.e = self.pcc();
                self.reg.d = self.pcc();
            }
            0x12 => self.set_target(self.pointer_convert(RegisterTarget::DE), self.reg.a.into()),  // LD (DE), A
            0x13 => self.inc_target(RegisterTarget::DE),
            0x14 => self.inc_target(RegisterTarget::D),
            0x15 => self.dec_target(RegisterTarget::D),
            0x21 =>{ // LD HL, u16
                self.reg.l = self.pcc();
                self.reg.h = self.pcc();
            },
            0x22 => {  // LD (HL+), A
                self.set_target(self.pointer_convert(RegisterTarget::HL), self.reg.a.into());
                self.inc_target(RegisterTarget::HL);
            },
            0x23 => self.inc_target(RegisterTarget::HL),
            0x24 => self.inc_target(RegisterTarget::H),
            0x25 => self.dec_target(RegisterTarget::H),
            0x31 => { //LD SP, u16
                let unib = self.pcc();
                let lnib = self.pcc();
                self.reg.sp = nibc(unib, lnib);

            },
            0x32 =>{ // LD (HL-), A
                self.dec_target(RegisterTarget::HL);
                self.set_target(self.pointer_convert(RegisterTarget::HL), self.reg.a.into());
            },
            0x33 => self.inc_target(RegisterTarget::SP),
            0x34 => self.inc_target(self.pointer_convert(RegisterTarget::HL)),
            0x35 => self.dec_target(self.pointer_convert(RegisterTarget::HL)),
            0x76 => { todo!()} // power down the cpu until an interrupt occurs
            0x40..=0x7f => {
                let first_register : RegisterTarget = ((self.opcode - 0x40) / 8).into();
                let target_register : RegisterTarget = (self.opcode & 0b0000_1111).into();
                self.ld_X_x(first_register, target_register);
            }
            0x80..=0x87 => {
                let register = (self.opcode & 0b0000_1111).into();
                self.add_A_x(register, false)
                
            }
            0x88..=0x8f => {
                let register = ((self.opcode - 0x8) & 0b0000_1111).into();
                self.add_A_x(register, true)
            }
            0x90..=0x97 => {
                let register = (self.opcode & 0b0000_1111).into();
                self.sub_A_x(register, false);
            }
            0x98..=0x9f => {
                let register = ((self.opcode - 0x8) & 0b0000_1111).into();
                self.sub_A_x(register, true);
            }
            0xA0..=0xA7 => {
                let register = (self.opcode & 0b0000_1111).into();
                self.and_A_x(register);
            }
            0xA8..=0xAF => {
                let register = ((self.opcode -0x8) & 0b0000_1111).into();
                self.xor_A_x(register);
            }
            0xB0..=0xB7 =>{
                let register = ((self.opcode -0x8) & 0b0000_1111).into();
                self.or_A_x(register);
            }

            0xc3 => { // jump to nn, set pc to nn
                let unib = self.pcc();
                let lnib = self.pcc();
                self.reg.pc = nibc(unib, lnib);
            }
            0xcb =>{ // * prefixes
                self.opcode = self.pcc(); 
                match self.opcode{
                    0x7c => { // checks to see if the 7th value of h is set, if so flag z is set to false. C is preserved, n is reset, h is 
                        if self.reg.h & 0b1000_0000 == 0b1000_0000{ 
                            self.reg.set_flag(Flag::Z, false)
                        } else {
                            self.reg.set_flag(Flag::Z, true)
                        }
                        self.reg.set_flag(Flag::N, false);
                        self.reg.set_flag(Flag::H, false);

                    }
                    _ => {
                        println!("Unimplemented prefixed instruction {:x}", self.mem.get(self.reg.pc-1)); // * {:x} to output as hex
                        println!("Register Dump as hex: ");
                        println!("a: {:x}, f: {:x} \nb: {:x} c: {:x} \nd: {:x} e:{:x} \nh:{:x} l: {:x} \nsp: {:x} pc: {:x}", self.reg.a, self.reg.f, self.reg.b, self.reg.c, self.reg.d, self.reg.e, self.reg.h, self.reg.l, self.reg.sp, self.reg.pc);
                        exit(0);
                    }
                }
            },
            _ => {
                println!("Unimplemented instruction {:x}", self.opcode);
                println!("Register Dump as hex: ");
                println!("a: {:x}, f: {:x} \nb: {:x} c: {:x} \nd: {:x} e:{:x} \nh:{:x} l: {:x} \nsp: {:x} pc: {:x}", self.reg.a, self.reg.f, self.reg.b, self.reg.c, self.reg.d, self.reg.e, self.reg.h, self.reg.l, self.reg.sp, self.reg.pc);

                exit(0);
            }
        }
        return *self;
    }
}
