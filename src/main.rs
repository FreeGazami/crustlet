//! File: main.rs
//! Author: Tomo Kaneko
//! Initial Date: 2025/10/05

#![no_std]
#![no_main]

extern crate alloc;

mod elf;

use log::info;
use uefi::boot::{self, SearchType};
use uefi::prelude::*;
use uefi::proto::device_path::text::{AllowShortcuts, DevicePathToText, DisplayOnly};
use uefi::proto::loaded_image::LoadedImage;
use uefi::{Identify, Result};
use uefi::CString16;
use uefi::fs::{FileSystem, FileSystemResult};
use elf::*;
use alloc::vec::Vec;
use uefi::mem::memory_map::{MemoryMapOwned, MemoryMapIter, MemoryMap, MemoryMapKey, MemoryMapMut};
use uefi::boot::{MemoryDescriptor, MemoryType};
use core::arch::asm;


#[cfg(target_arch="x86_64")]
#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().unwrap();

    /* TODO: add parser, load kernel path and rootfs from rEnv.txt dynamically */
    let path: CString16 = CString16::try_from("gazami").unwrap();
    let p_fs = boot::get_image_file_system(boot::image_handle()).unwrap();

    let mut fs = FileSystem::new(p_fs);

    let bytes: Vec<u8> = match fs.read(path.as_ref()) {
        Ok(vector) => vector,
        Err(error) => { 
            info!("Isseu reading the file: {}", error);
            return uefi::Status::VOLUME_CORRUPTED;
        },
    };

    let elf_header: &elf::ElfHeader = unsafe {
        match elf::ElfHeader::new(&bytes) {
            Ok(ptr) => ptr,
            Err(error) => return error,
        }
    };

    let ph_table: ProgramHeaderTable = match elf_header.new_ph_table(&bytes) {
        Ok(table) => table,
        Err(error) => return error,
    };

    let status = ph_table.load_segments(&bytes);

    if status != uefi::Status::SUCCESS {
        info!("Issue loading program headers!");
        return uefi::Status::COMPROMISED_DATA;
    }

    let mut mm: MemoryMapOwned = unsafe {
        boot::exit_boot_services(None)
    };

    let mm_ptr = unsafe {
        mm.buffer_mut().as_ptr()
    };

    // write address to register?
    unsafe {
        asm!{
            "mov rsi, {0}",
            in(reg) mm_ptr,
        }
    }

    elf_header.entry_start();

    // boot::stall(10_000_000);
    return status;
}
