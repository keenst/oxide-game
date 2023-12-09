extern crate oxide;

use std::mem;
use std::fs;

pub mod win32;

pub static mut LIBRARY: Option<libloading::Library> = None;

static LIB_PATH: &str = "../oxide/target/debug/";

pub fn main() {
    load_lib();
    win32::start_program();
}

pub fn reload_lib() {
    unsafe { drop(mem::replace(&mut LIBRARY, None)); }
    load_lib();
}

fn load_lib() {
    fs::copy(format!("{}/oxide.dll", LIB_PATH), format!("{}/oxide_temp.dll", LIB_PATH)).expect("Unable to copy dll to temp");

    unsafe {
        LIBRARY = match libloading::Library::new(format!("{}/oxide_temp.dll", LIB_PATH)) {
            Ok(value) => Some(value),
            Err(error) => panic!("Unable to load oxide lib: {}", error)
        };
    }
}
