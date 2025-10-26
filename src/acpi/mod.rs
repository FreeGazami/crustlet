#![no_std]

use uefi_raw::table::configuration::ConfigurationTable;
use uefi::table::cfg::ACPI2_GUID;
use uefi_raw::table::system::SystemTable;

pub fn get_acpi_table_pointer(syst: *mut SystemTable) -> Option<*mut ConfigurationTable> {
    let configt_entry_number: usize = unsafe {(*syst).number_of_configuration_table_entries};
    let configuration_table: *mut ConfigurationTable = unsafe {(*syst).configuration_table};

    let configt_slice = unsafe {core::slice::from_raw_parts(configuration_table, configt_entry_number)};

    let acpi_index = match configt_slice.iter().position(|entry| entry.vendor_guid == ACPI2_GUID) {
        Some(index) => index,
        None => return None,
    };

    let acpi_t_ptr: *mut ConfigurationTable = unsafe {configuration_table.add(acpi_index)};

    return Some(acpi_t_ptr);
}