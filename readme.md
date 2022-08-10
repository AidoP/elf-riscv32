# elf-riscv32
[![Get from crates.io][cratesio-image]][crates.io]
[![Documentation on docs.rs][docsrs-image]][docs.rs]

[cratesio-image]: https://img.shields.io/crates/v/elf\_riscv32.svg
[crates.io]: https://crates.io/crates/elf\_riscv32
[docsrs-image]: https://docs.rs/elf-riscv32/badge.svg
[docs.rs]: https://docs.rs/elf-riscv32

**A no-std ELF parser for elf32-littleriscv designed to be reasonably quick.**

```toml
elf_riscv32 = "0.0.1"
```

Requires the ELF file to be aligned in memory as it is parsed in-place.
Tries to be minimal as it is intended for use in OS loaders for rv32i.

```rust
let elf = Elf::new(&data).unwrap();
for section in elf.sections().unwrap() {
    let section = section.unwrap();
    println!("{} = {section:X?}", elf.section_name(&section).unwrap())
}
for program in elf.programs().unwrap() {
    let program = program.unwrap();
    println!("{program:X?}")
}
```
