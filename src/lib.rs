#![no_std]

use core::{mem::{size_of, align_of}, fmt};

macro_rules! c_enum {
    (
        $vis:vis $name:ident($ty:ty) {
            $($item:ident = $value:expr),*
        } $catch:pat => $return:expr
    ) => {
        #[derive(::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq)]
        #[repr(transparent)]
        $vis struct $name($ty);
        impl $name {
            $(
                #[allow(non_upper_case_globals)]
                $vis const $item: Self = Self($value);
            )*
            $vis fn validate(self) -> Result<()> {
                match self.0 {
                    $($value => Ok(()),)*
                    $catch => $return
                }
            }
        }
        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #[derive(Debug)]
                struct Unknown($ty);
                match *self {
                    $(Self::$item => f.write_str(stringify!($item)),)*
                    Self(value) => Unknown(value).fmt(f)
                }
            }
        }
        impl ::core::convert::TryFrom<$ty> for $name {
            type Error = $crate::Error;
            fn try_from(value: $ty) -> $crate::Result<Self> {
                match value {
                    $($value => Ok(Self::$item),)*
                    $catch => $return
                }
            }
        }
        impl ::core::convert::From<$name> for $ty {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    }
}
macro_rules! c_flags {
    (
        $vis:vis $name:ident($ty:ty) {
            $($item:ident = $value:expr),*
        } $catch:pat => $return:expr
    ) => {
        #[derive(::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq)]
        #[repr(transparent)]
        $vis struct $name($ty);
        impl $name {
            #[allow(non_upper_case_globals)]
            $vis const None: Self = Self(0);
            $(
                #[allow(non_upper_case_globals)]
                $vis const $item: Self = Self($value);
            )*
            #[allow(non_upper_case_globals)]
            $vis const Mask: Self = Self($($value|)* 0);
            /// Returns true if any of the bits are set
            pub fn any(self, bits: Self) -> bool {
                self & bits != Self::None
            }
            /// Returns true if all of the bits are set
            pub fn all(self, bits: Self) -> bool {
                self & bits == bits
            }
        }
        impl ::core::ops::BitAnd for $name {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }
        impl ::core::ops::BitAndAssign for $name {
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0
            }
        }
        impl ::core::ops::BitOr for $name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }
        impl ::core::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0
            }
        }
        impl ::core::ops::BitXor for $name {
            type Output = Self;
            fn bitxor(self, rhs: Self) -> Self::Output {
                Self(self.0 ^ rhs.0)
            }
        }
        impl ::core::ops::BitXorAssign for $name {
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 ^= rhs.0
            }
        }
        impl ::core::ops::Not for $name {
            type Output = Self;
            fn not(self) -> Self::Output {
                Self(!self.0)
            }
        }
        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #[derive(Debug)]
                struct Unknown($ty);
                $(
                    #[derive(Debug)]
                    struct $item;
                )*
                let mut list = f.debug_tuple(stringify!($name));
                $(if self.all(Self($value)) { list.field(&$item); })*
                let invalid = *self & !Self::Mask;
                if invalid != Self::None {
                    list.field(&Unknown(self.0));
                }
                list.finish()
            }
        }
        impl ::core::convert::TryFrom<$ty> for $name {
            type Error = $crate::Error;
            fn try_from(value: $ty) -> $crate::Result<Self> {
                match Self(value) & !Self::Mask {
                    Self::None => Ok(Self(value)),
                    $catch => $return
                }
            }
        }
        impl ::core::convert::From<$name> for $ty {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    }
}
pub type Result<T> = core::result::Result<T, Error>;
#[derive(Debug)]
pub enum Error {
    IntegerOverflow,
    IndexOutOfRange,
    Unaligned,
    UnexpectedEoF,
    InvalidMagic,
    InvalidFormat,
    InvalidVersion,
    UnsupportedFileType(FileType),
    UnsupportedMachine(Machine),
    UnsupportedProgramType(ProgramType),
    UnsupportedProgramFlags(ProgramFlags),
    UnsupportedSectionType(SectionType),
    UnsupportedSectionFlags(SectionFlags),
    WrongProgramType { expected: ProgramType, actual: ProgramType },
    WrongProgramFlags { expected: ProgramFlags, actual: ProgramFlags },
    WrongSectionType { expected: SectionType, actual: SectionType },
    WrongSectionFlags { expected: SectionFlags, actual: SectionFlags },
    UnterminatedString,
    NotUtf8(core::str::Utf8Error)
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Address(u32);
impl Address {
    pub fn as_usize(self) -> Result<usize> {
        self.0.try_into().map_err(|_| Error::IntegerOverflow)
    }
}
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Offset(u32);
impl Offset {
    pub fn as_usize(self) -> Result<usize> {
        self.0.try_into().map_err(|_| Error::IntegerOverflow)
    }
}

/// A 32bit little-endian RISC-V ELF file.
/// 
/// ```
/// use elf_riscv32::*;
/// # (|| -> Result<()> {
/// # let mut data = [0u32; 8192];
/// # let elf = include_bytes!("../examples/test.elf");
/// # if elf.len() > data.len() * core::mem::size_of::<u32>() {
/// #     panic!("array too small")
/// # }
/// # unsafe { (data.as_mut_ptr() as *mut u8).copy_from_nonoverlapping(elf.as_ptr(), elf.len()) };
/// let elf = Elf::new(&data).unwrap();
/// for section in elf.sections().unwrap() {
///     let section = section.unwrap();
///     println!("{} = {section:X?}", elf.section_name(&section).unwrap())
/// }
/// for program in elf.programs().unwrap() {
///     let program = program.unwrap();
///     println!("{program:X?}")
/// }
/// # Ok(()) })().unwrap()
/// ```
pub struct Elf<'a> {
    data: &'a [u8],
    pub header: &'a Header,
    pub section_names: StringTable<'a>
}
impl<'a> Elf<'a> {
    pub fn new(elf: &'a [u32]) -> Result<Self> {
        assert_eq!(align_of::<u32>(), align_of::<Header>());
        let data = unsafe { core::slice::from_raw_parts(elf.as_ptr() as *const u8, elf.len() * size_of::<u32>()) };
        let header = unsafe { Header::new_assume_aligned(data)? };
        let section_name_table = header.section_header(data, header.section_name_table)?;
        let section_names = StringTable(section_name_table.data(data)?);
        Ok(Self {
            data,
            header,
            section_names
        })
    }
    pub fn program(&self, index: u16) -> Result<Program<'a>> {
        let header = self.header.program_header(self.data, index)?;
        header.data(self.data).map(|data| Program::new(header, data))
    }
    /// Get an iterator over programs.
    pub fn programs(&'a self) -> Result<TableIter<Program<'a>>> {
        TableIter::new(self.data, self.header.ph_offset, self.header.ph_count, self.header.ph_entry_size)
    }
    /// Get the section name string given an offset into the section header string table.
    pub fn section_name(&'a self, section: &Section<'a>) -> Result<&'a str> {
        self.section_names.get_str(section.header.name)
    }
    pub fn section(&self, index: u16) -> Result<Section<'a>> {
        let header = self.header.section_header(self.data, index)?;
        header.data(self.data).map(|data| Section::new(header, data))
    }
    /// Get an iterator over sections.
    pub fn sections(&'a self) -> Result<TableIter<Section<'a>>> {
        TableIter::new(self.data, self.header.sh_offset, self.header.sh_count, self.header.sh_entry_size)
    }
}
impl<'a> fmt::Debug for Elf<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Elf")
            .field("data", &[..])
            .field("header", &self.header)
            .field("section_names", &self.section_names)
            .finish()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Header {
    pub ident: [u8; 16],
    pub ty: FileType,
    pub machine: Machine,
    pub version: u32,
    pub entry: Address,
    pub ph_offset: Offset,
    pub sh_offset: Offset,
    pub flags: u32,
    pub header_size: u16,
    pub ph_entry_size: u16,
    pub ph_count: u16,
    pub sh_entry_size: u16,
    pub sh_count: u16,
    pub section_name_table: u16
}
impl Header {
    pub fn new(elf: &[u32]) -> Result<&Self> {
        assert_eq!(align_of::<u32>(), align_of::<Self>());
        unsafe {
            let len = elf.len() * size_of::<u32>();
            Self::new_assume_aligned(core::slice::from_raw_parts(elf.as_ptr() as *const u8, len))
        }
    }
    /// Like `Header::new` but takes a byte slice and checks the alignment.
    pub fn new_aligned(elf: &[u8]) -> Result<&Self> {
        assert_eq!(align_of::<u32>(), align_of::<Self>());
        if elf.as_ptr() as usize & 0b11 != 0 {
            Err(Error::Unaligned)
        } else {
            unsafe { Self::new_assume_aligned(elf) }
        }
    }
    /// Coerce a byte slice into an ELF header.
    /// 
    /// # Safety
    /// It is undefined behaviour for `elf` to have a smaller alignment than `Header`.
    pub unsafe fn new_assume_aligned(elf: &[u8]) -> Result<&Self> {
        if elf.len() < size_of::<Self>() {
            return Err(Error::UnexpectedEoF)
        }
        let elf = &*(elf.as_ptr() as *const Header);
        if &elf.ident[..4] != b"\x7fELF" {
            return Err(Error::InvalidMagic)
        }
        if elf.ident[4] != 1 || elf.ident[5] != 1 {
            // Not riscv32-le
            return Err(Error::InvalidFormat)
        }
        if elf.ident[6] != 1 || elf.version != 1 {
            return Err(Error::InvalidVersion)
        }
        elf.machine.validate()?;
        Ok(elf)
    }
    pub fn program_header<'a>(&'a self, elf: &'a [u8], index: u16) -> Result<&'a ProgramHeader> {
        if index >= self.ph_count {
            Err(Error::IndexOutOfRange)
        } else {
            let offset: usize = self.ph_offset.as_usize()? + (self.ph_entry_size as usize * index as usize);
            ProgramHeader::new(&elf[offset..])
        }
    }
    pub fn section_header<'a>(&'a self, elf: &'a [u8], index: u16) -> Result<&'a SectionHeader> {
        if index >= self.sh_count {
            Err(Error::IndexOutOfRange)
        } else {
            let offset: usize = self.sh_offset.as_usize()? + (self.sh_entry_size as usize * index as usize);
            SectionHeader::new(&elf[offset..])
        }
    }
}

pub struct TableIter<'a, T: 'a + TableEntry<'a>> {
    elf: &'a [u8],
    offset: Offset,
    count: u16,
    size: u16,
    index: u16,
    _marker: core::marker::PhantomData<T>
}
impl<'a, T: 'a + TableEntry<'a>> TableIter<'a, T> {
    pub fn new(elf: &'a [u8], offset: Offset, count: u16, size: u16) -> Result<Self> {
        assert_eq!(align_of::<u32>(), align_of::<T::Header>());
        if elf.as_ptr() as usize & 0b11 != 0 || size & 0b11 != 0 {
            Err(Error::Unaligned)
        } else {
            Ok(Self {
                elf,
                offset,
                count,
                size,
                index: 0,
                _marker: core::marker::PhantomData,
            })
        }
    }
}
impl<'a, T: 'a + TableEntry<'a>> Iterator for TableIter<'a, T> {
    type Item = Result<T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.count {
            None
        } else {
            let offset = match self.offset.as_usize() {
                Ok(offset) => offset,
                Err(e) => return Some(Err(e))
            };
            let offset = offset + self.index as usize * self.size as usize;
            self.index += 1;
            self.elf.get(offset..).map(|header| T::new(self.elf, header))
        }
    }
}

pub trait TableEntry<'a> where Self: Sized {
    type Header;
    fn new(elf: &'a [u8], header: &'a [u8]) -> Result<Self>;
}


/// A program header and its associated data.
pub struct Program<'a> {
    pub header: &'a ProgramHeader,
    pub data: &'a [u8]
}
impl<'a> Program<'a> {
    pub fn new(header: &'a ProgramHeader, data: &'a [u8]) -> Self {
        Self {
            header,
            data
        }
    }
    /// Ensure that the type of the program header matches that expected, returning an `Err` otherwise.
    pub fn check_type(&self, ty: ProgramType) -> Result<()> {
        if self.header.ty != ty {
            Err(Error::WrongProgramType { expected: ty, actual: self.header.ty })
        } else {
            Ok(())
        }
    }
    pub fn check_flag(&self, flags: ProgramFlags) -> Result<()> {
        if self.header.flags.all(flags) {
            Ok(())
        } else {
            Err(Error::WrongProgramFlags { expected: flags, actual: self.header.flags })
        }
    }
}
impl<'a> fmt::Debug for Program<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.header.fmt(f)
    }
}
impl<'a> TableEntry<'a> for Program<'a> {
    type Header = ProgramHeader;
    fn new(elf: &'a [u8], header: &'a [u8]) -> Result<Self> {
        let header = ProgramHeader::new(header)?;
        Ok(Self::new(header, header.data(elf)?))
    }
}
#[derive(Debug)]
#[repr(C)]
pub struct ProgramHeader {
    pub ty: ProgramType,
    pub offset: Offset,
    pub virt_addr: Address,
    pub phys_addr: Address,
    pub file_size: u32,
    pub mem_size: u32,
    pub flags: ProgramFlags,
    pub align: u32
}
impl ProgramHeader {
    pub fn new(header: &[u8]) -> Result<&Self> {
        if header.len() < size_of::<Self>() {
            return Err(Error::UnexpectedEoF)
        }
        if header.as_ptr() as usize & 0b11 != 0 {
            return Err(Error::Unaligned)
        }
        let header = unsafe { &*(header.as_ptr() as *const ProgramHeader) };
        Ok(header)
    }
    pub fn data<'a>(&'a self, elf: &'a [u8]) -> Result<&'a [u8]> {
        let size: usize = self.file_size.try_into().map_err(|_| Error::IntegerOverflow)?;
        let offset = self.offset.as_usize()?;
        elf.get(offset..offset + size).ok_or(Error::UnexpectedEoF)
    }
}

/// A section header and its associated data.
pub struct Section<'a> {
    pub header: &'a SectionHeader,
    pub data: &'a [u8]
}
impl<'a> Section<'a> {
    pub fn new(header: &'a SectionHeader, data: &'a [u8]) -> Self {
        Self {
            header,
            data
        }
    }
    /// Ensure that the type of the section matches that expected, returning an `Err` otherwise.
    pub fn check_type(&self, ty: SectionType) -> Result<()> {
        if self.header.ty != ty {
            Err(Error::WrongSectionType { expected: ty, actual: self.header.ty })
        } else {
            Ok(())
        }
    }
    pub fn check_flag(&self, flags: SectionFlags) -> Result<()> {
        if self.header.flags.all(flags) {
            Ok(())
        } else {
            Err(Error::WrongSectionFlags { expected: flags, actual: self.header.flags })
        }
    }
}
impl<'a> fmt::Debug for Section<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.header.fmt(f)
    }
}
impl<'a> TableEntry<'a> for Section<'a> {
    type Header = SectionHeader;
    fn new(elf: &'a [u8], header: &'a [u8]) -> Result<Self> {
        let header = SectionHeader::new(header)?;
        Ok(Self::new(header, header.data(elf)?))
    }
}
#[derive(Debug)]
#[repr(C)]
pub struct SectionHeader {
    pub name: u32,
    pub ty: SectionType,
    pub flags: SectionFlags,
    pub address: Address,
    pub offset: Offset,
    pub size: u32,
    pub link: u32,
    pub info: u32,
    pub alignment: u32,
    pub entry_size: u32
}
impl SectionHeader {
    pub fn new(header: &[u8]) -> Result<&Self> {
        if header.len() < size_of::<Self>() {
            return Err(Error::UnexpectedEoF)
        }
        if header.as_ptr() as usize & 0b11 != 0 {
            return Err(Error::Unaligned)
        }
        let header = unsafe { &*(header.as_ptr() as *const SectionHeader) };
        Ok(header)
    }
    pub fn data<'a>(&'a self, elf: &'a [u8]) -> Result<&'a [u8]> {
        let size: usize = self.size.try_into().map_err(|_| Error::IntegerOverflow)?;
        let offset = self.offset.as_usize()?;
        elf.get(offset..offset + size).ok_or(Error::UnexpectedEoF)
    }
}

#[derive(Copy, Clone)]
pub struct StringTable<'a>(&'a [u8]);
impl<'a> StringTable<'a> {
    /// Coerce a Section into a string table.
    /// 
    /// The section must be of type `SHT_STRTAB` and have the `SHF_STRINGS` flag.
    pub fn new(section: Section<'a>) -> Result<Self> {
        section.check_type(SectionType::StringTable)?;
        section.check_flag(SectionFlags::Strings)?;
        Ok(Self(section.data))
    }
    pub fn get_str(self, index: u32) -> Result<&'a str> {
        self.get_bytes(index)
            .and_then(|b|
                core::str::from_utf8(b)
                    .map_err(Error::NotUtf8)
            )
    }
    /// Get the byte slice of a string in the string table, not including the null terminator.
    pub fn get_bytes(self, index: u32) -> Result<&'a [u8]> {
        self.0.get(index as usize..).ok_or(Error::IndexOutOfRange)
            .and_then(|s| {
                let end = memchr::memchr(0, s).ok_or(Error::UnterminatedString)?;
                Ok(&s[0..end])
            })
    }
    /// Returns a pointer to the start of the string table.
    /// 
    /// The string table is not confirmed to be null terminated.
    pub fn get_ptr(self) -> *const u8 {
        self.0.as_ptr()
    }
    /// Returns the length of the string table in bytes.
    pub fn len(self) -> usize {
        self.0.len()
    }
    /// Returns true if the string table has a length of 0.
    pub fn is_empty(self) -> bool {
        self.0.is_empty()
    }
}
impl<'a> fmt::Debug for StringTable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("StringTable")
            .field(&[..])
            .finish()
    }
}

c_enum!{
    pub FileType(u16) {
        None = 0,
        Relocatable = 1,
        Executable = 2,
        SharedObject = 3,
        Core = 4
    } v => Err(Error::UnsupportedFileType(Self(v)))
}
c_enum!{
    pub Machine(u16) {
        RiscV = 243
    } v => Err(Error::UnsupportedMachine(Self(v)))
}
c_enum!{
    pub ProgramType(u32) {
        Null = 0,
        Load = 1,
        Dynamic = 2,
        Interpreter = 3,
        Note = 4,
        ProgramHeader = 6,
        ThreadLocalStorage = 7,
        GnuStack = 0x6474E551,
        RiscVAttributes = 0x70000003
    } v => Err(Error::UnsupportedProgramType(Self(v)))
}
c_flags!{
    pub ProgramFlags(u32) {
        Read = 0b001,
        Exec = 0b010,
        Write = 0b100
    } v => Err(Error::UnsupportedProgramFlags(v))
}
c_enum!{
    pub SectionType(u32) {
        Null = 0,
        Program = 1,
        SymbolTable = 2,
        StringTable = 3,
        Rela = 4,
        HashTable = 5,
        Dynamic = 6,
        Note = 7,
        NoBits = 8,
        Rel = 9,
        DynamicSymbolTable = 11,
        InitArray = 14,
        FiniArray = 15,
        PreinitArray = 16,
        Group = 17,
        SymbolIndex = 18
    } v => Err(Error::UnsupportedSectionType(Self(v)))
}
c_flags!{
    pub SectionFlags(u32) {
        Write = 0x1,
        Alloc = 0x2,
        Exec = 0x4,
        Merge = 0x10,
        Strings = 0x20,
        InfoLink = 0x40,
        LinkOrder = 0x80,
        OsNonconforming = 0x100,
        Group = 0x200,
        Tls = 0x400,
        Compressed = 0x800
    } v => Err(Error::UnsupportedSectionFlags(v))
}