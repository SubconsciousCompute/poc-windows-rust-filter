#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::useless_transmute)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_cast)]

pub use cty::*;

include!(concat!(env!("OUT_DIR"), "/base.rs"));

pub const STATUS_SUCCESS: NTSTATUS = 0x00000000;
pub const STATUS_UNSUCCESSFUL: NTSTATUS = -1073741823i32;
