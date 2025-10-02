#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;
use alloc::vec::Vec;

static C_MAGIC: u32 = 0x7f_45_4c_46;

// #[cfg(arch = "x86_64")]
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
    pub eoh: u16,
}

// #[cfg(arch = "x86_64")]
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
    pub eoh: [u8; 24],
}

// #[cfg(arch = "x86_64")]
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
    eoh: [u8; 24],
}

impl ElfHeader {
    pub fn new(bytes: &Vec<u8>) -> Result<&ElfHeader, uefi::Status> {
        if bytes.len() < core::mem::size_of::<ElfHeader>() {
            return Err(uefi::Status::BAD_BUFFER_SIZE);
        }

        let header_ref: &ElfHeader = unsafe {
            &*(bytes.as_ptr() as *const ElfHeader)
        };

        Ok(header_ref)
    }

    pub fn check_magic(&self) -> bool {
        let mut magic: u32 = 0;

        magic |= (self.e_ident[0] as u32) << 24;
        magic |= (self.e_ident[1] as u32) << 16;
        magic |= (self.e_ident[2] as u32) << 8;
        magic |= (self.e_ident[3] as u32) << 0;

        return (magic == C_MAGIC);
    }
}