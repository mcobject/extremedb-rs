// runtime.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

//! *e*X*treme*DB runtime management.
//!
//! The *e*X*treme*DB runtime must be explicitly started before any database
//! operations can be done. The underlying implementation does not allow
//! the runtime to be stopped and restarted; hence, dropping the Runtime
//! object and starting it again will panic.
//!
//! # Examples
//!
//! Starting a runtime with default options:
//!
//! ```
//! # use extremedb::runtime::Runtime;
//! let runtime = Runtime::start(vec![]);
//! // Runtime is now started, and can be used to create a database.
//! ```
//!
//! See the [`options`] module for more information on runtime options.
//!
//! [`options`]: ./options/index.html

use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{exdb_sys, mco_ret};

/// Runtime option definitions.
///
/// These options can only be set when the runtime is started.
///
/// # Example
///
/// The example below enables the `SHM_HUGETLB` setting on Posix systems
/// and sets the timestamp resolution to 1 millisecond.
///
/// ```
/// # use extremedb::runtime::{
/// #     options::DateTimePrecision, options::Opt, options::PosixSharedMemoryOptions, Runtime,
/// # };
/// #
/// # fn main() {
///     let mut shmem_opt = PosixSharedMemoryOptions::new();
///     shmem_opt.huge_tlb();
///
///     let runtime = Runtime::start(vec![
///         Opt::DateTimePrecision(DateTimePrecision::milliseconds(1)),
///         Opt::PosixSharedMemory(shmem_opt),
///     ]);
///
/// #     drop(runtime);
/// # }
/// ```
pub mod options {
    // Option names and values definitions.
    // The constants in this module correspond to the runtime macros
    // defined in the mco.h header.
    pub(crate) mod mco_rt_defines {
        pub(crate) mod keys {
            pub const MCO_RT_WINDOWS_SHM_OPT: u32 = 1;
            pub const MCO_RT_OPTION_MARK_LAST_OBJ: u32 = 2;
            pub const MCO_RT_OPTION_UNIX_SHM_MASK: u32 = 3;
            pub const MCO_RT_POSIX_SHM_OPT: u32 = 4;
            pub const MCO_RT_CRC_ALGORITHM: u32 = 5;
            pub const MCO_RT_MAX_PATRICIA_DEPTH: u32 = 6;
            pub const MCO_RT_MAX_DYNAMIC_PIPES: u32 = 7;
            pub const MCO_RT_OPTION_CLASS_LOAD_MERGE: u32 = 8;
            pub const MCO_RT_OPTION_DATETIME_PRECISION: u32 = 9;
            // pub const MCO_RT_OPTION_LAST: u32 = 10;
        }

        pub(crate) mod values {
            pub(crate) mod shm_posix {
                pub const MCO_RT_POSIX_SHM_ANONYMOUS: u32 = 1;
                pub const MCO_RT_POSIX_SHM_SHARED: u32 = 2;
                pub const MCO_RT_POSIX_SHM_HUGETLB: u32 = 4;
            }

            pub(crate) mod shm_windows {
                pub const MCO_RT_WINDOWS_SHM_PREFIX_GLOBAL: u32 = 0;
                pub const MCO_RT_WINDOWS_SHM_PREFIX_LOCAL: u32 = 1;
                pub const MCO_RT_WINDOWS_SHM_PREFIX_SESSIONS: u32 = 2;
                pub const MCO_RT_WINDOWS_SHM_PREFIX_NONE: u32 = 3;
                // pub const MCO_RT_WINDOWS_SHM_PREFIX_MASK: u32 = 3;

                pub const MCO_RT_WINDOWS_SHM_SEC_DESCR_EMPTY: u32 = 0;
                pub const MCO_RT_WINDOWS_SHM_SEC_DESCR_NULL: u32 = 4;
                pub const MCO_RT_WINDOWS_SHM_SEC_DESCR_SAMEUSER: u32 = 8;
                // pub const MCO_RT_WINDOWS_SHM_SEC_DESCR_MASK: u32 = 12;
            }

            pub(crate) mod crc {
                pub const MCO_RT_CRC32_NONE: u32 = 0;
                pub const MCO_RT_CRC32: u32 = 1;
                pub const MCO_RT_CRC32_FAST: u32 = 2;
                pub const MCO_RT_CRC32_OLD: u32 = 3;
                pub const MCO_RT_CRC32C: u32 = 4;
            }
        }
    }

    /// Shared memory prefix (Windows only).
    ///
    /// The default value is `None`.
    pub enum WinSharedMemoryPrefix {
        /// No prefix.
        None = mco_rt_defines::values::shm_windows::MCO_RT_WINDOWS_SHM_PREFIX_NONE as isize,
        /// Use Global prefix. Shared objects are visible in all sessions.
        Global = mco_rt_defines::values::shm_windows::MCO_RT_WINDOWS_SHM_PREFIX_GLOBAL as isize,
        /// Use Local prefix. Shared objects are visible in the current session
        /// only.
        Local = mco_rt_defines::values::shm_windows::MCO_RT_WINDOWS_SHM_PREFIX_LOCAL as isize,
        /// Vista sessions fix.
        Sessions = mco_rt_defines::values::shm_windows::MCO_RT_WINDOWS_SHM_PREFIX_SESSIONS as isize,
    }

    /// Shared memory security descriptor (Windows only).
    ///
    /// The default value is `Null`.
    pub enum WinSharedMemoryDescr {
        /// Null descriptor; enables default system policy:
        /// the database will be accessible from the current session only.
        Null = mco_rt_defines::values::shm_windows::MCO_RT_WINDOWS_SHM_SEC_DESCR_NULL as isize,
        /// Empty security descriptor; allows database access for all users
        /// and sessions.
        Empty = mco_rt_defines::values::shm_windows::MCO_RT_WINDOWS_SHM_SEC_DESCR_EMPTY as isize,
        /// Allow access for the current user from all sessions.
        SameUser =
            mco_rt_defines::values::shm_windows::MCO_RT_WINDOWS_SHM_SEC_DESCR_SAMEUSER as isize,
    }

    /// Shared memory options (Posix systems only).
    ///
    /// By default, *e*X*treme*DB runtime uses the "anonymous" and "shared"
    /// flags.
    pub struct PosixSharedMemoryOptions(u32);

    impl PosixSharedMemoryOptions {
        /// Creates an empty options structure. All options are reset.
        pub fn new() -> Self {
            PosixSharedMemoryOptions(0)
        }

        /// Enables anonymous memory mapping.
        pub fn anonymous(&mut self) -> &mut Self {
            self.0 |= mco_rt_defines::values::shm_posix::MCO_RT_POSIX_SHM_ANONYMOUS;
            self
        }

        /// Enables shared memory mapping (`MAP_SHARED`).
        /// Private mapping (`MAP_PRIVATE`) is used otherwise.
        pub fn shared(&mut self) -> &mut Self {
            self.0 |= mco_rt_defines::values::shm_posix::MCO_RT_POSIX_SHM_SHARED;
            self
        }

        /// Sets the `MAP_HUGETLB` `mmap()` flag (where available).
        pub fn huge_tlb(&mut self) -> &mut Self {
            self.0 |= mco_rt_defines::values::shm_posix::MCO_RT_POSIX_SHM_HUGETLB;
            self
        }
    }

    /// CRC32 algorithm used to verify persistent database consistency.
    pub enum CRCAlgorithm {
        /// Do not compute CRC.
        None = mco_rt_defines::values::crc::MCO_RT_CRC32_NONE as isize,
        /// The CRC implementation used prior to release 15523 (2014-07-17).
        CRC32 = mco_rt_defines::values::crc::MCO_RT_CRC32 as isize,
        /// Compute the CRC only for the first and last 8-byte word of data.
        CRC32Fast = mco_rt_defines::values::crc::MCO_RT_CRC32_FAST as isize,
        /// The CRC implementation used prior to release 12823 (2013-01-28).
        CRC32Old = mco_rt_defines::values::crc::MCO_RT_CRC32_OLD as isize,
        /// CRC32C hardware & software implementations used after release
        /// 15523 (2014-07-17).
        CRC32C = mco_rt_defines::values::crc::MCO_RT_CRC32C as isize,
    }

    /// Time resolution (precision) for `datetime` values.
    ///
    /// The default value is 1 (one second).
    pub struct DateTimePrecision(u32);

    impl DateTimePrecision {
        /// Sets the `datetime` precision in seconds.
        pub fn seconds(val: u32) -> Self {
            Self(val)
        }

        /// Sets the `datetime` precision in milliseconds.
        pub fn milliseconds(val: u32) -> Self {
            Self(val.checked_mul(1_000).expect("value overflow"))
        }

        /// Sets the `datetime` precision in microseconds.
        pub fn microseconds(val: u32) -> Self {
            Self(val.checked_mul(1_000_000).expect("value overflow"))
        }
    }

    /// A runtime option.
    ///
    /// The options can only be set when the runtime is started.
    pub enum Opt {
        /// Shared memory options, Windows only.
        WinSharedMemory((WinSharedMemoryPrefix, WinSharedMemoryDescr)),
        /// Require the "end-of-transaction" notification (used with
        /// *e*X*treme*DB Transaction Logging).
        MarkLastObject(bool),
        /// Shared memory access mask, Unix only.
        UnixSharedMemoryAccessMask(u32),
        /// Shared memory options, Posix only.
        PosixSharedMemory(PosixSharedMemoryOptions),
        /// CRC algorithm used to verify the integrity of persistent databases.
        CRCAlgorithm(CRCAlgorithm),
        /// Maximum depth of a Patricia trie index before the
        /// `MCO_E_PATRICIA_TOO_DEEP` error is thrown.
        MaxPatriciaDepth(u32),
        /// Maximum number of the `PIPE_BUF` devices that can be added.
        MaxDynamicPipes(u32),
        /// Enable merging of loaded objects with the objects already existing
        /// in the database.
        ClassLoadMerge(bool),
        /// Resolution of the `datetime` values.
        DateTimePrecision(DateTimePrecision),
    }

    impl Opt {
        pub(super) fn raw(self) -> (i32, i32) {
            use mco_rt_defines::keys::*;

            match self {
                Opt::WinSharedMemory((p, d)) => {
                    (MCO_RT_WINDOWS_SHM_OPT as i32, (p as i32 | d as i32))
                }
                Opt::MarkLastObject(v) => (MCO_RT_OPTION_MARK_LAST_OBJ as i32, v as i32),
                Opt::UnixSharedMemoryAccessMask(v) => {
                    (MCO_RT_OPTION_UNIX_SHM_MASK as i32, v as i32)
                }
                Opt::PosixSharedMemory(v) => (MCO_RT_POSIX_SHM_OPT as i32, v.0 as i32),
                Opt::CRCAlgorithm(v) => (MCO_RT_CRC_ALGORITHM as i32, v as i32),
                Opt::MaxPatriciaDepth(v) => (MCO_RT_MAX_PATRICIA_DEPTH as i32, v as i32),
                Opt::MaxDynamicPipes(v) => (MCO_RT_MAX_DYNAMIC_PIPES as i32, v as i32),
                Opt::ClassLoadMerge(v) => (MCO_RT_OPTION_CLASS_LOAD_MERGE as i32, v as i32),
                Opt::DateTimePrecision(v) => (MCO_RT_OPTION_DATETIME_PRECISION as i32, v.0 as i32),
            }
        }
    }
}

/// Runtime information.
///
/// This structure can be used by the applications to get the configuration
/// of the runtime: *e*X*treme*DB version, data types and features supported,
/// policies, capabilities, etc.
///
/// The methods of this structure correspond to the fields of the
/// `mco_runtime_info_t` structure declared in *mco.h*.
/// Refer to the *e*X*treme*DB documentation for information on individual
/// parameters.
///
/// # Examples
///
/// ```
/// # use extremedb::runtime::Runtime;
/// #
/// # fn main() {
///     let runtime = Runtime::start(vec![]);
///     let caps = runtime.info();
///
///     println!(
///         "eXtremeDB version {}.{}.{}",
///         caps.mco_version_major(),
///         caps.mco_version_minor(),
///         caps.mco_build_number()
///     );
/// # }
/// ```
pub struct Info {
    caps: exdb_sys::mco_runtime_info_t,
}

impl Info {
    fn new(caps: exdb_sys::mco_runtime_info_t) -> Self {
        Info { caps }
    }

    pub fn mco_version_major(&self) -> u8 {
        self.caps.mco_version_major
    }

    pub fn mco_version_minor(&self) -> u8 {
        self.caps.mco_version_minor
    }

    pub fn mco_build_number(&self) -> u16 {
        self.caps.mco_build_number
    }

    pub fn mco_size_t(&self) -> usize {
        self.caps.mco_size_t as usize
    }

    pub fn mco_offs_t(&self) -> usize {
        self.caps.mco_offs_t as usize
    }

    pub fn uint4_supported(&self) -> bool {
        self.caps.uint4_supported != 0
    }

    pub fn float_supported(&self) -> bool {
        self.caps.float_supported != 0
    }

    pub fn mco_checklevel(&self) -> u8 {
        self.caps.mco_checklevel
    }

    pub fn evaluation_version(&self) -> bool {
        self.caps.evaluation_version != 0
    }

    pub fn large_database_supported(&self) -> bool {
        self.caps.large_database_supported != 0
    }

    pub fn collation_supported(&self) -> bool {
        self.caps.collation_supported != 0
    }

    pub fn heap31_supported(&self) -> bool {
        self.caps.heap31_supported != 0
    }

    pub fn bin_serialization_supported(&self) -> bool {
        self.caps.bin_serialization_supported != 0
    }

    pub fn fixedrec_supported(&self) -> bool {
        self.caps.fixedrec_supported != 0
    }

    pub fn statistics_supported(&self) -> bool {
        self.caps.statistics_supported != 0
    }

    pub fn events_supported(&self) -> bool {
        self.caps.events_supported != 0
    }

    pub fn save_load_supported(&self) -> bool {
        self.caps.save_load_supported != 0
    }

    pub fn object_initialization_supported(&self) -> bool {
        self.caps.object_initialization_supported != 0
    }

    pub fn direct_index_field_access_supported(&self) -> bool {
        self.caps.direct_index_field_access_supported != 0
    }

    pub fn multiprocess_access_supported(&self) -> bool {
        self.caps.multiprocess_access_supported != 0
    }

    pub fn object_repack_supported(&self) -> bool {
        self.caps.object_repack_supported != 0
    }

    pub fn transaction_logging_supported(&self) -> bool {
        self.caps.transaction_logging_supported != 0
    }

    pub fn cluster_supported(&self) -> bool {
        self.caps.cluster_supported != 0
    }

    pub fn high_availability_supported(&self) -> bool {
        self.caps.high_availability_supported != 0
    }

    pub fn iot_supported(&self) -> bool {
        self.caps.iot_supported != 0
    }

    pub fn ha_multicast_supported(&self) -> bool {
        self.caps.ha_multicast_supported != 0
    }

    pub fn ha_incremental_replication_supported(&self) -> bool {
        self.caps.ha_incremental_replication_supported != 0
    }

    pub fn binary_schema_evolution_supported(&self) -> bool {
        self.caps.binary_schema_evalution_supported != 0
    }

    pub fn unicode_supported(&self) -> bool {
        self.caps.unicode_supported != 0
    }

    pub fn wchar_supported(&self) -> bool {
        self.caps.wchar_supported != 0
    }

    pub fn recovery_supported(&self) -> bool {
        self.caps.recovery_supported != 0
    }

    pub fn disk_supported(&self) -> bool {
        self.caps.disk_supported != 0
    }

    pub fn direct_pointers_supported(&self) -> bool {
        self.caps.direct_pointers_supported != 0
    }

    pub fn persistent_object_supported(&self) -> bool {
        self.caps.persistent_object_supported != 0
    }

    pub fn xml_import_export_supported(&self) -> bool {
        self.caps.xml_import_export_supported != 0
    }

    pub fn user_defined_index_supported(&self) -> bool {
        self.caps.user_defined_index_supported != 0
    }

    pub fn multifile_supported(&self) -> bool {
        self.caps.multifile_supported != 0
    }

    pub fn multifile_descriptor_supported(&self) -> bool {
        self.caps.multifile_descriptor_supported != 0
    }

    pub fn two_phase_commit_supported(&self) -> bool {
        self.caps.two_phase_commit_supported != 0
    }

    pub fn rtree_supported(&self) -> bool {
        self.caps.rtree_supported != 0
    }

    pub fn tree_based_hash(&self) -> bool {
        self.caps.tree_based_hash != 0
    }

    pub fn tmgr_mvcc_async_cleanup(&self) -> bool {
        self.caps.tmgr_mvcc_async_cleanup != 0
    }

    pub fn concurent_disk_btree(&self) -> bool {
        self.caps.concurent_disk_btree != 0
    }

    pub fn open_cursor_goto_first(&self) -> bool {
        self.caps.open_cursor_goto_first != 0
    }

    pub fn smart_index_insert(&self) -> bool {
        self.caps.smart_index_insert != 0
    }

    pub fn btree_leaf_lock(&self) -> bool {
        self.caps.btree_leaf_lock != 0
    }

    pub fn null_statistics(&self) -> bool {
        self.caps.null_statistics != 0
    }

    pub fn implicit_runtime_start(&self) -> bool {
        self.caps.implicit_runtime_start != 0
    }

    pub fn bufferized_sync_iostream(&self) -> bool {
        self.caps.bufferized_sync_iostream != 0
    }

    pub fn async_replication(&self) -> bool {
        self.caps.async_replication != 0
    }

    pub fn fast_transaction_list(&self) -> bool {
        self.caps.fast_transaction_list != 0
    }

    pub fn extendable_dirty_page_bitmap(&self) -> bool {
        self.caps.extendable_dirty_page_bitmap != 0
    }

    pub fn mursiw_policy(&self) -> u8 {
        self.caps.mursiw_policy
    }

    pub fn sync_capabilities(&self) -> u8 {
        self.caps.sync_capabilities
    }

    pub fn char_comparison_policy(&self) -> u8 {
        self.caps.char_comparison_policy
    }

    pub fn stream_buffer_size(&self) -> usize {
        self.caps.stream_buffer_size as usize
    }

    pub fn max_db_instances(&self) -> usize {
        self.caps.max_db_instances as usize
    }

    pub fn max_db_name_length(&self) -> usize {
        self.caps.max_db_name_length as usize
    }

    pub fn max_extends(&self) -> isize {
        self.caps.max_extends as isize
    }

    pub fn tl_page_buffer_size(&self) -> usize {
        self.caps.tl_page_buffer_size as usize
    }

    pub fn ha_max_replicas(&self) -> usize {
        self.caps.ha_max_replicas as usize
    }

    pub fn ha_transmit_buffer_size(&self) -> usize {
        self.caps.ha_transmit_buffer_size as usize
    }

    pub fn ha_syncronization_buffer_size(&self) -> usize {
        self.caps.ha_syncronization_buffer_size as usize
    }

    pub fn default_redo_log_limit(&self) -> usize {
        self.caps.default_redo_log_limit as usize
    }

    pub fn mvcc_critical_sections(&self) -> usize {
        self.caps.mvcc_critical_sections as usize
    }

    pub fn mvcc_per_index_locks(&self) -> usize {
        self.caps.mvcc_per_index_locks as usize
    }

    pub fn con_disk_page_cache_size(&self) -> usize {
        self.caps.con_disk_page_cache_size as usize
    }

    pub fn small_con_cache_threshold(&self) -> usize {
        self.caps.small_con_cache_threshold as usize
    }

    pub fn extendable_dirty_page_bitmap_limit(&self) -> usize {
        self.caps.extendable_dirty_page_bitmap_limit as usize
    }

    pub fn max_vista_sessions(&self) -> usize {
        self.caps.max_vista_sessions as usize
    }

    pub fn concurrent_write_transactions(&self) -> bool {
        self.caps.concurrent_write_transactions != 0
    }

    pub fn encryption_support(&self) -> bool {
        self.caps.encryption_support != 0
    }

    pub fn backup_support(&self) -> bool {
        self.caps.backup_support != 0
    }

    pub fn mco_revision(&self) -> &'static str {
        let ret = unsafe { CStr::from_ptr(self.caps.mco_revision).to_str() };
        match ret {
            Ok(s) => s,
            Err(_) => "",
        }
    }
}

/// Runtime interface.
///
/// The *e*X*treme*DB runtime must be explicitly started before any database
/// operations are possible.
///
/// The runtime can be started only once per process. Dropping the runtime
/// structure and trying to start it again will panic.
///
/// # Example
///
/// In many cases the runtime can be started without any options:
///
/// ```
/// use extremedb::runtime::Runtime;
///
/// fn main() {
///     let runtime = Runtime::start(vec![]);
///     // ...
/// #
/// #     drop(runtime);
/// }
/// ```
///
/// For more information on the runtime options, refer to the [`options`]
/// module page.
///
/// [`options`]: ./options/index.html
pub struct Runtime {}

impl Runtime {
    /// Starts the *e*X*treme*DB runtime.
    ///
    /// The `opts` array may contain additional options if required.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    ///
    /// * The *e*X*treme*DB runtime fails to start.
    /// * Called more than once: dropping and restarting the runtime is
    /// currently forbidden.
    pub fn start(opts: Vec<options::Opt>) -> Self {
        static mut RUNTIME_STARTED: AtomicBool = AtomicBool::new(false);

        unsafe {
            if !RUNTIME_STARTED.fetch_or(true, Ordering::SeqCst) {
                let rc = exdb_sys::mco_runtime_start();
                if rc != mco_ret::MCO_S_OK {
                    panic!("failed to start runtime: error {}", rc)
                }
            } else {
                panic!("runtime has already been started");
            }
        }

        Runtime::apply_options(opts);

        Runtime {}
    }

    /// Returns the information about the active runtime.
    pub fn info(&self) -> Info {
        Runtime::info_impl()
    }

    fn apply_options(opts: Vec<options::Opt>) {
        for opt in opts {
            let (o, v) = opt.raw();
            unsafe {
                exdb_sys::mco_runtime_setoption(o, v);
            }
        }
    }

    pub(crate) fn info_impl() -> Info {
        let mut caps = MaybeUninit::uninit();
        unsafe { exdb_sys::mco_get_runtime_info(caps.as_mut_ptr()) };
        Info::new(unsafe { caps.assume_init() })
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        // Do not reset the "runtime started" flag: prevent it from being
        // started again in this instance, ever.
        let rc = unsafe { exdb_sys::mco_runtime_stop() };
        debug_assert_eq!(mco_ret::MCO_S_OK, rc);
    }
}
