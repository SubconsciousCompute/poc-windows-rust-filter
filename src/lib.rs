#![no_std]
#![allow(non_snake_case)]

use core::panic::PanicInfo;
use core::ptr::null_mut;

use windows_kernel_sys::base::_FLT_PREOP_CALLBACK_STATUS::FLT_PREOP_SUCCESS_NO_CALLBACK;
use windows_kernel_sys::base::{
    DRIVER_OBJECT, FLT_FILESYSTEM_TYPE, FLT_FILTER_UNLOAD_FLAGS, FLT_INSTANCE_QUERY_TEARDOWN_FLAGS,
    FLT_INSTANCE_SETUP_FLAGS, FLT_INSTANCE_TEARDOWN_FLAGS, FLT_OPERATION_REGISTRATION,
    FLT_PREOP_CALLBACK_STATUS, FLT_REGISTRATION, FLT_REGISTRATION_VERSION, NTSTATUS,
    PCFLT_RELATED_OBJECTS, PFLT_CALLBACK_DATA, PFLT_FILTER, PVOID, STATUS_SUCCESS, ULONG,
    UNICODE_STRING, USHORT,
};
use windows_kernel_sys::fltmgr::{
    DbgPrint, FltRegisterFilter, FltStartFiltering, FltUnregisterFilter,
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

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

///
/// The minifilter handle that results from a call to FltRegisterFilter
/// NOTE: This handle must be passed to FltUnregisterFilter during minifilter unloading
///
static mut G_MINIFILTER_HANDLE: PFLT_FILTER = null_mut();


///
/// The FLT_REGISTRATION structure provides information about a file system minifilter to the filter manager.
///
const G_FILTER_REGISTRATION: FLT_REGISTRATION = FLT_REGISTRATION {
    Size: core::mem::size_of::<FLT_REGISTRATION>() as USHORT, //  Size
    Version: FLT_REGISTRATION_VERSION as USHORT,
    Flags: 0,
    ContextRegistration: null_mut(),
    OperationRegistration: G_CALLBACKS.as_ptr(),
    FilterUnloadCallback: Some(InstanceFilterUnloadCallback),
    InstanceSetupCallback: Some(InstanceSetupCallback),
    InstanceQueryTeardownCallback: Some(InstanceQueryTeardownCallback),
    InstanceTeardownStartCallback: Some(InstanceTeardownStartCallback),
    InstanceTeardownCompleteCallback: Some(InstanceTeardownCompleteCallback),
    GenerateFileNameCallback: None,
    NormalizeNameComponentCallback: None,
    NormalizeContextCleanupCallback: None,
    TransactionNotificationCallback: None,
    NormalizeNameComponentExCallback: None,
    SectionNotificationCallback: None,
};

#[link_section = "PAGE"]
unsafe extern "C" fn InstanceTeardownStartCallback(
    _flt_objects: PCFLT_RELATED_OBJECTS,
    _flags: FLT_INSTANCE_TEARDOWN_FLAGS,
) {
}

#[link_section = "PAGE"]
unsafe extern "C" fn InstanceTeardownCompleteCallback(
    _flt_objects: PCFLT_RELATED_OBJECTS,
    _flags: FLT_INSTANCE_TEARDOWN_FLAGS,
) {
}

///
/// Constant FLT_REGISTRATION structure for our filter.
/// This initializes the callback routines our filter wants to register for.
///
const G_CALLBACKS: &[FLT_OPERATION_REGISTRATION] = {
    &[
        FLT_OPERATION_REGISTRATION::new()
            .set_major_function(FLT_OPERATION_REGISTRATION::IRP_MJ_CREATE)
            .set_preop(Some(PreOperationCreate)),
        FLT_OPERATION_REGISTRATION::new()
            .set_major_function(FLT_OPERATION_REGISTRATION::IRP_MJ_OPERATION_END),
    ]
};

///
/// Pre-create callback to get file info during creation or opening
///
unsafe extern "C" fn PreOperationCreate(
    Data: PFLT_CALLBACK_DATA,
    _FltObjects: PCFLT_RELATED_OBJECTS,
    _CompletionContext: *mut PVOID,
) -> FLT_PREOP_CALLBACK_STATUS {
    let k = &(*(*(*Data).Iopb).TargetFileObject).FileName;

    unsafe {
        DbgPrint("%wZ\n".as_ptr() as _, k);
    }

    FLT_PREOP_SUCCESS_NO_CALLBACK
}

///
/// This is called before a filter is unloaded.
/// If NULL is specified for this routine, then the filter can never be unloaded.
///
extern "C" fn InstanceFilterUnloadCallback(_Flags: FLT_FILTER_UNLOAD_FLAGS) -> NTSTATUS {
    PAGED_CODE!();


    unsafe {
        DbgPrint("Unloading\0\n".as_ptr() as _);

        FltUnregisterFilter(G_MINIFILTER_HANDLE);
    }

    STATUS_SUCCESS
}

///
/// This is called to see if a filter would like to attach an instance to the given volume.
///
#[link_section = "PAGE"]
unsafe extern "C" fn InstanceSetupCallback(
    _flt_objects: PCFLT_RELATED_OBJECTS,
    _flags: FLT_INSTANCE_SETUP_FLAGS,
    _volume_device_type: ULONG,
    _volume_filesystem_type: FLT_FILESYSTEM_TYPE,
) -> NTSTATUS {
    PAGED_CODE!();

    STATUS_SUCCESS
}

///
/// This is called to see if the filter wants to detach from the given volume.
///
#[link_section = "PAGE"]
extern "C" fn InstanceQueryTeardownCallback(
    _flt_objects: PCFLT_RELATED_OBJECTS,
    _flags: FLT_INSTANCE_QUERY_TEARDOWN_FLAGS,
) -> NTSTATUS {
    PAGED_CODE!();

    unsafe {
        FltUnregisterFilter(G_MINIFILTER_HANDLE);
    }

    STATUS_SUCCESS
}

#[link_section = "INIT"]
#[no_mangle]
pub extern "system" fn DriverEntry(
    driver: &mut DRIVER_OBJECT,
    _registry_path: *const UNICODE_STRING,
) -> NTSTATUS {
    unsafe {
        DbgPrint("Hello from Rust!\0".as_ptr() as _);
    }

    //
    // register minifilter driver
    //
    let mut status: NTSTATUS =
        unsafe { FltRegisterFilter(driver, &G_FILTER_REGISTRATION, &mut G_MINIFILTER_HANDLE) };

    if !NT_SUCCESS!(status) {
        return status;
    }

    driver.DriverUnload = Some(driver_exit);


    //
    // start minifilter driver
    //
    status = unsafe { FltStartFiltering(G_MINIFILTER_HANDLE) };

    if !NT_SUCCESS!(status) {
        unsafe {
            FltUnregisterFilter(G_MINIFILTER_HANDLE);
        }
    }

    status
}

unsafe extern "C" fn driver_exit(_driver: *mut DRIVER_OBJECT) {
    FltUnregisterFilter(G_MINIFILTER_HANDLE);

    unsafe {
        DbgPrint("\nBye bye from Rust!\0".as_ptr() as _);
    }
}
