mod cpu;

fn main() {
    let registers = cpu::Registers::new(245,0,0,0,0,0,0,0,0,0);
    let operation = cpu::Operation::new(0, 0, 0, 0);
    let mut mainCpu= cpu::Cpu::new(registers, operation);
    mainCpu.tick();
    println!("Main init");
    while true {
        mainCpu.tick()
    }

}
