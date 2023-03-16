#![no_std]
#![allow(non_snake_case)]

extern crate alloc;

use windows_kernel_alloc;
use windows_kernel_alloc::kernel_alloc::POOL_TAG;

use core::panic::PanicInfo;
use core::ptr;
use core::ptr::null_mut;
use windows_kernel_macros::{InitializeObjectAttributes, NT_SUCCESS, PAGED_CODE};
use windows_kernel_string::UNICODE_STRING;

use windows_kernel_sys::base::_FLT_PREOP_CALLBACK_STATUS::FLT_PREOP_SUCCESS_NO_CALLBACK;
use windows_kernel_sys::base::{
    DRIVER_OBJECT, FLT_FILESYSTEM_TYPE, FLT_FILTER_UNLOAD_FLAGS, FLT_INSTANCE_QUERY_TEARDOWN_FLAGS,
    FLT_INSTANCE_SETUP_FLAGS, FLT_INSTANCE_TEARDOWN_FLAGS, FLT_OPERATION_REGISTRATION,
    FLT_PORT_ALL_ACCESS, FLT_PREOP_CALLBACK_STATUS, FLT_REGISTRATION, FLT_REGISTRATION_VERSION,
    NTSTATUS, OBJECT_ATTRIBUTES, OBJ_CASE_INSENSITIVE, OBJ_KERNEL_HANDLE, PCFLT_RELATED_OBJECTS,
    PCHAR, PFLT_CALLBACK_DATA, PFLT_FILTER, PFLT_PORT, PSECURITY_DESCRIPTOR, PULONG, PVOID,
    STATUS_SUCCESS, ULONG, USHORT,
};
use windows_kernel_sys::c_int;
use windows_kernel_sys::fltmgr::{
    strcpy, DbgPrint, FltBuildDefaultSecurityDescriptor, FltCloseClientPort,
    FltCloseCommunicationPort, FltCreateCommunicationPort, FltFreeSecurityDescriptor,
    FltRegisterFilter, FltStartFiltering, FltUnregisterFilter,
};

static mut PORT: PFLT_PORT = null_mut();
static mut CLIENT_PORT: PFLT_PORT = null_mut();

/// The minifilter handle that results from a call to FltRegisterFilter
/// NOTE: This handle must be passed to FltUnregisterFilter during minifilter unloading
static mut G_MINIFILTER_HANDLE: PFLT_FILTER = null_mut();

/// The FLT_REGISTRATION structure provides information about a file system minifilter to the filter manager.
const G_FILTER_REGISTRATION: FLT_REGISTRATION = FLT_REGISTRATION {
    Size: core::mem::size_of::<FLT_REGISTRATION>() as USHORT, //  Size
    Version: FLT_REGISTRATION_VERSION as USHORT,
    Flags: 0,
    ContextRegistration: null_mut(),
    OperationRegistration: G_CALLBACKS.as_ptr(),
    FilterUnloadCallback: Some(InstanceFilterUnloadCallback),
    InstanceSetupCallback: None,            //Some(InstanceSetupCallback),
    InstanceQueryTeardownCallback: None,    // Some(InstanceQueryTeardownCallback),
    InstanceTeardownStartCallback: None,    //Some(InstanceTeardownStartCallback),
    InstanceTeardownCompleteCallback: None, //Some(InstanceTeardownCompleteCallback),
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
        // FLT_OPERATION_REGISTRATION::new()
        // .set_major_function(FLT_OPERATION_REGISTRATION::IRP_MJ_CREATE)
        // .set_preop(Some(PreOperationCreate)),
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
        DbgPrint("%wZ\n\0".as_ptr() as _, k);
    }

    FLT_PREOP_SUCCESS_NO_CALLBACK
}

/// This is called before a filter is unloaded.
/// If NULL is specified for this routine, then the filter can never be unloaded.
extern "C" fn InstanceFilterUnloadCallback(_Flags: FLT_FILTER_UNLOAD_FLAGS) -> NTSTATUS {
    PAGED_CODE!();

    unsafe {
        DbgPrint("Unloading rust minifilter\n\0".as_ptr() as _);
        FltCloseCommunicationPort(PORT);

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
    let mut sd: PSECURITY_DESCRIPTOR = null_mut();
    let mut oa: OBJECT_ATTRIBUTES = unsafe { core::mem::zeroed() };
    let mut name = "\\mf";

    unsafe {
        DbgPrint("Hello from Rust!\n\0".as_ptr() as _);
    }

    // driver.DriverUnload = Some(driver_exit);

    //
    // register minifilter driver
    //
    let mut status: NTSTATUS =
        unsafe { FltRegisterFilter(driver, &G_FILTER_REGISTRATION, &mut G_MINIFILTER_HANDLE) };

    unsafe {
        DbgPrint("1 Here\n\0".as_ptr() as _);
    }

    if !NT_SUCCESS!(status) {
        return status;
    }

    unsafe {
        DbgPrint("2 Here\n\0".as_ptr() as _);
    }

    status = unsafe { FltBuildDefaultSecurityDescriptor(&mut sd, FLT_PORT_ALL_ACCESS) };

    let name = UNICODE_STRING::create(name);

    if NT_SUCCESS!(status) {
        unsafe {
            InitializeObjectAttributes(
                &mut oa,
                &mut name.as_base_unicode(),
                OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
                null_mut(),
                sd,
            );
        }
        unsafe {
            DbgPrint("3 Here\n\0".as_ptr() as _);
        }

        status = unsafe {
            FltCreateCommunicationPort(
                G_MINIFILTER_HANDLE,
                &mut PORT,
                &mut oa,
                null_mut(),
                Some(MiniConnect),
                Some(MiniDisconnect),
                Some(MiniSendRec),
                1,
            )
        };

        unsafe {
            FltFreeSecurityDescriptor(sd);
        }

        unsafe {
            DbgPrint("4 Here\n\0".as_ptr() as _);
        }

        if NT_SUCCESS!(status) {
            unsafe {
                DbgPrint("5 Here\n\0".as_ptr() as _);
            }
            // driver.DriverUnload = Some(driver_exit);

            // start minifilter driver
            status = unsafe { FltStartFiltering(G_MINIFILTER_HANDLE) };

            if !NT_SUCCESS!(status) {
                unsafe {
                    DbgPrint("6 Here\0\n".as_ptr() as _);
                }
                unsafe {
                    FltUnregisterFilter(G_MINIFILTER_HANDLE);
                }
            }
        } else {
            unsafe {
                FltCloseCommunicationPort(PORT);
            }
        }
    }

    status
}

unsafe extern "C" fn MiniConnect(
    ClientPort: PFLT_PORT,
    ServerPortCookie: PVOID,
    ConnectionContext: PVOID,
    SizeOfContext: ULONG,
    ConnectionPortCookie: *mut PVOID,
) -> NTSTATUS {
    CLIENT_PORT = ClientPort;
    DbgPrint("Rust connect from application\n\0".as_ptr() as _);

    STATUS_SUCCESS
}

unsafe extern "C" fn MiniDisconnect(ConnectionCookie: PVOID) {
    DbgPrint("Rust disconnect form application\n\0".as_ptr() as _);
    FltCloseClientPort(G_MINIFILTER_HANDLE, &mut CLIENT_PORT);
}

unsafe extern "C" fn MiniSendRec(
    PortCookie: PVOID,
    InputBuffer: PVOID,
    InputBufferLength: ULONG,
    OutputBuffer: PVOID,
    OutputBufferLength: ULONG,
    ReturnOutputBufferLength: PULONG,
) -> NTSTATUS {
    let mut msg: PCHAR = "Rust from kernel\n\0".as_bytes().as_ptr() as *mut i8;
    DbgPrint(
        "Rust message from application: %s".as_ptr() as _,
        InputBuffer as *mut i8,
    );

    strcpy(OutputBuffer as PCHAR, msg);

    STATUS_SUCCESS
}

/*
unsafe extern "C" fn driver_exit(_driver: *mut DRIVER_OBJECT) {
    FltUnregisterFilter(G_MINIFILTER_HANDLE);

    unsafe {
        DbgPrint("\nBye bye from Rust!\0".as_ptr() as _);
    }
}
*/
