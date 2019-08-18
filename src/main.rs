mod cpu;
mod memory;
use std::fs::File;
use std::io::Read;


fn main() {
    let registers = cpu::Registers::new(0,0,0,0,0,0,0,0,0,0); // * sets starting registers and opcode
    let operation = cpu::Operation::new(0, 0, 0, 0);
    let mut main_memory = memory::Memory::new();
    let mut main_cpu= cpu::Cpu::new(registers, operation);

    let mut file=File::open("resources/bios.gb").unwrap(); // ! dirty rom load, please replace this when cartridge controller implemented
    let mut buf=[0u8;256];
    file.read(&mut buf).unwrap();
    for x in 0..255 { // ! dirty rom into memeory merge, bad method only supports bios at the moment
        main_memory.memory[x] = buf[x];
    }


    loop {
        main_memory = main_cpu.tick(main_memory)
    }

}
