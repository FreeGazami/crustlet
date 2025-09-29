#![no_main]
#![no_std]

// use core::panic::PanicInfo;
use log::info;
use uefi::boot::{self, SearchType};
use uefi::prelude::*;
use uefi::proto::device_path::text::{AllowShortcuts, DevicePathToText, DisplayOnly};
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::system_table_raw;
use uefi::{Identify, Result};

fn print_image_path() -> Result {
    let loaded_image = boot::open_protocol_exclusive::<LoadedImage>(boot::image_handle())?;

    let device_path_to_text_handle =
        *boot::locate_handle_buffer(SearchType::ByProtocol(&DevicePathToText::GUID))?
            .first()
            .expect("DevicePathToText is missing");

    let device_path_to_text =
        boot::open_protocol_exclusive::<DevicePathToText>(device_path_to_text_handle)?;

    let image_device_path = loaded_image.file_path().expect("File path is not set");
    let image_device_path_text = device_path_to_text
        .convert_device_path_to_text(image_device_path, DisplayOnly(true), AllowShortcuts(false))
        .expect("convert_device_path_to_text failed");

    info!("Image path: {}", &*image_device_path_text);

    return Ok(());
}

#[entry]
fn efi_main(mage_handle: Handle, system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init().unwrap();

    // print_image_path().unwrap();

    // load the elf format
    let mut system_table = match system_table_raw() {
        Some(table) => table,
        None => return Status::NOT_FOUND,
    };

    boot::stall(10_000_000);
    return Status::SUCCESS;
}
