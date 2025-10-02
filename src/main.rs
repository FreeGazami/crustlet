#![no_main]
#![no_std]

mod elf;

// use core::panic::PanicInfo;
use log::info;
use uefi::boot::{self, SearchType};
use uefi::prelude::*;
use uefi::proto::device_path::text::{AllowShortcuts, DevicePathToText, DisplayOnly};
use uefi::proto::loaded_image::LoadedImage;
// use uefi::table::system_table_raw;
use uefi::{Identify, Result};
use uefi::CString16;
use uefi::fs::{FileSystem, FileSystemResult};
use elf::*;

// fn print_image_path() -> Result {
//     let loaded_image = boot::open_protocol_exclusive::<LoadedImage>(boot::image_handle())?;
// 
//     let device_path_to_text_handle =
//         *boot::locate_handle_buffer(SearchType::ByProtocol(&DevicePathToText::GUID))?
//             .first()
//             .expect("DevicePathToText is missing");
// 
//     let device_path_to_text =
//         boot::open_protocol_exclusive::<DevicePathToText>(device_path_to_text_handle)?;
// 
//     let image_device_path = loaded_image.file_path().expect("File path is not set");
//     let image_device_path_text = device_path_to_text
//         .convert_device_path_to_text(image_device_path, DisplayOnly(true), AllowShortcuts(false))
//         .expect("convert_device_path_to_text failed");
// 
//     info!("Image path: {}", &*image_device_path_text);
// 
//     return Ok(());
// }

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

    info!("elf magic: {:x}{}{}{}", elf_header.e_ident[0], elf_header.e_ident[1] as char, elf_header.e_ident[2] as char, elf_header.e_ident[3] as char);
    info!("elf entry address: 0x{:x}", elf_header.e_entry);
    info!("endian: 0x{:x}", elf_header.e_ident[5] as u8);

    if elf_header.check_magic() {
        info!("checked the elf header magic");
    }

    boot::stall(10_000_000);
    return Status::SUCCESS;
}
