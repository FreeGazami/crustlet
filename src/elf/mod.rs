#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::vec::Vec;
use log::info;

static C_MAGIC: u32 = 0x7f_45_4c_46;


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
            return Err(uefi::Status::INVALID_PARAMETER);
        }

        return Ok(header_ref);
    }

    fn check_magic(&self) -> bool {
        let mut magic: u32 = 0;

        magic |= (self.e_ident[0] as u32) << 24;
        magic |= (self.e_ident[1] as u32) << 16;
        magic |= (self.e_ident[2] as u32) << 8;
        magic |= (self.e_ident[3] as u32) << 0;

        return magic == C_MAGIC;
    }

    pub fn new_ph_table(&self, file: &Vec<u8>) -> Result<&[ProgramHeader], uefi::Status> {
        let e_phoff: u64 = self.e_phoff;
        let e_phnum: u16 = self.e_phnum;
        let e_phentsize: u16 = self.e_phentsize;
        let table_size: u64 = (e_phnum * e_phentsize).into();

        let program_header_table = unsafe {
            core::slice::from_raw_parts(
                file.as_ptr().wrapping_add(
                    e_phoff.try_into().unwrap(),
                ) as *const ProgramHeader,
                e_phnum.try_into().unwrap(),
            )
        };

        return Ok(program_header_table);
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
    pub fn dump_info(&self) -> () {
        info!("p_type: 0x{:x}", self.p_type);
    }
}