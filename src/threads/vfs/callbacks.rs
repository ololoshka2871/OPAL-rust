#![allow(dead_code)]

use alloc::string::String;
use freertos_rust::Duration;
use heatshrink_rust::{decoder::HeatshrinkDecoder, CompressedData};

use super::StaticBinData;

pub(crate) unsafe extern "C" fn const_binary_reader(
    dest: *mut u8,
    size: i32,
    offset: u32,
    userdata: usize,
) {
    let dptr = &*(userdata as *const StaticBinData);
    if offset as usize > dptr.data.len() {
        return;
    }
    let to_read = if offset as usize + size as usize > dptr.data.len() {
        dptr.data.len() - offset as usize
    } else {
        size as usize
    };

    core::ptr::copy_nonoverlapping(dptr.data.as_ptr().add(offset as usize), dest, to_read);
}

pub(crate) unsafe extern "C" fn unpack_reader(
    dest: *mut u8,
    size: i32,
    offset: u32,
    userdata: usize,
) {
    let dptr = &*(userdata as *const CompressedData);
    if offset as usize > dptr.original_size {
        return;
    }
    let to_read = if (offset as usize + size as usize) > dptr.original_size {
        dptr.original_size - offset as usize
    } else {
        size as usize
    };

    HeatshrinkDecoder::source(dptr.data.iter().cloned())
        .skip(offset as usize)
        .take(to_read)
        .enumerate()
        .for_each(|(n, d)| *dest.add(n) = d);
}

//pub (crate) unsafe extern "C" fn null_read(_dest: *mut u8, _size: i32, _offset: u32, _userdata: usize) {}

pub(crate) unsafe fn store_block_data(s: String, dest: *mut u8, size: i32, _offset: u32) {
    let src = s.as_bytes();
    let offset = _offset as usize;
    if src.len() > offset {
        let src = &src[offset..];
        let to_write = core::cmp::min(size as usize, src.len());
        core::ptr::copy_nonoverlapping(src.as_ptr(), dest, to_write);

        // забиваем буфер пробелами до конца, чтобы в блокноте он нормально выглядел
        core::ptr::write_bytes(dest.add(src.len()), b' ', size as usize - to_write);
    } else {
        // все пробелами забить
        core::ptr::write_bytes(dest, b' ', size as usize);
    }
}

pub(crate) unsafe extern "C" fn settings_read(
    dest: *mut u8,
    size: i32,
    offset: u32,
    _userdata: usize,
) {
    match crate::settings::settings_action(Duration::ms(5), |(ws, _)| {
        serde_json::to_string_pretty(&ws)
    }) {
        Ok(s) => store_block_data(s, dest, size, offset),
        Err(crate::settings::SettingActionError::AccessError(e)) => {
            defmt::error!("Failed to serialise settings: {}", defmt::Debug2Format(&e));
        }
        Err(crate::settings::SettingActionError::ActionError(e)) => {
            defmt::error!(
                "Failed to serialise settings: {}",
                defmt::Display2Format(&e)
            );
        }
    }
}

pub(crate) unsafe extern "C" fn meminfo_read(
    dest: *mut u8,
    size: i32,
    offset: u32,
    _userdata: usize,
) {
    use serde::Serialize;

    #[allow(non_snake_case)]
    #[derive(Serialize)]
    struct MemInfo {
        FlashPageSize: u32,
        FlashPages: u32,
        FlashUsedPages: u32,
    }

    let info = MemInfo {
        FlashPageSize: crate::main_data_storage::flash_page_size(),
        FlashPages: crate::main_data_storage::flash_size_pages(),
        FlashUsedPages: crate::main_data_storage::find_next_empty_page(0).unwrap_or_default(),
    };

    match serde_json::to_string_pretty(&info) {
        Ok(s) => store_block_data(s, dest, size, offset),
        Err(e) => defmt::error!(
            "Failed to serialise flash info: {}",
            defmt::Display2Format(&e)
        ),
    }
}

pub(crate) unsafe extern "C" fn master_read(
    dest: *mut u8,
    _size: i32,
    _offset: u32,
    userdata: usize,
) {
    let boxed = alloc::boxed::Box::from_raw(
        userdata as *mut crate::sensors::freqmeter::master_counter::MasterTimerInfo,
    );

    let s = alloc::format!("0x{:08X}", boxed.value().0);
    core::ptr::copy_nonoverlapping(s.as_ptr(), dest, s.len());

    core::mem::forget(boxed);
}

pub(crate) unsafe extern "C" fn flash_read(
    dest: *mut u8,
    size: i32,
    offset: u32,
    _userdata: usize,
) {
    use crate::main_data_storage::*;

    if let Ok(page) = select_page(offset / flash_page_size()) {
        page.read_to(
            (offset % flash_page_size()) as usize,
            core::slice::from_raw_parts_mut(dest, size as usize),
        );
    }
}
