use std::ffi::c_void;
use std::ptr::null;

use windows_sys::w;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Storage::InstallableFileSystems::{
    FilterConnectCommunicationPort, FilterSendMessage,
};

static mut PORT: HANDLE = -1;

fn main() {
    let mut byterec = 0;
    let buffer = "Hello from Rust user space application\n\0"
        .as_bytes()
        .as_ptr()
        .cast::<c_void>();
    let bufferlen = 50;
    let rbuffer_size = 256;
    let mut rbuffer: Vec<u8> = vec![0; rbuffer_size];
    let recbuffer: *mut c_void = rbuffer.as_mut_ptr().cast::<c_void>();

    unsafe {
        if PORT == -1
            && FilterConnectCommunicationPort(w!("\\mf"), 0, null(), 0, null(), &mut PORT) != 0
        {
            panic!("Port connection failed");
        }
    }

    println!("Connection to minifilter was successful!!!");

    unsafe {
        if FilterSendMessage(PORT, buffer, bufferlen as u32, recbuffer, 50, &mut byterec) != 0 {
            println!("Failed to get kernel message");
        } else {
            let pchar = recbuffer.cast::<i8>();
            let string = std::ffi::CStr::from_ptr(pchar)
                .to_str()
                .expect("Not a valid String");
            println!("Message from minifilter: {string}");
        }
    }

    println!("\nPress ctrl-z to ctrl-c to exit...\n");

    loop {}
}
