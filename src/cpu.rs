use std::process::exit;

//use gtk::BookmarkListBuilder;

#[derive(PartialEq, PartialOrd, Copy, Clone, Debug)]
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
    pub fn new() -> Registers{
        Registers {a:0, f:0, b:0, c:0, d:0, e:0, h:0, l:0, sp:0, pc:0 }
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
    mem: super::memory::Memory,
    interupts_enabled: bool,

}

impl Cpu {
    pub fn new(reg : Registers, mem : super::memory::Memory) -> Cpu{
        let mut new_cpu = Cpu { reg, opcode : 0, mem, interupts_enabled : true};
        // pseudo bios - CHECK THIS, IM NOT SURE IF IT WORKS
        new_cpu.reg.pc = 0x100;
        new_cpu.reg.set_af(0x01B0);
        new_cpu.reg.set_bc(0x0013);
        new_cpu.reg.set_de(0x00D8);
        new_cpu.reg.set_hl(0x014D);
        new_cpu.reg.sp = 0xFFFE;
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
        // TODO implement all these and the ime, see https://gbdev.io/pandocs/ for more info
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
            _ => panic!("{:?}", register),
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
            _ => panic!("{:?}: value: {}", register, value),

        }
    }
    pub fn get_register_debug_string(&self) -> String{
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
    fn get_target_hl(&mut self, target: RegisterTarget) -> u8{
        return if target == RegisterTarget::HL{
            self.get_target(RegisterTarget::MemoryAdress(self.get_target(RegisterTarget::HL))) as u8
        } else {
            self.get_target(target) as u8
        }
    }
    fn get_a_and_target_hl(&mut self, target : RegisterTarget) -> (u8, u8) { // okay this function is kinda pointless now but i used it for most of the alu instructions so whatever
        let reg_a_value = self.reg.a;
        let target_value = self.get_target_hl(target);
        return (reg_a_value, target_value as u8)
    }
    fn inc_target(&mut self, target : RegisterTarget, set_flags: bool) {
        let target_val = self.get_target(target);
        self.set_target(target, target_val.wrapping_add(1));
        if set_flags {
            self.reg.set_flag(Flag::Z, self.get_target(target) == 0);
            self.reg.set_flag(Flag::N, false);
            self.reg.set_flag(Flag::H, target_val & 0xf == 0xf);
        }
    }
    fn dec_target(&mut self, target : RegisterTarget, set_flags: bool) {
        let target_val = self.get_target(target);
        self.set_target(target, target_val.wrapping_sub(1));
        if set_flags {
            self.reg.set_flag(Flag::Z, self.get_target(target) == 0);
            self.reg.set_flag(Flag::N, true);
            self.reg.set_flag(Flag::H, target_val & 0xf == 0xf);
        }

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
        
        self.reg.a = self.reg.a.wrapping_add(target_value);
    }
    fn add_HL_x(&mut self, target: RegisterTarget) {
        let (reg_hl_value, mut target_value) = (self.reg.get_hl(), self.get_target(target));
        let result = reg_hl_value.wrapping_add(target_value);
        
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, (reg_hl_value ^ target_value ^ result) & 0x1000 != 0);
        self.reg.set_flag(Flag::C, reg_hl_value.checked_add(target_value) == None);

        self.reg.set_hl(result);
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

        self.reg.a = self.reg.a.wrapping_sub(target_value);
    }
    // AND x register with A
    fn and_A_x(&mut self, target: RegisterTarget){
        let (reg_a_value, target_value) = self.get_a_and_target_hl(target);

        
        self.reg.a = reg_a_value & target_value;
        self.reg.set_flag(Flag::Z, self.reg.a == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::C, false);
        self.reg.set_flag(Flag::H, true);

    }
    // XOR x register with A
    fn xor_A_x(&mut self, target: RegisterTarget){
        let (reg_a_value, target_value) = self.get_a_and_target_hl(target);

        self.reg.a = reg_a_value ^ target_value;
        self.reg.set_flag(Flag::Z, self.reg.a == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::C, false);
        self.reg.set_flag(Flag::H, false);
    }
    // OR x register with A
    fn or_A_x(&mut self, target: RegisterTarget){
        let (reg_a_value, target_value) = self.get_a_and_target_hl(target);

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

    fn rcla(&mut self) { // rotate A left
        self.reg.set_flag(Flag::Z, false); // this may be wrong
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::C, (self.reg.a >> 7) == 1);
        self.reg.set_flag(Flag::H, false);

        self.reg.a = self.reg.a.rotate_left(1);
    }

    fn rla(&mut self) { // rotate A left through carry
        let newcarry = (self.reg.a >> 7) == 1;
        let oldcarry = self.reg.get_flag(Flag::C) as u8;

        self.reg.a = (self.reg.a << 1) | oldcarry;

        self.reg.set_flag(Flag::Z, false); // this maybe wrong
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::C, newcarry);
        self.reg.set_flag(Flag::H, false);
    }

    fn rrca(&mut self) { // rotate A right, old bit [0] to carry
        self.reg.set_flag(Flag::Z, false); // this may be wrong
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::C, (self.reg.a & 1) == 1);

        self.reg.a = self.reg.a.rotate_right(1);
    }

    fn rra(&mut self) {
        let oldcarry = self.reg.get_flag(Flag::C) as u8;

        self.reg.set_flag(Flag::C, (self.reg.a & 1) == 1);
        self.reg.a = (self.reg.a >> 1) | oldcarry << 7;

        self.reg.set_flag(Flag::Z, false); // this may be wrong
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, false);
    }

    /*
    This is a fun instruction. It converts the A register in BCD
    representation. BCD is funky and is mostly used in some
    alphanumeric displays. Its fucking insane i have no idea why you'd
    wanna use this, just use normal binary. I guess it kinda makes
    sense for human readable binary BUT WHEN THE HELL DOES A PERSON
    PLAYING A GAMEBOY NEED TO READ BINARY. JESUS NINTY THIS IS A
    CUSTOM CPU, WHY IS THIS INSTRUCTION HERE????  See:
    https://ehaskins.com/2018-01-30%20Z80%20DAA/ for more info 

    ----

    Some non gba related thoughts: x86 cpus support operations on BCD
    data. This is probably because the BIOS in many early computers
    store the time in BCD, sure it would be easier to just leave the
    conversion up to program code but hey intel tends to only implment
    important instructions so they must have a decent reason /s

    According to this page:
    https://handwiki.org/wiki/Intel_BCD_opcode#Number_representation
    BCD is used to store decimal numbers in financal software.

    BCD, despite angiring me, is also one of the most important things
    to happen to computer science and
    law. https://en.wikipedia.org/wiki/Gottschalk_v._Benson
    */
    fn daa(&mut self){
        let a_value = self.reg.a;
        let mut adjust = 0;

        if self.reg.get_flag(Flag::H) {
            adjust |= 0x06;
        }

        if self.reg.get_flag(Flag::C) {
            adjust |= 0x60;
        }

        let res =
            if self.reg.get_flag(Flag::N) {
                a_value.wrapping_sub(adjust)
            } else {
                if a_value & 0x0f > 0x09 {
                    adjust |= 0x06
                }

                if a_value > 0x99 {
                    adjust |= 0x60
                }

                a_value.wrapping_add(adjust)
            };

        self.reg.a = res;

        self.reg.set_flag(Flag::Z, res == 0);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::C, adjust & 0x60 != 0);
    }

    fn relative_jump(&mut self, condition : bool) {
        let to_add = self.pcc() as i8; // gotta increment the program counter even if we dont do anything
        if condition {
            let temp_pc = self.reg.pc as i16;
            self.reg.pc = temp_pc.wrapping_add(to_add as i16) as u16;
        }
    }

    fn pop_byte(&mut self) -> u8{
        self.reg.sp += 1;
        self.mem.memory[(self.reg.sp-1) as usize]
    }

    fn pop_word(&mut self) -> u16 {
        let lo = self.pop_byte();
        let hi = self.pop_byte();

        (hi as u16) << 8 | lo as u16
    }

    fn push_byte(&mut self, val:u8) {
        self.reg.sp -= 1;
        self.mem.memory[self.reg.sp as usize] = val
    }

    fn push_word(&mut self, val:u16){
        self.push_byte((val >> 8) as u8);
        self.push_byte(val as u8);
    }

    fn rst(&mut self, addr:u16){
        self.push_word(self.reg.pc);
        self.reg.pc = addr;
    }
        
    

    pub fn tick(&mut self){
        fn nibc(unib : u8, lnib : u8) -> u16 { //combines 2 nibbles, upper nibble and lower nibble - USING THE WRONG TERMINOLOGY HERE, ITS NOT NIBBLES ITS 2 BYTES COMBINED INTO A WORD CAUSE WE DOING A SPECIAL LOAD etc
            (lnib as u16) << 8 | unib as u16
        }
        self.opcode = self.pcc();

        match self.opcode { // * massive switch that implements all the cpu functions
            // ! perhaps move each instruction into its own function, will shorten code, but at cost of increased complexity
            0x0 => {}, // No Operation (NOP)
            0x01 => { // LD BC, u16
                self.reg.c = self.pcc();
                self.reg.b = self.pcc();
            }
            0x02 => self.set_target(self.pointer_convert(RegisterTarget::BC), self.reg.a.into()), // LD (BC), A
            0x03 => self.inc_target(RegisterTarget::BC, false),
            0x04 => self.inc_target(RegisterTarget::B, true),
            0x05 => self.dec_target(RegisterTarget::B, true),
            0x06 => {
                let value = RegisterTarget::Value(self.pcc().into());
                self.ld_X_x(RegisterTarget::B, value);
            }
            0x07 => self.rcla(),
            0x08 => { // LD (u16), SP. Store SP & $FF at address u16 and SP >> 8 at address u16 + 1. 
                let addr = nibc(self.pcc(), self.pcc());
                let val1 = (self.reg.sp & 0xff) as u8;
                let val2 = (self.reg.sp >> 8) as u8;
                self.mem.memory[addr as usize] = val1;
                self.mem.memory[(addr + 1) as usize] = val2;
            }
            0x09 => self.add_HL_x(RegisterTarget::BC),
            0x0A => self.ld_X_x(RegisterTarget::A, self.pointer_convert(RegisterTarget::BC)),
            0x0B => self.dec_target(RegisterTarget::BC, false),
            0x0C => self.inc_target(RegisterTarget::C, true),
            0x0D => self.dec_target(RegisterTarget::C, true),
            0x0E => self.reg.c = self.pcc(),
            0x0F => self.rrca(),
            0x10 => (exit(1)), // STOP
            0x11 => { // LD DE, u16
                self.reg.e = self.pcc();
                self.reg.d = self.pcc();
            }
            0x12 => self.set_target(self.pointer_convert(RegisterTarget::DE), self.reg.a.into()),  // LD (DE), A
            0x13 => self.inc_target(RegisterTarget::DE, false),
            0x14 => self.inc_target(RegisterTarget::D, true),
            0x15 => self.dec_target(RegisterTarget::D, true),
            0x16 => {
                let value = RegisterTarget::Value(self.pcc().into());
                self.ld_X_x(RegisterTarget::D, value);
            }
            0x17 => self.rla(),
            0x18 => { // Unconditional jump to relative adress. this might be wrong
                self.relative_jump(true);
            }
            0x19 => self.add_HL_x(RegisterTarget::DE),
            0x1A => self.ld_X_x(RegisterTarget::A, self.pointer_convert(RegisterTarget::DE)),
            0x1B => self.dec_target(RegisterTarget::DE, false),
            0x1C => self.inc_target(RegisterTarget::E, true),
            0x1D => self.dec_target(RegisterTarget::E, true),
            0x1E => self.reg.e = self.pcc(),
            0x1F => self.rra(),
            0x20 => self.relative_jump(! self.reg.get_flag(Flag::Z)),
            0x21 =>{ // LD HL, u16
                self.reg.l = self.pcc();
                self.reg.h = self.pcc();
            },
            0x22 => {  // LD (HL+), A
                self.set_target(self.pointer_convert(RegisterTarget::HL), self.reg.a.into());
                self.inc_target(RegisterTarget::HL, false);
            },
            0x23 => self.inc_target(RegisterTarget::HL, false),
            0x24 => self.inc_target(RegisterTarget::H, true),
            0x25 => self.dec_target(RegisterTarget::H, true),
            0x26 => {
                let value = RegisterTarget::Value(self.pcc().into());
                self.ld_X_x(RegisterTarget::H, value);
            }
            0x27 => self.daa(),
            0x28 => self.relative_jump(self.reg.get_flag(Flag::Z)),
            0x29 => self.add_HL_x(RegisterTarget::HL),
            0x2A => {
                self.ld_X_x(RegisterTarget::A, self.pointer_convert(RegisterTarget::HL));
                self.inc_target(RegisterTarget::HL, false);
            }
            0x2B => self.dec_target(RegisterTarget::HL, false),
            0x2C => self.inc_target(RegisterTarget::L, true),
            0x2D => self.dec_target(RegisterTarget::L, true),
            0x2E => self.reg.l = self.pcc(),
            0x2F => { // compliment A
                self.reg.a = ! self.reg.a;
                self.reg.set_flag(Flag::N, true);
                self.reg.set_flag(Flag::H, true);
            },
            0x30 =>  self.relative_jump(! self.reg.get_flag(Flag::C)),
            0x31 => { //LD SP, u16
                let unib = self.pcc();
                let lnib = self.pcc();
                self.reg.sp = nibc(unib, lnib);

            },
            0x32 =>{ // LD (HL-), A
                self.set_target(self.pointer_convert(RegisterTarget::HL), self.reg.a.into());
                self.dec_target(RegisterTarget::HL, false);
            },
            0x33 => self.inc_target(RegisterTarget::SP, false),
            0x34 => self.inc_target(self.pointer_convert(RegisterTarget::HL), true),
            0x35 => self.dec_target(self.pointer_convert(RegisterTarget::HL), true),
            0x36 => {
                let value = RegisterTarget::Value(self.pcc().into());
                self.ld_X_x(RegisterTarget::HL, value);
            }
            0x37 => { // SCF: set carry flag
                self.reg.set_flag(Flag::N, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, true);
            }
            0x38 => self.relative_jump(self.reg.get_flag(Flag::C)),
            0x39 => self.add_HL_x(RegisterTarget::SP),
            0x3A => {
                self.ld_X_x(RegisterTarget::A, self.pointer_convert(RegisterTarget::HL));
                self.inc_target(RegisterTarget::HL, false);
            }
            0x3B => self.dec_target(RegisterTarget::SP, false),
            0x3C => self.inc_target(RegisterTarget::A, true),
            0x3D => self.dec_target(RegisterTarget::A, true),
            0x3E => self.reg.a = self.pcc(),
            0x3F => self.reg.set_flag(Flag::C, ! self.reg.get_flag(Flag::C)),
            0x76 => { todo!()} // power down the cpu until an interrupt occurs
            0x40..=0x7f => {
                let target : RegisterTarget = ((self.opcode - 0x40) / 8).into();
                let val : RegisterTarget = ((self.opcode % 8) & 0b0000_1111).into();
                self.ld_X_x(target, val);
            }
            0x80..=0x87 => { // could collapse these two branches into a single satement
                let register: RegisterTarget = (self.opcode & 0b0000_1111).into();
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
                let register = (self.opcode & 0b0000_1111).into();
                self.or_A_x(register);
            }
            0xB8..=0xBF => {
                let register = ((self.opcode -0x8) & 0b0000_1111).into();
                self.cp_A_x(register)
            }
            0xc0 => { // pop word from stack and jump if Z flag is 0
                if !self.reg.get_flag(Flag::Z) {
                    self.reg.pc = self.pop_word();
                }
            }
            0xc1 => {
                let val = self.pop_word();
                self.reg.set_bc(val);
            }
            0xc2 => {
                if !self.reg.get_flag(Flag::Z){
                    let unib = self.pcc();
                    let lnib = self.pcc();
                    self.reg.pc = nibc(unib,lnib);
                }
            }
            0xc3 => { // jump to nn, set pc to nn
                let unib = self.pcc();
                let lnib = self.pcc();
                self.reg.pc = nibc(unib, lnib);
            }
            0xc4 => {
                todo!();
            }
            0xc5 => { // push BC onto stacks
                self.push_word(self.reg.get_bc())
            }
            0xc6 => {
                let value = RegisterTarget::Value(self.pcc().into());
                self.add_A_x(value, true)
            }
            0xc7 => {
                self.rst(0x0);
            }
            0xc8 => {
                if !self.reg.get_flag(Flag::C) {
                    self.reg.pc = self.pop_word();
                }
            }
            0xca => {
                if self.reg.get_flag(Flag::Z){
                    let unib = self.pcc();
                    let lnib = self.pcc();
                    self.reg.pc = nibc(unib, lnib);
                }
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
            0xcf => {
                self.rst(0x08)
            }
            0xd0 => {
                if !self.reg.get_flag(Flag::C) {
                    self.reg.pc = self.pop_word();
                }
            }
            0xd1 => {
                let val = self.pop_word();
                self.reg.set_de(val);
            }
            0xd2 => {
                if !self.reg.get_flag(Flag::C){
                    let unib = self.pcc();
                    let lnib = self.pcc();
                    self.reg.pc = nibc(unib,lnib);
                }
            }
            0xd5 => {
                self.push_word(self.reg.get_de())
            }
            0xd6 => {
                let value = RegisterTarget::Value(self.pcc().into());
                self.sub_A_x(value, true)
            }
            0xd7 => {
                self.rst(0x10)
            }
            0xd8 => {
                if self.reg.get_flag(Flag::C) {
                    self.reg.pc = self.pop_word();
                }
            }
            0xda => {
                if self.reg.get_flag(Flag::C){
                    let unib = self.pcc();
                    let lnib = self.pcc();
                    self.reg.pc = nibc(unib,lnib);
                }
            }
            0xdf => {
                self.rst(0x18)
            }
            0xe0 => {
                let addr = 0xff00 + self.pcc() as u16;
                self.ld_X_x(RegisterTarget::MemoryAdress(addr), RegisterTarget::A)
            }
            0xe1 => {
                let val = self.pop_word();
                self.reg.set_hl(val);
            }
            0xe5 => {
                self.push_word(self.reg.get_hl())
            }
            0xe6 => {
                let value = RegisterTarget::Value(self.pcc().into());
                self.and_A_x(value)
            }
            0xe7 => {
                self.rst(0x20)
            }
            0xe8 => {
                let (reg_sp_value, mut target_value) = (self.reg.sp as i32, self.pcc() as i32);

                let result = reg_sp_value.wrapping_add(target_value);

                let carry_check = ((reg_sp_value ^ target_value ^ result) & 0x100 != 0);
                let half_carry = ((reg_sp_value ^ target_value ^ result) & 0x10 != 0);

                self.reg.set_flag(Flag::Z,false);
                self.reg.set_flag(Flag::N, false);
                self.reg.set_flag(Flag::H,half_carry);
                self.reg.set_flag(Flag::C,carry_check);

                self.reg.sp = result as u16;
            }
            0xef => {
                self.rst(0x28)
            }
            0xf0 => {
                let addr = 0xff00 + self.pcc() as u16;
                self.ld_X_x(RegisterTarget::A, RegisterTarget::MemoryAdress(addr))
            }
            0xf1 => {
                self.ld_X_x(RegisterTarget::A, self.pointer_convert(RegisterTarget::SP));
                self.inc_target(RegisterTarget::SP, false);
                self.reg.f = self.get_target_hl(self.pointer_convert(RegisterTarget::SP));
                self.inc_target(RegisterTarget::SP, false);
            }
            0xf3 => {
                self.interupts_enabled = false; // TODO implement interupts
            }
            0xf5 => {
                self.dec_target(RegisterTarget::SP, false);
                self.ld_X_x(self.pointer_convert(RegisterTarget::SP), RegisterTarget::A);
                self.dec_target(RegisterTarget::SP, false);
                self.ld_X_x(self.pointer_convert(RegisterTarget::SP), RegisterTarget::Value(self.reg.f.into()));
            }
            0xf6 => {
                let value = RegisterTarget::Value(self.pcc().into());
                self.or_A_x(value)
            }
            0xf7 => {
                self.rst(0x30)
            }
            0xf8 => {
                let (reg_sp_value, mut target_value) = (self.reg.sp as i32, self.pcc() as i32);

                let result = reg_sp_value.wrapping_add(target_value);

                let carry_check = (reg_sp_value ^ target_value ^ result) & 0x100 != 0;
                let half_carry = (reg_sp_value ^ target_value ^ result) & 0x10 != 0;

                self.reg.set_flag(Flag::Z,false);
                self.reg.set_flag(Flag::N, false);
                self.reg.set_flag(Flag::H,half_carry);
                self.reg.set_flag(Flag::C,carry_check);

                self.reg.set_hl(result as u16);
            }
            0xfb => {
                self.interupts_enabled = true;
            }
            0xfe => {
                let val = self.pcc().into();
                self.cp_A_x(RegisterTarget::Value(val))
            }
            0xff => {
                self.rst(0x38)
            }
            _ => {
                println!("Unimplemented instruction {:x}", self.opcode);
                println!("Register Dump as hex: ");
                println!("a: {:x}, f: {:x} \nb: {:x} c: {:x} \nd: {:x} e:{:x} \nh:{:x} l: {:x} \nsp: {:x} pc: {:x}", self.reg.a, self.reg.f, self.reg.b, self.reg.c, self.reg.d, self.reg.e, self.reg.h, self.reg.l, self.reg.sp, self.reg.pc);

                exit(0);
            }
        }
    }
}
