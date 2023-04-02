use core::ptr::null_mut;
use crate::base::{BOOLEAN, EVENT_TYPE, FALSE, LIST_ENTRY, LONG, PVOID, ULONG};
use crate::base::_EVENT_TYPE::SynchronizationEvent;

macro_rules! STRUCT {
    (#[debug] $($rest:tt)*) => (
        STRUCT!{#[cfg_attr(feature = "impl-debug", derive(Debug))] $($rest)*}
    );
    ($(#[$attrs:meta])* struct $name:ident {
        $($field:ident: $ftype:ty,)+
    }) => (
        #[repr(C)] #[derive(Copy)] $(#[$attrs])*
        pub struct $name {
            $(pub $field: $ftype,)+
        }
        impl Clone for $name {
            #[inline]
            fn clone(&self) -> $name { *self }
        }
        #[cfg(feature = "impl-default")]
        impl Default for $name {
            #[inline]
            fn default() -> $name { unsafe { $crate::_core::mem::zeroed() } }
        }
    );
}

STRUCT!{ struct DISPATCHER_HEADER {
    Type: u8,
    Absolute: u8,
    Size: u8,
    Inserted: u8,
    SignalState: i32,
    WaitListHead: LIST_ENTRY,
}}

STRUCT!{ struct KEVENT {
    Header: DISPATCHER_HEADER,
}}

pub type PKEVENT = *mut KEVENT;

#[repr(C)]
pub struct FAST_MUTEX {
    pub(crate) Count: LONG,
    pub(crate) Owner: PVOID,
    pub(crate) Contention: ULONG,
    pub(crate) Event: KEVENT,
    pub(crate) OldIrql: ULONG,
}

impl FAST_MUTEX {
    pub const fn new() -> Self {
        Self {
            Count: 0,
            Owner: null_mut(),
            Contention: 9,
            Event: KEVENT {
                Header: DISPATCHER_HEADER {
                    Type: 0,
                    Absolute: 0,
                    Size: 0,
                    Inserted: 0,
                    SignalState: 0,
                    WaitListHead: LIST_ENTRY {
                        Blink: null_mut(),
                        Flink: null_mut(),
                    },
                },
            },
            OldIrql: 0,
        }
    }
}
type PFAST_MUTEX = *mut FAST_MUTEX;

pub unsafe fn ExInitializeFastMutex(Mutex: &mut FAST_MUTEX) {
    Mutex.Count = 1;
    Mutex.Owner = null_mut();
    Mutex.Contention = 0;

    KeInitializeEvent(&mut Mutex.Event as PKEVENT, SynchronizationEvent, FALSE.try_into().unwrap());
}

extern "system" {
    pub fn ExAcquireFastMutex(Mutex: PFAST_MUTEX);
    pub fn ExReleaseFastMutex(Mutex: PFAST_MUTEX);
    pub fn KeInitializeEvent(Mutex: PKEVENT, Type: EVENT_TYPE, State: BOOLEAN);
}
