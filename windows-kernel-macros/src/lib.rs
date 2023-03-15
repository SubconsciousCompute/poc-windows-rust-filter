#![no_std]

#[macro_export]
macro_rules! NT_SUCCESS {
    ($status:expr) => {
        $status as NTSTATUS >= 0
    };
}

#[macro_export]
macro_rules! PAGED_CODE {
    () => {
        unsafe {
            if u64::from(windows_kernel_sys::fltmgr::KeGetCurrentIrql())
                > windows_kernel_sys::base::APC_LEVEL as u64
            {
                return windows_kernel_sys::base::STATUS_UNSUCCESSFUL;
            }
        }
    };
}

use windows_kernel_sys::c_void;

pub type PVOID = *mut c_void;
pub const NULL: PVOID = 0 as PVOID;

#[inline]
pub unsafe fn InitializeObjectAttributes(
    p: windows_kernel_sys::base::POBJECT_ATTRIBUTES,
    n: windows_kernel_string::PUNICODE_STRING,
    a: windows_kernel_sys::base::ULONG,
    r: windows_kernel_sys::base::HANDLE,
    s: windows_kernel_sys::base::PVOID,
) {
    let mut n = windows_kernel_sys::base::_UNICODE_STRING{
        Length: (*n).Length,
        MaximumLength: (*n).MaximumLength,
        Buffer: (*n).ptr as *mut u16,
    };

    use core::mem::size_of;
    (*p).Length = size_of::<windows_kernel_sys::base::OBJECT_ATTRIBUTES>() as windows_kernel_sys::base::ULONG;
    (*p).RootDirectory = r;
    (*p).Attributes = a;
    (*p).ObjectName = &mut n;
    (*p).SecurityDescriptor = s;
    (*p).SecurityQualityOfService = NULL;
}