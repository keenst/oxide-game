pub mod win32;

use std::ffi::c_void;

pub struct GameState {
    green_offset: u8,
    blue_offset: u8
}

pub struct GameMemory {
    is_initialized: bool,
    permanent_storage_size: u64,
    permanent_storage: *mut c_void,
    transient_storage_size: u64,
    transient_storage: *mut c_void
}

pub fn main() {
    let result = call_dynamic();
    match result {
        Ok(num) => println!("{}", num),
        Err(error) => panic!("Error: {}", error)
    };

    win32::start_program();
}

pub fn call_dynamic() -> Result<u32, Box<dyn std::error::Error>> {
    unsafe {
        let lib = libloading::Library::new("../oxide/target/debug/oxide.dll")?;
        let func: libloading::Symbol<unsafe extern fn() -> u32> = lib.get(b"get_message")?;
        Ok(func())
    }
}
