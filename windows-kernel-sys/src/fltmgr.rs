#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::useless_transmute)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_cast)]

use crate::base::*;

include!(concat!(env!("OUT_DIR"), "/fltmgr.rs"));
use core::ptr::null_mut;

impl FLT_OPERATION_REGISTRATION {
    pub const IRP_MJ_CREATE: UCHAR = 0x00;
    pub const IRP_MJ_SET_INFORMATION: UCHAR = 0x06;
    pub const IRP_MJ_OPERATION_END: UCHAR = 0x80;

    pub const fn new() -> Self {
        FLT_OPERATION_REGISTRATION {
            MajorFunction: 0,
            Flags: 0,
            PreOperation: None,
            PostOperation: None,
            Reserved1: null_mut(),
        }
    }

    pub const fn set_major_function(&self, major_function: UCHAR) -> Self {
        FLT_OPERATION_REGISTRATION {
            MajorFunction: major_function,
            Flags: self.Flags,
            PreOperation: self.PreOperation,
            PostOperation: self.PostOperation,
            Reserved1: null_mut(),
        }
    }

    pub const fn set_flags(&self, flags: FLT_REGISTRATION_FLAGS) -> Self {
        FLT_OPERATION_REGISTRATION {
            MajorFunction: self.MajorFunction,
            Flags: flags,
            PreOperation: self.PreOperation,
            PostOperation: self.PostOperation,
            Reserved1: null_mut(),
        }
    }

    pub const fn set_preop(&self, preop: PFLT_PRE_OPERATION_CALLBACK) -> Self {
        FLT_OPERATION_REGISTRATION {
            MajorFunction: self.MajorFunction,
            Flags: self.Flags,
            PreOperation: preop,
            PostOperation: self.PostOperation,
            Reserved1: null_mut(),
        }
    }

    pub const fn set_postop(&self, postop: PFLT_POST_OPERATION_CALLBACK) -> Self {
        FLT_OPERATION_REGISTRATION {
            MajorFunction: self.MajorFunction,
            Flags: self.Flags,
            PreOperation: self.PreOperation,
            PostOperation: postop,
            Reserved1: null_mut(),
        }
    }
}
