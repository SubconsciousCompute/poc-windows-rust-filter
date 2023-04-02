// source idea: https://os.phil-opp.com/minimal-rust-kernel/

#![no_std]
#![feature(lang_items)]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::string::ToString;
use core::{ffi::c_void, panic::PanicInfo};

pub mod kernel_alloc;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL: kernel_alloc::KernelAlloc = kernel_alloc::KernelAlloc;

#[cfg(not(test))]
#[export_name = "_fltused"]
static _FLTUSED: i32 = 0;

/// When using the alloc crate it seems like it does some unwinding. Adding this
/// export satisfies the compiler but may introduce undefined behaviour when a
/// panic occurs.
#[cfg(not(test))]
#[no_mangle]
extern "system" fn __CxxFrameHandler3(_: *mut u8, _: *mut u8, _: *mut u8, _: *mut u8) -> i32 {
    unimplemented!()
}

/// Base code
const BUGCHECK_CODE: u32 = 0xDEAD0000;

#[cfg(not(test))]
#[cfg_attr(all(target_env = "msvc", feature = "kernel"), link(name = "ntoskrnl"))]
extern "system" {
    fn KeBugCheckEx(
        BugCheckCode: u32,
        BugCheckParameter1: u32,
        BugCheckParameter2: u32,
        BugCheckParameter3: u32,
        BugCheckParameter4: u32,
    ) -> c_void;
}

#[cfg(not(test))]
/// An unrecoverable error will cause the kernel to crash.
fn unrecoverable_error(info: &PanicInfo) {
    let msg = info.to_string();
    unsafe {
        KeBugCheckEx(BUGCHECK_CODE, msg.as_ptr() as u32, 0, 0, 0);
    }
}

// TODO: configuration option that allows users to choose how to handle unrecoverable errors (e.g. log them, crash the kernel, etc.)
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unrecoverable_error(info);
    loop {}
}

#[cfg(not(test))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
