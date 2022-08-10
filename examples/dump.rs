use elf_riscv32::*;

fn main() {
    let mut data = [0u32; 8192];
    let elf = include_bytes!("test.elf");
    if elf.len() > data.len() * core::mem::size_of::<u32>() {
        panic!("array too small")
    }
    unsafe { (data.as_mut_ptr() as *mut u8).copy_from_nonoverlapping(elf.as_ptr(), elf.len()) };
    
    let elf = Elf::new(&data).unwrap();
    for section in elf.sections().unwrap() {
        let section = section.unwrap();
        println!("{} = {section:X?}", elf.section_name(&section).unwrap())
    }
    for program in elf.programs().unwrap() {
        let program = program.unwrap();
        println!("{program:X?}")
    }
}