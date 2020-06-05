// device.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

//! Logical database devices.
//!
//! *e*X*treme*DB uses *logical devices* to describe the database storage.
//!
//! In-memory databases only need one memory device. Both conventional and
//! shared memory devices are supported.
//!
//! Persistent databases require multiple devices for operation:
//! in-memory devices for the in-memory part of the database and the disk page
//! pool, as well as file devices for the database data and log.
//!
//! For more information on the supported devices and their configuration,
//! refer to the *e*X*treme*DB reference pages.
//!
//! # Examples
//!
//! Creating a new in-memory device (the memory region is allocated
//! internally, and released when the device is dropped):
//!
//! ```
//! # use extremedb::{device, Result};
//! #
//! # fn main() -> Result<()> {
//!     let dev = device::Device::new_mem_conv(device::Assignment::Database, 1024 * 1024)?;
//! #
//! #     drop(dev);
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! Creating a new file device:
//!
//! ```
//! # use extremedb::{device, Result};
//! # use std::fs;
//! #
//! # fn main() -> Result<()> {
//!     let db_file = "db.dbs";
//!     let dev = device::Device::new_file(
//!         device::Assignment::Persistent,
//!         device::FileOpenFlags::new(),
//!         db_file,
//!     )?;
//! #
//! #     drop(dev);
//! #
//! #     let _ = fs::remove_file(db_file);
//! #
//! #     Ok(())
//! # }
//! ```

use std::alloc::{self, Layout};
use std::ffi::{c_void, CStr};
use std::mem;
use std::ptr;

use crate::runtime;
use crate::{exdb_sys, mco_ret, Error, Result};

#[doc(hidden)]
pub mod util;

type McoDeviceTypeUnion = exdb_sys::mco_device_t_dev;
type McoDeviceTypeConv = exdb_sys::mco_device_t_dev_conv;
type McoDeviceTypeNamed = exdb_sys::mco_device_t_dev_named;
type McoDeviceTypeFile = exdb_sys::mco_device_t_dev_file;
type McoDeviceTypeMultiFile = exdb_sys::mco_device_t_dev_multifile;
type McoDeviceTypeRaid = exdb_sys::mco_device_t_dev_raid;
// type McoDeviceTypeIDesc = exdb_sys::mco_device_t_dev_idesc;

mod mco_dev_type {
    // pub const MCO_MEMORY_NULL: u32 = 0;
    pub const MCO_MEMORY_CONV: u32 = 1;
    pub const MCO_MEMORY_NAMED: u32 = 2;
    pub const MCO_MEMORY_FILE: u32 = 3;
    pub const MCO_MEMORY_MULTIFILE: u32 = 4;
    pub const MCO_MEMORY_RAID: u32 = 5;
    // pub const MCO_MEMORY_INT_DESC: u32 = 6;
    // pub const MCO_MEMORY_CYCLIC_FILE_BUF: u32 = 7;
}

/// Device assignment.
///
/// The applications use this enumeration to describe the intended purpose
/// of the device.
#[derive(Copy, Clone, Debug)]
pub enum Assignment {
    /// Metadata and database objects, indexes, and other database structures.
    Database,
    /// Disk manager cache (page pool).
    Cache,
    /// A persistent storage device that contains the data.
    Persistent,
    /// A persistent storage device that contains the database log.
    Log,
    // HAAsyncBuf,
    // PipeBuf,
}

impl Assignment {
    fn to_mco(&self) -> u32 {
        match self {
            Assignment::Database => 0,   // MCO_MEMORY_ASSIGN_DATABASE
            Assignment::Cache => 1,      // MCO_MEMORY_ASSIGN_CACHE
            Assignment::Persistent => 2, // MCO_MEMORY_ASSIGN_PERSISTENT
            Assignment::Log => 3,        // MCO_MEMORY_ASSIGN_LOG
                                          // Assignment::HAAsyncBuf => 4, // MCO_MEMORY_ASSIGN_HA_ASYNC_BUF
                                          // Assignment::PipeBuf => 5,    // MCO_MEMORY_ASSIGN_PIPE_BUF
        }
    }
}

macro_rules! flags_builder_method {
    ($(#[$outer:meta])* $method:ident, $ns:path, $flag:expr) => {
        $(#[$outer])*
        pub fn $method(&mut self) -> &mut Self {
            use $ns::*;
            self.0 |= $flag;
            self
        }
    }
}

macro_rules! named_mem_flags_builder_method {
    ($(#[$outer:meta])* $method:ident, $flag:expr) => {
        flags_builder_method!(
            $(#[$outer])*
            $method,
            runtime::options::mco_rt_defines::values::shm_posix,
            $flag
        );
    };
}

/// Shared memory device flags.
///
/// Only used on Posix systems.
pub struct NamedMemFlags(u32);

impl NamedMemFlags {
    /// Creates an empty flags structure. All flags are reset.
    pub fn new() -> Self {
        NamedMemFlags(0)
    }

    named_mem_flags_builder_method!(
        /// Enables anonymous memory mapping.
        anonymous,
        MCO_RT_POSIX_SHM_ANONYMOUS
    );

    named_mem_flags_builder_method!(
        /// Enables shared memory mapping (`MAP_SHARED`).
        shared,
        MCO_RT_POSIX_SHM_SHARED
    );

    named_mem_flags_builder_method!(
        /// Sets the `MAP_HUGETLB` `mmap()` flag (where available).
        huge_tlb,
        MCO_RT_POSIX_SHM_HUGETLB
    );
}

macro_rules! file_open_flags_builder_method {
    ($(#[$outer:meta])* $method:ident, $flag:expr) => {
        flags_builder_method!(
            $(#[$outer])*
            $method,
            exdb_sys::mco_file_open_flags,
            $flag
        );
    };
}

/// Flags for persistent devices.
///
/// These flags allow for fine tuning of file system operations.
/// The default value produced by the [`new()`] method will be suitable
/// for most applications.
///
/// [`new()`]: #method.new
pub struct FileOpenFlags(u32);

impl FileOpenFlags {
    /// The standard, unified, and fail-safe way to open a file in a manner
    /// supported by the target platform file system.
    pub fn new() -> Self {
        FileOpenFlags(exdb_sys::mco_file_open_flags::MCO_FILE_OPEN_DEFAULT as u32)
    }

    file_open_flags_builder_method!(
        /// Opens file in read-only mode.
        read_only,
        MCO_FILE_OPEN_READ_ONLY as u32
    );
    file_open_flags_builder_method!(
        /// Truncates the file after opening.
        truncate,
        MCO_FILE_OPEN_TRUNCATE as u32
    );
    file_open_flags_builder_method!(
        /// Instructs the underlying filesystem to disable caching.
        no_buffering,
        MCO_FILE_OPEN_NO_BUFFERING as u32
    );
    file_open_flags_builder_method!(
        /// Requires the file to exist; a new file is not created.
        existing,
        MCO_FILE_OPEN_EXISTING as u32
    );
    file_open_flags_builder_method!(
        /// Instructs the underlying filesystem to open the file as temporary.
        temporary,
        MCO_FILE_OPEN_TEMPORARY as u32
    );
    file_open_flags_builder_method!(
        /// Enables the flush-operation fix for the *u98* filesystem
        /// wrappers.
        fsync_fix,
        MCO_FILE_OPEN_FSYNC_FIX as u32
    );
    file_open_flags_builder_method!(
        /// Requires the filesystem wrapper to take the RAID device's offset
        /// value into account for all segments of the RAID.
        subpartition,
        MCO_FILE_OPEN_SUBPARTITION as u32
    );
    file_open_flags_builder_method!(
        /// Execute a barrier for flush operations (AIO filesystem wrapper
        /// only).
        fsync_aio_barrier,
        MCO_FILE_OPEN_FSYNC_AIO_BARRIER as u32
    );
    file_open_flags_builder_method!(
        /// Enables file compression (UNIX systems using the
        /// *u98zip*/*u98ziplog* wrappers only).
        compressed,
        MCO_FILE_OPEN_COMPRESSED as u32
    );
    file_open_flags_builder_method!(
        /// Instructs the filesystem wrapper to use `flock()` to setup locking
        /// (*u98* wrapper only).
        lock,
        MCO_FILE_OPEN_LOCK as u32
    );
    file_open_flags_builder_method!(
        /// Use `posix_fadvise()` to control the usage of the pre-read hints
        /// (*u98* wrapper only; use only if it is advised by McObject
        /// support).
        no_read_buffering,
        MCO_FILE_OPEN_NO_READ_BUFFERING as u32
    );
    file_open_flags_builder_method!(
        /// Use `posix_fadvise()` to control the usage of the pre-write hints
        /// (*u98* wrapper only; use only if it is advised by McObject
        /// support).
        no_write_buffering,
        MCO_FILE_OPEN_NO_WRITE_BUFFERING as u32
    );
}

/// A logical device.
#[repr(transparent)]
pub struct Device(exdb_sys::mco_device_t);

impl Device {
    /// Creates a new in-memory device.
    ///
    /// This method allocates `s` bytes of memory using the `std::alloc`
    /// allocator. The memory is freed when the device is dropped.
    pub fn new_mem_conv(a: Assignment, s: usize) -> Result<Self> {
        let l = Layout::from_size_align(s, 1).unwrap();
        let p = unsafe { alloc::alloc(l) };

        if p.is_null() {
            Err(Error::new_core(mco_ret::MCO_E_NOMEM))
        } else {
            Ok(Device(exdb_sys::mco_device_t {
                type_: mco_dev_type::MCO_MEMORY_CONV,
                assignment: a.to_mco(),
                size: s as exdb_sys::mco_size_t,
                dev: McoDeviceTypeUnion {
                    conv: McoDeviceTypeConv {
                        ptr: p as *mut c_void,
                        flags: 0,
                    },
                },
            }))
        }
    }

    /// Creates a new named (shared) memory device.
    ///
    /// The `hint` argument is the address of the shared memory block
    /// allocated for the database. Setting it to zero causes *e*X*treme*DB
    /// to determine the actual shared memory segment address. This could fail
    /// when called from a second process attempting to open the shared
    /// database. In this case it is the application's responsibility
    /// to provide a valid hint address.
    pub fn new_mem_named(
        a: Assignment,
        s: usize,
        name: &str,
        flags: NamedMemFlags,
        hint: usize,
    ) -> Result<Self> {
        let mut named = unsafe { mem::zeroed::<McoDeviceTypeNamed>() };

        if name.len() >= named.name.len() {
            return Err(Error::new_core(mco_ret::MCO_E_ILLEGAL_PARAM));
        }

        unsafe {
            ptr::copy_nonoverlapping(
                name.as_ptr(),
                named.name.as_mut_ptr() as *mut u8,
                name.len(),
            )
        }

        named.flags = flags.0;
        named.hint = hint as *mut c_void;

        Ok(Device(exdb_sys::mco_device_t {
            type_: mco_dev_type::MCO_MEMORY_NAMED,
            assignment: a.to_mco(),
            size: s as exdb_sys::mco_size_t,
            dev: McoDeviceTypeUnion { named },
        }))
    }

    /// Creates a new file device.
    pub fn new_file(a: Assignment, flags: FileOpenFlags, name: &str) -> Result<Self> {
        let mut file = unsafe { mem::zeroed::<McoDeviceTypeFile>() };

        if name.len() >= file.name.len() {
            return Err(Error::new_core(mco_ret::MCO_E_ILLEGAL_PARAM));
        }

        unsafe {
            ptr::copy_nonoverlapping(name.as_ptr(), file.name.as_mut_ptr() as *mut u8, name.len())
        }

        file.flags = flags.0 as i32;

        Ok(Device(exdb_sys::mco_device_t {
            type_: mco_dev_type::MCO_MEMORY_FILE,
            assignment: a.to_mco(),
            size: 0,
            dev: McoDeviceTypeUnion { file },
        }))
    }

    /// Creates a new multi-file device.
    pub fn new_multifile(
        a: Assignment,
        flags: FileOpenFlags,
        name: &str,
        segment_size: isize,
    ) -> Result<Self> {
        let mut multifile = unsafe { mem::zeroed::<McoDeviceTypeMultiFile>() };

        if name.len() >= multifile.name.len() {
            return Err(Error::new_core(mco_ret::MCO_E_ILLEGAL_PARAM));
        }

        unsafe {
            ptr::copy_nonoverlapping(
                name.as_ptr(),
                multifile.name.as_mut_ptr() as *mut u8,
                name.len(),
            )
        }

        multifile.flags = flags.0 as i32;
        multifile.segment_size = segment_size as exdb_sys::mco_offs_t;

        Ok(Device(exdb_sys::mco_device_t {
            type_: mco_dev_type::MCO_MEMORY_MULTIFILE,
            assignment: a.to_mco(),
            size: 0,
            dev: McoDeviceTypeUnion { multifile },
        }))
    }

    /// Creates a new RAID device.
    pub fn new_raid(
        a: Assignment,
        flags: FileOpenFlags,
        name: &str,
        level: i32,
        offset: isize,
    ) -> Result<Self> {
        let mut raid = unsafe { mem::zeroed::<McoDeviceTypeRaid>() };

        if name.len() >= raid.name.len() {
            return Err(Error::new_core(mco_ret::MCO_E_ILLEGAL_PARAM));
        }

        unsafe {
            ptr::copy_nonoverlapping(name.as_ptr(), raid.name.as_mut_ptr() as *mut u8, name.len())
        }

        raid.flags = flags.0 as i32;
        raid.level = level;
        raid.offset = offset as exdb_sys::mco_offs_t;

        Ok(Device(exdb_sys::mco_device_t {
            type_: mco_dev_type::MCO_MEMORY_RAID,
            assignment: a.to_mco(),
            size: 0,
            dev: McoDeviceTypeUnion { raid },
        }))
    }

    fn file_name(&self) -> Option<&str> {
        match self.0.type_ {
            mco_dev_type::MCO_MEMORY_FILE => {
                let cname = unsafe { CStr::from_ptr(self.0.dev.file.name.as_ptr()) };
                cname.to_str().ok()
            }
            _ => None,
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        if self.0.type_ == mco_dev_type::MCO_MEMORY_CONV {
            let l = Layout::from_size_align(self.0.size as usize, 1).unwrap();
            unsafe { alloc::dealloc(self.0.dev.conv.ptr as *mut u8, l) };
        }
    }
}
