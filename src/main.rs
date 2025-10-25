//! File: main.rs
//! Author: Tomo Kaneko
//! Initial Date: 2025/10/05

#![no_std]
#![no_main]

extern crate alloc;

mod elf;

use alloc::vec::Vec;
use core::arch::asm;
use core::ffi::c_void;
use elf::*;
use log::info;
use uefi::boot::{MemoryDescriptor, MemoryType};
use uefi::boot::{self, SearchType};
use uefi::CString16;
use uefi::fs::{FileSystem, FileSystemResult};
use uefi_handoff::BootInfo;
use uefi::{Identify, Result};
use uefi::mem::memory_map::{MemoryMapOwned, MemoryMapIter, MemoryMap, MemoryMapKey, MemoryMapMut};
use uefi::prelude::*;
use uefi::proto::device_path::text::{AllowShortcuts, DevicePathToText, DisplayOnly};
use uefi::proto::loaded_image::LoadedImage;
use uefi_raw::table::runtime::{RuntimeServices, ResetType};
use uefi_raw::table::system::SystemTable;
use uefi_raw::table::configuration::ConfigurationTable;


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

    let size = core::mem::size_of::<BootInfo>();
    let ptr = match boot::allocate_pool(MemoryType::LOADER_DATA, size) {
        Ok(pointer) => pointer,
        Err(error) => return Status::ABORTED,
    };

    let boot_info: *mut BootInfo = ptr.as_ptr() as *mut BootInfo;

    let system_table = match uefi::table::system_table_raw() {
        Some(non_null) => {
            non_null.as_ptr() as *mut SystemTable
        },
        None => {
            return Status::ABORTED;
        }
    };

    let image_handle = boot::image_handle().as_ptr() as *mut c_void;

    let runtime_services: *mut c_void = unsafe { 
        ((*system_table).runtime_services) as *mut c_void
    };

    let configuration_table: *mut ConfigurationTable = unsafe {(*system_table).configuration_table};

    let config_table_p: *mut c_void = configuration_table as *mut c_void;


    let mut mm: MemoryMapOwned = unsafe {
        boot::exit_boot_services(None)
    };

    unsafe {
        (*boot_info).image_handle = image_handle;
        (*boot_info).runtime_services = runtime_services;
        (*boot_info).mm = mm.buffer_mut().as_ptr() as *mut c_void;
        (*boot_info).mm_len = mm.len();
        (*boot_info).configuration_table = config_table_p;
    }

    unsafe {
        elf_header.entry_start(boot_info as *mut c_void)
    }

    return Status::ABORTED;
}
