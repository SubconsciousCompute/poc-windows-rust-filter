#![allow(non_snake_case)]

use windows_kernel_sys::mutex::{ExAcquireFastMutex, ExInitializeFastMutex, ExReleaseFastMutex, FAST_MUTEX};
use crate::locker::Locker;

pub struct FastMutex {
    Mutex: FAST_MUTEX,
}

impl FastMutex {
    pub const fn new() -> Self {
        Self {
            Mutex: FAST_MUTEX::new(),
        }
    }
}

impl Locker for FastMutex {
    fn Init(&mut self) {
        unsafe { ExInitializeFastMutex(&mut self.Mutex) }
    }

    fn Lock(&mut self) {
        unsafe { ExAcquireFastMutex(&mut self.Mutex as *mut FAST_MUTEX) }
    }

    fn Unlock(&mut self) {
        unsafe { ExReleaseFastMutex(&mut self.Mutex as *mut FAST_MUTEX) }
    }
}