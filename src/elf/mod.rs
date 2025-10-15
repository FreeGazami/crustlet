#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use log::info;

static C_MAGIC: u32 = 0x7f_45_4c_46;
static EI_MAG0: usize = 0;
static EI_MAG1: usize = 1;
static EI_MAG2: usize = 2;
static EI_MAG3: usize = 3;


enum E_P_TYPE {
    PT_NULL =    0x00000000,
    PT_LOAD =    0x00000001,
    PT_DYNAMIC = 0x00000002,
    PT_INTERP =  0x00000003,
    PT_NOTE =    0x00000004,
    PT_SHLIB =   0x00000005,
    PT_PHDR =    0x00000006,
    PT_TLS =     0x00000007,
    PT_LOOS =    0x60000000,
    PT_HIOS =    0x6FFFFFFF,
    PT_LOPROC =  0x70000000,
    PT_HIPROC =  0x7FFFFFFF,
}


#[cfg(target_arch="x86_64")]
static C_E_MACHINE: u16 = 0x3e;


#[cfg(target_arch="x86_64")]
#[repr(C)]
pub struct ElfHeader {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}


#[cfg(target_arch="x86_64")]
#[repr(C)]
pub struct ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr:u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}


#[cfg(target_arch="x86_64")]
#[repr(C)]
pub struct ProgramHeaderTable<'a> {
    pub entries: &'a [ProgramHeader],
}


#[cfg(target_arch="x86_64")]
#[repr(C)]
pub struct SectionHeader {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u64,
    sh_addr:  u64,
    sh_offset: u64,
    sh_size:  u64,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u64,
    sh_entsize: u64,
}


#[cfg(target_arch="x86_64")]
impl ElfHeader {
    pub fn new(bytes: &Vec<u8>) -> Result<&ElfHeader, uefi::Status> {
        if bytes.len() < core::mem::size_of::<ElfHeader>() {
            return Err(uefi::Status::BAD_BUFFER_SIZE);
        }

        let header_ref: &ElfHeader = unsafe {
            &*(bytes.as_ptr() as *const ElfHeader)
        };

        // Verify magic beforehand
        if !header_ref.check_magic() {
            info!(
                "Elf header doesn't contain correct magic number: 0x{:x}{}{}{}",
                header_ref.e_ident[EI_MAG0],
                header_ref.e_ident[EI_MAG1] as char,
                header_ref.e_ident[EI_MAG2] as char,
                header_ref.e_ident[EI_MAG3] as char,
            );

            return Err(uefi::Status::INVALID_PARAMETER);
        }

        if header_ref.e_machine != C_E_MACHINE {
            info!("Elf header has incorrect e_machine: 0x{:x}", header_ref.e_machine);
            return Err(uefi::Status::UNSUPPORTED);
        }

        return Ok(header_ref);
    }

    fn check_magic(&self) -> bool {
        let mut magic: u32 = 0;

        magic |= (self.e_ident[EI_MAG0] as u32) << 24;
        magic |= (self.e_ident[EI_MAG1] as u32) << 16;
        magic |= (self.e_ident[EI_MAG2] as u32) << 8;
        magic |= (self.e_ident[EI_MAG3] as u32) << 0;

        return magic == C_MAGIC;
    }

    pub fn new_ph_table(&self, file: &Vec<u8>) -> Result<ProgramHeaderTable<'_>, uefi::Status> {
        let e_phoff: u64 = self.e_phoff;
        let e_phnum: u16 = self.e_phnum;
        let e_phentsize: u16 = self.e_phentsize;
        let table_size: u64 = (e_phnum * e_phentsize).into();

        let program_header_array = unsafe {
            core::slice::from_raw_parts(
                file.as_ptr().wrapping_add(
                    e_phoff.try_into().unwrap(),
                ) as *const ProgramHeader,
                e_phnum.try_into().unwrap(),
            )
        };

        return Ok(
            ProgramHeaderTable {
                entries: program_header_array,
            }
        );
    }

    pub fn entry_start(&self, handoff_addr: u64) -> () {
        let e_entry_u: usize = self.e_entry as usize;
        let entry_ptr = e_entry_u as *const ();

        let e_fn: extern "C" fn(u64) = unsafe { 
            core::mem::transmute(entry_ptr)
        };

        unsafe {
           e_fn(handoff_addr)
        };
    }

    pub fn dump_info(&self) -> () {
        info!("elf magic: {:x}{}{}{}", self.e_ident[0], self.e_ident[1] as char, self.e_ident[2] as char, self.e_ident[3] as char);
        info!("elf entry address: 0x{:x}", self.e_entry);
        info!("endian: 0x{:x}", self.e_ident[5] as u8);
        info!("program header offset: {} bytes", self.e_phoff);
        info!("section header offset: {} bytes", self.e_shoff);
        info!("number of program headers: {}", self.e_phnum);
        info!("number of section headers: {}", self.e_shnum);
        info!("section header string table index: {}", self.e_shstrndx);
        info!("target machine: 0x{:x}", self.e_machine);
        info!("size of elf header: {} bytes", core::mem::size_of::<ElfHeader>());
    }
}


#[cfg(target_arch="x86_64")]
impl ProgramHeader {
    pub fn flags_executable(&self) -> bool {
        return (self.p_flags & 0b1) != 0;
    }

    pub fn flags_writable(&self) -> bool {
        return ((self.p_flags & 0b10) >> 1) != 0;
    }

    pub fn flags_readable(&self) -> bool {
        return ((self.p_flags & 0b100) >> 2) != 0;
    }

    pub fn dump_info(&self) -> () {
        info!("p_type: 0x{:x}", self.p_type);
        info!("p_flags: 0b{:b}", self.p_flags);
        info!("p_offset: 0x{:x}", self.p_offset);
        info!("p_vaddr: 0x{:x}", self.p_vaddr);
        info!("p_paddr: 0x{:x}", self.p_paddr);
        info!("p_filesz: 0x{:x}", self.p_filesz);
        info!("p_memsz: 0x{:x}", self.p_memsz);
        info!("p_align: 0x{:x}", self.p_align);
    }
}

// Try using core::intrinsics::volatile_copy_memory
// TODO: add error cases...
// TODO: add 0 stuffing / null out to the segment using filesz and memsz
#[cfg(target_arch="x86_64")]
impl ProgramHeaderTable<'_> {
    pub fn load_segments(&self, file: &Vec<u8>) -> uefi::Status {
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.p_type == E_P_TYPE::PT_LOAD as u32 {
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        file.as_ptr().wrapping_add(entry.p_offset as usize),
                        entry.p_paddr as *mut u8,
                        entry.p_filesz as usize,
                    );
                }
            }
        }

        return uefi::Status::SUCCESS;
    }
}