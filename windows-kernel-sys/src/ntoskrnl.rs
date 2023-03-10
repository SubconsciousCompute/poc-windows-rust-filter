#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::base::*;

include!(concat!(env!("OUT_DIR"), "/ntoskrnl.rs"));

#[link(name = "wrapper_ntoskrnl")]
extern "C" {
    pub fn _IoGetCurrentIrpStackLocation(irp: PIRP) -> PIO_STACK_LOCATION;
}

pub use self::_IoGetCurrentIrpStackLocation as IoGetCurrentIrpStackLocation;
