use alloc::format;
use uefi::{
    CStr16, CString16,
    runtime::{VariableAttributes, VariableVendor},
};

use crate::timer;
const BLI_VENDOR: VariableVendor =
    VariableVendor(uefi::guid!("4a67b082-0a4c-41cf-b6c7-440b29bb8c4f"));

fn set_loader_var(name: &CStr16, value: &str) -> uefi::Result {
    let value16 =
        CString16::try_from(value).map_err(|_| uefi::Error::from(uefi::Status::LOAD_ERROR))?;
    uefi::runtime::set_variable(
        name,
        &BLI_VENDOR,
        VariableAttributes::BOOTSERVICE_ACCESS | VariableAttributes::RUNTIME_ACCESS,
        value16.as_bytes(),
    )
}

pub fn set_loader_info(entry_name: &str) {
    let loader_info = alloc::format!("ferriboot {}", env!("CARGO_PKG_VERSION"));

    if let Err(e) = set_loader_var(uefi::cstr16!("LoaderInfo"), &loader_info) {
        uefi::println!(
            "ferriboot: warning: cannot set LoaderInfo ({:?})",
            e.status()
        );
    }
    if let Err(e) = set_loader_var(uefi::cstr16!("LoaderEntrySelected"), entry_name) {
        uefi::println!(
            "ferriboot: warning: cannot set LoaderEntrySelected ({:?})",
            e.status()
        );
    }
}

pub fn set_loader_times(init_tsc: u64) {
    let Some(freq) = timer::frequency() else {
        return;
    };

    let exec_tsc = timer::read_tsc();

    let init_us = timer::ticks_to_us(init_tsc, freq);
    let exec_us = timer::ticks_to_us(exec_tsc, freq);

    if let Err(e) = set_loader_var(uefi::cstr16!("LoaderTimeInitUSec"), &format!("{}", init_us)) {
        uefi::println!(
            "ferriboot: warning: cannot set LoaderTimeInitUSec ({:?})",
            e.status()
        );
    }
    if let Err(e) = set_loader_var(uefi::cstr16!("LoaderTimeExecUSec"), &format!("{}", exec_us)) {
        uefi::println!(
            "ferriboot: warning: cannot set LoaderTimeExecUSec ({:?})",
            e.status()
        );
    }
}
