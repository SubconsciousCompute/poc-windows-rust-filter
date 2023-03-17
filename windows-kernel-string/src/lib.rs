#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::upper_case_acronyms)]

extern crate alloc;

use alloc::string::String;
use core::fmt::{Display, Formatter};
use windows_kernel_sys::base;
use windows_kernel_sys::base::{BOOLEAN, LONG, NTSTATUS, TRUE, ULONG, ULONGLONG};

type PCSZ = *const u8;
type PCWSTR = *const u16;

#[repr(C)]
pub struct ANSI_STRING {
    pub Length: u16,
    pub MaximumLength: u16,
    pub Buffer: *const u8,
}
pub type PANSI_STRING = *mut ANSI_STRING;

impl ANSI_STRING {
    fn create(buffer: &[u8]) -> Self {
        ANSI_STRING::from(buffer)
    }
}

impl<'a> From<&'a [u8]> for ANSI_STRING {
    fn from(buffer: &'a [u8]) -> Self {
        let mut str = ANSI_STRING::default();

        let mut buffer = buffer.to_vec();

        if let Some(last_byte) = buffer.last() {
            if *last_byte != 0 {
                buffer.push(0);
            }
        }

        unsafe {
            RtlInitAnsiString(&mut str, buffer.as_ptr());
        }

        core::mem::forget(buffer);
        str
    }
}

impl<'a> From<&str> for ANSI_STRING {
    fn from(buffer: &str) -> Self {
        ANSI_STRING::from(buffer.as_bytes())
    }
}

impl Default for ANSI_STRING {
    fn default() -> Self {
        Self {
            Length: 0,
            MaximumLength: 0_u16,
            Buffer: core::ptr::null(),
        }
    }
}

#[repr(C)]
pub struct _UNICODE_STRING {
    pub Length: u16,
    pub MaximumLength: u16,
    pub ptr: *const u16,
}
pub type UNICODE_STRING = _UNICODE_STRING;
pub type PUNICODE_STRING = *mut UNICODE_STRING;
pub type PCUNICODE_STRING = *const UNICODE_STRING;

impl UNICODE_STRING {
    pub fn create(buffer: &str) -> Self {
        UNICODE_STRING::from(buffer.as_bytes())
    }

    pub fn as_base_unicode(&self) -> base::UNICODE_STRING {
        base::UNICODE_STRING {
            Length: self.Length,
            MaximumLength: self.MaximumLength,
            Buffer: self.ptr as *mut u16,
        }
    }

    pub fn as_rust_string(&self) -> String {
        unsafe {
            let ar = core::slice::from_raw_parts(self.ptr, self.Length as usize / 2);
            if let Ok(s) = String::from_utf16(ar) {
                s
            } else {
                String::new()
            }
        }
    }

    pub fn as_ptr(&self) -> *const base::UNICODE_STRING {
        self as *const Self as *const base::UNICODE_STRING
    }
}

impl From<base::UNICODE_STRING> for UNICODE_STRING {
    fn from(unicode: base::UNICODE_STRING) -> Self {
        UNICODE_STRING {
            Length: unicode.Length,
            MaximumLength: unicode.MaximumLength,
            ptr: unicode.Buffer,
        }
    }
}

impl<'a> From<&'a [u8]> for UNICODE_STRING {
    fn from(buffer: &'a [u8]) -> Self {
        UNICODE_STRING::from(&ANSI_STRING::create(buffer))
    }
}

impl<'a> From<&str> for UNICODE_STRING {
    fn from(buffer: &str) -> Self {
        UNICODE_STRING::from(buffer.as_bytes())
    }
}

impl<'a> From<&'a [u16]> for UNICODE_STRING {
    fn from(buffer: &'a [u16]) -> Self {
        let mut str = UNICODE_STRING::default();

        let mut buffer = buffer.to_vec();

        if let Some(last_byte) = buffer.last(){
            if *last_byte == 0{
                buffer.push(0);
            }
        }

        unsafe {
            RtlCreateUnicodeString(&mut str, buffer.as_ptr());
        }
        str
    }
}

impl<'a> From<&ANSI_STRING> for UNICODE_STRING {
    fn from(source: &ANSI_STRING) -> Self {
        let mut u = UNICODE_STRING::default();
        unsafe {
            RtlAnsiStringToUnicodeString(&mut u, source, TRUE as BOOLEAN);
        }
        u
    }
}

impl Display for UNICODE_STRING {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_rust_string())
    }
}

impl Default for UNICODE_STRING {
    fn default() -> Self {
        Self {
            Length: 0,
            MaximumLength: 0_u16,
            ptr: core::ptr::null(),
        }
    }
}

impl Drop for UNICODE_STRING {
    fn drop(&mut self) {
        unsafe { RtlFreeUnicodeString(self) }
    }
}

extern "system" {
    pub fn RtlInitAnsiString(DestinationString: &mut ANSI_STRING, SourceString: PCSZ);

    pub fn RtlCreateUnicodeString(
        DestinationString: &mut UNICODE_STRING,
        SourceString: PCWSTR,
    ) -> BOOLEAN;

    pub fn RtlIntegerToUnicodeString(
        Value: ULONG,
        Base: ULONG,
        String: &mut UNICODE_STRING,
    ) -> NTSTATUS;

    pub fn RtlInt64ToUnicodeString(
        Value: ULONGLONG,
        Base: ULONG,
        String: &mut UNICODE_STRING,
    ) -> NTSTATUS;

    pub fn RtlUnicodeStringToInteger(
        String: &UNICODE_STRING,
        Base: ULONG,
        Value: &mut ULONG,
    ) -> NTSTATUS;

    pub fn RtlUnicodeStringToAnsiString(
        DestinationString: &mut ANSI_STRING,
        SourceString: &UNICODE_STRING,
        AllocateDestination: BOOLEAN,
    ) -> NTSTATUS;

    pub fn RtlxUnicodeStringToAnsiSize(SourceString: &UNICODE_STRING) -> ULONG;

    pub fn RtlAnsiStringToUnicodeString(
        DestinationString: &mut UNICODE_STRING,
        SourceString: &ANSI_STRING,
        AllocateDestination: BOOLEAN,
    ) -> NTSTATUS;

    pub fn RtlxAnsiStringToUnicodeSize(SourceString: &ANSI_STRING) -> ULONG;

    pub fn RtlCompareUnicodeString(
        String1: &UNICODE_STRING,
        String2: &UNICODE_STRING,
        CaseInSensitive: BOOLEAN,
    ) -> LONG;

    pub fn RtlCompareString(
        String1: &ANSI_STRING,
        String2: &ANSI_STRING,
        CaseInSensitive: BOOLEAN,
    ) -> i32;

    pub fn RtlEqualUnicodeString(String1: &UNICODE_STRING, String2: &UNICODE_STRING) -> bool;

    pub fn RtlEqualString(String1: &ANSI_STRING, String2: &ANSI_STRING) -> bool;

    pub fn RtlFreeAnsiString(UnicodeString: &mut ANSI_STRING);

    pub fn RtlFreeUnicodeString(UnicodeString: &mut UNICODE_STRING);
}

#[allow(non_upper_case_globals)]
pub const RtlUnicodeStringToAnsiSize: unsafe extern "system" fn(
    SourceString: &UNICODE_STRING,
) -> ULONG = RtlxUnicodeStringToAnsiSize;

#[allow(non_upper_case_globals)]
pub const RtlAnsiStringToUnicodeSize: unsafe extern "system" fn(
    SourceString: &ANSI_STRING,
) -> ULONG = RtlxAnsiStringToUnicodeSize;
