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
use uefi_handoff::BootInfo;
use core::ffi::c_void;


#[cfg(target_arch="x86_64")]
#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().unwrap();

    info!("start?");

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

    // write address to register?
    let size = core::mem::size_of::<BootInfo>();
    let ptr = match boot::allocate_pool(MemoryType::LOADER_DATA, size) {
        Ok(pointer) => pointer,
        Err(error) => return Status::ABORTED,
    };

    let boot_info = ptr.cast::<BootInfo>();
    let system_table = match uefi::table::system_table_raw() {
        Some(non_null) => {
            non_null.as_ptr() as *mut u8
        },
        None => {
            return Status::ABORTED;
        }
    };
    let image_handle = boot::image_handle().as_ptr();

    info!("calling exit_boot_services");
    let mut mm: MemoryMapOwned = unsafe {
        boot::exit_boot_services(None)
    };

    let mm_ptr = unsafe {
        mm.buffer_mut().as_ptr()
    };

    unsafe {
        (*boot_info.as_ptr()).image_handle = image_handle;
        (*boot_info.as_ptr()).system_table = system_table;
        (*boot_info.as_ptr()).mm_ptr = unsafe {mm.buffer_mut()};
    }

    elf_header.entry_start(ptr.as_ptr());

    return Status::ABORTED;
}
