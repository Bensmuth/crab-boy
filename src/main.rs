mod cpu;
mod memory;
use std::fs::File;
use std::io::Read;


fn main() {
    let registers = cpu::Registers::new(0,0,0,0,0,0,0,0,0,0);
    let operation = cpu::Operation::new(0, 0, 0, 0);
    let mut main_memory = memory::Memory::new();
    let mut main_cpu= cpu::Cpu::new(registers, operation);

    let mut file=File::open("resources/bios.gb").unwrap();
    let mut buf=[0u8;256];
    file.read(&mut buf).unwrap();
    for x in 0..255 {
        main_memory.memory[x] = buf[x];
    }


    loop {
        main_memory = main_cpu.tick(main_memory)
    }

}
