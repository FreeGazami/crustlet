#![no_std]
#![no_main]


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


#[entry]
fn osloader_main() -> Status {
    uefi::helpers::init().unwrap();

    let path: CString16 = CString16::try_from("gazami").unwrap();
    let p_fs = boot::get_image_file_system(boot::image_handle()).unwrap();

    let mut fs = FileSystem::new(p_fs);
    let bytes = match fs.read(path.as_ref()) {
        Ok(vector) => vector,
        Err(error) => panic!("Isseu reading the file: {}", error),
    };

    let elf_header: &elf::ElfHeader = unsafe {
        match elf::ElfHeader::new(&bytes) {
            Ok(ptr) => ptr,
            Err(error) => return error,
        }
    };

    if elf_header.check_magic() {
        info!("checked the elf header magic");
    }

    elf_header.dump_info();

    // load program header into memory
    let ph_table: &[elf::ProgramHeader] = unsafe {
        match elf_header.new_ph_table(&bytes) {
            Ok(table) => table,
            Err(error) => return error,
        }
    };

    info!("test entry[0]: 0x{:x}", ph_table[0].p_type);
    info!("test entry[1]: 0x{:x}", ph_table[1].p_type);
    info!("slice length:  {}", ph_table.len());
    info!("e_phnum: {}", elf_header.e_phnum);


    boot::stall(10_000_000);
    return Status::SUCCESS;
}
