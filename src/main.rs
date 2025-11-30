//! File: main.rs
//! Author: Tomo Kaneko
//! Initial Date: 2025/10/05

#![no_std]
#![no_main]

extern crate alloc;

mod elf;
mod acpi;
mod parser;
mod configs;

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use core::arch::asm;
use core::ffi::c_void;
use elf::*;
use log::info;
use uefi::boot::{MemoryDescriptor, MemoryType};
use uefi::boot::{self, SearchType};
use uefi::CString16;
use uefi::fs::{FileSystem, FileSystemResult};
use uefi::Error;
use uefi_handoff::BootInfo;
use uefi::{Identify, Result};
use uefi::mem::memory_map::{MemoryMapOwned, MemoryMapIter, MemoryMap, MemoryMapKey, MemoryMapMut};
use uefi::prelude::*;
use uefi::proto::device_path::text::{AllowShortcuts, DevicePathToText, DisplayOnly};
use uefi::proto::loaded_image::LoadedImage;
use uefi_raw::table::runtime::{RuntimeServices, ResetType};
use uefi_raw::table::system::SystemTable;
use uefi_raw::table::configuration::ConfigurationTable;
use uefi_raw::table::boot::{BootServices};
use uefi::table::cfg::ACPI2_GUID;
use uefi_raw::protocol::console::{GraphicsOutputProtocol, GraphicsOutputProtocolMode, GraphicsOutputModeInformation};

use parser::*;

use alloc::string::String;


use acpi::get_acpi_table_pointer;

fn load_file(path: &str) -> Result<Vec<u8>, uefi::Status> {
    let path_CString16 = CString16::try_from(path).unwrap();
    let img_fs = boot::get_image_file_system(boot::image_handle()).unwrap();
    let mut fs_handle = FileSystem::new(img_fs);

    let bytes: Vec<u8> = match fs_handle.read(path_CString16.as_ref()) {
        Ok(vec) => vec,
        Err(error) => {
            let status = match error {
                uefi::fs::Error::Io(_) => uefi_raw::Status::DEVICE_ERROR,
                uefi::fs::Error::Path(_) => uefi_raw::Status::NOT_FOUND,
                uefi::fs::Error::Utf8Encoding(_) => uefi_raw::Status::VOLUME_CORRUPTED,
                _ => uefi_raw::Status::UNSUPPORTED,
            };

            // let e_final = uefi::Error::from(status);
            return Err(uefi::Error::new(status, status));
        },
    };

    Ok(bytes)
}


#[cfg(target_arch="x86_64")]
#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().unwrap();

    /* TODO: add parser, load kernel path and rootfs from rEnv.txt dynamically */
    let r_env_path = configs::ENVTXT;
    let env_bytes: Vec<u8> = load_file(r_env_path).unwrap();
    let mut configs: BTreeMap<String, String> = parser::parse_env(env_bytes);

    let kimg = match configs.get(configs::IMG_KEY) {
        Some(value) => value,
        None => configs::DEFAULT_IMG,
    };

    info!("loading kernel: {}", kimg);
    let kimg_bytes: Vec<u8> = load_file(&kimg).unwrap();

    let elf_header: &elf::ElfHeader = unsafe {
        match elf::ElfHeader::new(&kimg_bytes) {
            Ok(ptr) => ptr,
            Err(error) => return error,
        }
    };

    let ph_table: ProgramHeaderTable = match elf_header.new_ph_table(&kimg_bytes) {
        Ok(table) => table,
        Err(error) => return error,
    };

    if ph_table.load_segments(&kimg_bytes) != uefi::Status::SUCCESS {
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

    let boot_services: *mut BootServices = unsafe {(*system_table).boot_services};

    let image_handle = boot::image_handle().as_ptr() as *mut c_void;

    let runtime_services: *mut c_void = unsafe { 
        ((*system_table).runtime_services) as *mut c_void
    };

    let acpi_t_ptr: *mut ConfigurationTable = match get_acpi_table_pointer(system_table) {
        Some(pointer) => pointer,
        None => return uefi::Status::ABORTED,
    };

    let mut cvoid_gop: *mut c_void = core::ptr::null_mut();
    let double_gop: *mut *mut c_void = &mut cvoid_gop as *mut *mut c_void;

    unsafe {
        match ((*boot_services).locate_protocol)(&GraphicsOutputProtocol::GUID, core::ptr::null_mut(), double_gop) {
            uefi::Status::SUCCESS => (),
            status_else => {
                info!("Issue getting protocol: {:?}", status_else);
                return status_else;
            },
        }
    }

    let gop: *mut GraphicsOutputProtocol = cvoid_gop as *mut GraphicsOutputProtocol;

    // get gop stuff
    let mode: *mut GraphicsOutputProtocolMode = unsafe{ (*gop).mode };

    let mode_info: *mut GraphicsOutputModeInformation = unsafe {(*mode).info};

    let mut mm: MemoryMapOwned = unsafe {
        boot::exit_boot_services(None)
    };

    unsafe {
        (*boot_info).image_handle = image_handle;
        (*boot_info).runtime_services = runtime_services;
        (*boot_info).mm = mm.buffer_mut().as_ptr() as *mut c_void;
        (*boot_info).mm_len = mm.len();
        (*boot_info).acpi_table = acpi_t_ptr as *mut c_void;
        (*boot_info).frame_buffer_base = (*mode).frame_buffer_base;
        (*boot_info).frame_buffer_size = (*mode).frame_buffer_size;
        (*boot_info).info = *((*mode).info);
    }

    // jump to elf e_entry address
    unsafe {
        elf_header.entry_start(boot_info as *mut c_void)
    }

    return Status::ABORTED;
}
