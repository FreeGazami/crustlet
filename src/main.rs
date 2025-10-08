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
use uefi::mem::memory_map::{MemoryMapOwned};
use uefi::boot::{MemoryDescriptor, MemoryType};


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

    // get memory map for runtime services
    let runtime_mm_data: MemoryMapOwned = uefi::boot::memory_map(MemoryType::RUNTIME_SERVICES_DATA).expect("FAILED to get runtime service code memory map");
    let runtime_mm_code: MemoryMapOwned = uefi::boot::memory_map(MemoryType::RUNTIME_SERVICES_CODE).expect("FAILED to get runtime service code memory map");

    // get memory map for ACPI
    let acpi_mm_reclaim: MemoryMapOwned = uefi::boot::memory_map(MemoryType::ACPI_RECLAIM).expect("failed to get acpi reclaim mm");
    let acpi_mm_nvolatile: MemoryMapOwned = uefi::boot::memory_map(MemoryType::ACPI_NON_VOLATILE).expect("failed to get acpi reclaim mm");

    // boot::exit_boot_services();
    // elf_header.entry_start();

    boot::stall(10_000_000);
    return status;
}
