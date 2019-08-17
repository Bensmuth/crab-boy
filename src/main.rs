mod cpu;
mod memory;

fn main() {
    let registers = cpu::Registers::new(245,0,0,0,0,0,0,0,0,0);
    let operation = cpu::Operation::new(0, 0, 0, 0);
    let mut main_memory = memory::Memory::new();
    let mut main_cpu= cpu::Cpu::new(registers, operation);
    println!("Main init");
    loop {
        main_memory = main_cpu.tick(main_memory)
    }

}
