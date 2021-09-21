// database.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

//! Core database types.
//!
//! For detailed information on database configuration, refer to
//! the *e*X*treme*DB reference pages.
//!
//! # Examples
//!
//! Creating an in-memory database using conventional memory:
//!
//! ```
//! use extremedb::{database, device, runtime, Result};
//!
//! fn main() -> Result<()> {
//!     let runtime = runtime::Runtime::start(vec![]);
//!
//! #     if runtime.info().multiprocess_access_supported() {
//! #         return Ok(());
//! #     }
//!     // This example creates a conventional memory device, and will not work with the
//!     // shared memory runtime.
//!     assert!(!runtime.info().multiprocess_access_supported());
//!
//!     // Default parameters.
//!     let db_params = database::Params::new();
//!
//!     // Create an in-memory device and allocate memory.
//!     let mut devs = vec![device::Device::new_mem_conv(
//!         device::Assignment::Database,
//!         1024 * 1024,
//!     )?];
//!
//!     // Open the database.
//!     let db = database::Database::open(&runtime, "test_db", None, &mut devs, db_params)?;
//!
//!     // ...
//!
//! #     drop(db);
//!     Ok(())
//! }
//! ```
//!
//! Creating a persistent database (note that `extremedb_sys` must be built
//! with persistent database support — refer to its documentation pages for
//! the details):
//!
//! ```
//! use extremedb::{database, device, runtime, Result};
//! # use std::fs;
//!
//! fn main() -> Result<()> {
//!     let runtime = runtime::Runtime::start(vec![]);
//! #     // Do not run this example if built without disk support, or with shared memory.
//! #     if !runtime.info().disk_supported() || runtime.info().multiprocess_access_supported() {
//! #         return Ok(());
//! #     }
//!
//!     // Make sure that persistent database support is enabled.
//!     assert!(runtime.info().disk_supported());
//!
//!     // This example creates conventional memory devices, and will not work with the
//!     // shared memory runtime.
//!     assert!(!runtime.info().multiprocess_access_supported());
//!
//!     // Default parameters.
//!     let db_params = database::Params::new();
//!
//!     // Database and log file names.
//!     let db_file = "db.dbs";
//!     let log_file = "db.log";
//!
//!     // Create an array of devices for the persistent database.
//!     let mut devs = vec![
//!         // In-memory portion of the database.
//!         device::Device::new_mem_conv(device::Assignment::Database, 1024 * 1024)?,
//!         // Disk page pool.
//!         device::Device::new_mem_conv(device::Assignment::Cache, 1024 * 1024)?,
//!         // Database file.
//!         device::Device::new_file(
//!             device::Assignment::Persistent,
//!             device::FileOpenFlags::new(),
//!             db_file,
//!         )?,
//!         // Database log file.
//!         device::Device::new_file(
//!             device::Assignment::Log,
//!             device::FileOpenFlags::new(),
//!             log_file,
//!         )?,
//!     ];
//!
//!     // Open the database.
//!     let db = database::Database::open(&runtime, "test_db", None, &mut devs, db_params)?;
//!
//!     // ...
//! #    drop(db);
//! #    drop(runtime);
//! #     let _ = fs::remove_file(db_file);
//! #     let _ = fs::remove_file(log_file);
//!
//!     Ok(())
//! }
//! ```

use std::ffi::{c_void, CStr, CString};
use std::marker::PhantomData;
use std::mem::{self, MaybeUninit};
use std::ptr;

use crate::device::Device;
use crate::dict;
use crate::runtime::Runtime;
use crate::util::BitMask32;
use crate::{exdb_sys, mco_ret, result_from_code, Error, Result};

use exdb_sys::MCO_COMMIT_POLICY_E as mco_commit_policy;
use exdb_sys::MCO_COMPRESSION_MASK_ as mco_compression_mask;
use exdb_sys::MCO_DB_MODE_MASK_ as mco_db_mode_mask;
use exdb_sys::MCO_LOG_TYPE_ as mco_log_type;
use exdb_sys::MCO_TRANS_SCHED_POLICY_E_ as mco_trans_sched_policy;

macro_rules! db_param_scalar {
    ($(#[$p_meta:meta])* $param:ident, $(#[$g_meta:meta])* $getter:ident, $ty:ty, $mco_ty:ty) => {
        $(#[$p_meta])*
        pub fn $param(&mut self, $param: $ty) -> &mut Self {
            self.p.$param = $param as $mco_ty;
            self
        }

        $(#[$g_meta])*
        pub fn $getter(&self) -> $ty {
            self.p.$param as $ty
        }
    };

    ($(#[$p_meta:meta])* $param:ident, $(#[$g_meta:meta])* $getter:ident, $ty:ty) => {
        db_param_scalar!($(#[$p_meta])* $param, $(#[$g_meta])* $getter, $ty, $ty);
    };
}

/// Database log type.
pub enum LogType {
    /// No log.
    None = mco_log_type::NO_LOG as isize,
    /// Redo log.
    Redo = mco_log_type::REDO_LOG as isize,
    /// Undo log.
    Undo = mco_log_type::UNDO_LOG as isize,
}

impl LogType {
    fn from_mco(t: mco_log_type::Type) -> Option<Self> {
        match t {
            mco_log_type::NO_LOG => Some(LogType::None),
            mco_log_type::REDO_LOG => Some(LogType::Redo),
            mco_log_type::UNDO_LOG => Some(LogType::Undo),
            _ => None,
        }
    }
}

/// Transaction commit policy.
///
/// The default policy is `SyncFlush`.
pub enum CommitPolicy {
    /// Flush the cache and synchronize the filesystem buffers for both
    /// database and log files on every commit.
    SyncFlush = mco_commit_policy::MCO_COMMIT_SYNC_FLUSH as isize,

    /// Don't flush the database cache to disk upon transaction commit.
    Buffered = mco_commit_policy::MCO_COMMIT_BUFFERED as isize,

    /// Keep modified pages in the page pool (like `Buffered`); commit delayed
    /// transactions to the persistent storage at once when the total size of
    /// all kept pages or the number of delayed commits reach a threshold
    /// value.
    Delayed = mco_commit_policy::MCO_COMMIT_DELAYED as isize,

    /// Don't synchronize the file system buffers explicitly and let the
    /// filesystem determine when the data is actually written to the media.
    NoSync = mco_commit_policy::MCO_COMMIT_NO_SYNC as isize,
}

impl CommitPolicy {
    fn from_mco(p: mco_commit_policy::Type) -> Option<Self> {
        match p {
            mco_commit_policy::MCO_COMMIT_SYNC_FLUSH => Some(CommitPolicy::SyncFlush),
            mco_commit_policy::MCO_COMMIT_BUFFERED => Some(CommitPolicy::Buffered),
            mco_commit_policy::MCO_COMMIT_DELAYED => Some(CommitPolicy::Delayed),
            mco_commit_policy::MCO_COMMIT_NO_SYNC => Some(CommitPolicy::NoSync),
            _ => None,
        }
    }
}

/// Transaction scheduling policy.
///
/// The default policy is `FIFO`.
pub enum TransSchedPolicy {
    /// First in, first out scheduling.
    FIFO = mco_trans_sched_policy::MCO_SCHED_FIFO as isize,

    /// Prioritize read-only transactions.
    ReaderFavor = mco_trans_sched_policy::MCO_SCHED_READER_FAVOR as isize,

    /// Prioritize read-write transactions.
    WriterFavor = mco_trans_sched_policy::MCO_SCHED_WRITER_FAVOR as isize,
}

impl TransSchedPolicy {
    fn from_mco(p: mco_trans_sched_policy::Type) -> Option<Self> {
        match p {
            mco_trans_sched_policy::MCO_SCHED_FIFO => Some(TransSchedPolicy::FIFO),
            mco_trans_sched_policy::MCO_SCHED_READER_FAVOR => Some(TransSchedPolicy::ReaderFavor),
            mco_trans_sched_policy::MCO_SCHED_WRITER_FAVOR => Some(TransSchedPolicy::WriterFavor),
            _ => None,
        }
    }
}

/// Database log parameters.
///
/// # Examples
///
/// Configuring the `Delayed` commit policy:
///
/// ```
/// # use extremedb::database::{LogParams, CommitPolicy};
/// let mut params = LogParams::new();
/// params.default_commit_policy(CommitPolicy::Delayed).max_delayed_transactions(10);
/// ```
pub struct LogParams {
    p: exdb_sys::mco_log_params_t,
}

impl LogParams {
    /// Creates a new log parameters structure initialized with default values.
    pub fn new() -> Self {
        LogParams {
            p: exdb_sys::mco_log_params_t {
                default_commit_policy: mco_commit_policy::MCO_COMMIT_SYNC_FLUSH,
                redo_log_limit: 16 * 1024 * 1024,
                delayed_commit_threshold: 0,
                max_delayed_transactions: 0,
                max_commit_delay: 0,
            },
        }
    }

    fn with_params(p: &exdb_sys::mco_log_params_t) -> Self {
        LogParams { p: *p }
    }

    /// Sets the default commit policy.
    pub fn default_commit_policy(&mut self, default_commit_policy: CommitPolicy) -> &mut Self {
        self.p.default_commit_policy = default_commit_policy as mco_commit_policy::Type;
        self
    }

    /// Returns the default commit policy.
    ///
    /// Returns `None` if the policy value is invalid.
    pub fn get_default_commit_policy(&self) -> Option<CommitPolicy> {
        CommitPolicy::from_mco(self.p.default_commit_policy)
    }

    db_param_scalar!(
        /// Sets the log file size limit.
        redo_log_limit,
        /// Returns the log file size limit.
        get_redo_log_limit,
        usize,
        exdb_sys::mco_offs_t
    );

    db_param_scalar!(
        /// Sets the delayed commit threshold.
        ///
        /// When the total size of all pinned pages reaches this threshold,
        /// all delayed transactions are committed to the persistent storage
        /// at once.
        ///
        /// This option is only used with [`CommitPolicy::Delayed`].
        ///
        /// [`CommitPolicy::Delayed`]: ./enum.CommitPolicy.html#variant.Delayed
        delayed_commit_threshold,
        /// Returns the delayed commit threshold.
        get_delayed_commit_threshold,
        usize,
        exdb_sys::mco_offs_t
    );

    db_param_scalar!(
        /// Sets the maximum number of delayed transactions.
        ///
        /// Delayed transactions are committed to the persistent storage
        /// when their number reaches this threshold.
        ///
        /// This option is only used with [`CommitPolicy::Delayed`].
        ///
        /// [`CommitPolicy::Delayed`]: ./enum.CommitPolicy.html#variant.Delayed
        max_delayed_transactions,
        /// Returns the maximum number of delayed transactions.
        get_max_delayed_transactions,
        u32,
        exdb_sys::mco_counter32_t
    );

    db_param_scalar!(
        /// Sets the maximum commit delay.
        ///
        /// This is the number of application-defined clock cycles to wait
        /// before all delayed transactions are committed to the persistent
        /// storage at once.
        ///
        /// This option is only used with [`CommitPolicy::Delayed`].
        ///
        /// [`CommitPolicy::Delayed`]: ./enum.CommitPolicy.html#variant.Delayed
        max_commit_delay,
        /// Returns the maximum commit delay.
        get_max_commit_delay,
        u32
    );
}

macro_rules! bitmask_flag {
    ($(#[$f_meta:meta])* $flag:ident, $(#[$g_meta:meta])* $getter:ident, $f:expr) => {
        $(#[$f_meta])*
        pub fn $flag(&mut self, v: bool) -> &mut Self {
            self.0.set_bit_value($f, v);
            self
        }

        $(#[$g_meta])*
        pub fn $getter(&self) -> bool {
            self.0.get_bit_value($f)
        }
    };
}

/// A mask of page classes for compression.
///
/// This structure is only used when the in-memory database compression
/// is enabled.
///
/// # Examples
///
/// Enable compression of object and blob header pages:
///
/// ```
/// # use extremedb::database::CompressionMask;
/// let mut mask = CompressionMask::new();
/// mask.obj_head(true).blob_head(true);
/// ```
pub struct CompressionMask(BitMask32);

impl CompressionMask {
    /// Creates a new compression mask filled with zeroes.
    pub fn new() -> Self {
        CompressionMask(BitMask32::new())
    }

    fn from_mco(m: u32) -> Self {
        CompressionMask(BitMask32::from(m))
    }

    bitmask_flag!(
        /// Enables compression of object header pages.
        obj_head,
        /// Returns the current flag value.
        get_obj_head,
        mco_compression_mask::MCO_COMPRESSION_OBJ_HEAD as u32
    );
    bitmask_flag!(
        /// Enables compression of object node pages.
        obj_node,
        /// Returns the current flag value.
        get_obj_node,
        mco_compression_mask::MCO_COMPRESSION_OBJ_NODE as u32
    );
    bitmask_flag!(
        /// Enables compression of blob header pages.
        blob_head,
        /// Returns the current flag value.
        get_blob_head,
        mco_compression_mask::MCO_COMPRESSION_BLOB_HEAD as u32
    );
    bitmask_flag!(
        /// Enables compression of blob tail pages.
        blob_tail,
        /// Returns the current flag value.
        get_blob_tail,
        mco_compression_mask::MCO_COMPRESSION_BLOB_TAIL as u32
    );
    bitmask_flag!(
        /// Enables compression of fixed record pages.
        fixed_rec_set,
        /// Returns the current flag value.
        get_fixed_rec_set,
        mco_compression_mask::MCO_COMPRESSION_FIXEDRECSET as u32
    );
}

/// Database open mode mask.
///
/// # Examples
///
/// Requesting the runtime to open an existing database instance:
///
/// ```
/// # use extremedb::database::ModeMask;
/// let mut mask = ModeMask::new();
/// mask.open_existing(true);
/// ```
pub struct ModeMask(BitMask32);

impl ModeMask {
    /// Creates a new mode mask filled with zeroes.
    pub fn new() -> Self {
        ModeMask(BitMask32::new())
    }

    fn from_mco(m: u32) -> Self {
        ModeMask(BitMask32::from(m))
    }

    bitmask_flag!(
        /// Enables automatic cleanup of stale versions (MVCC) when the
        /// database is opened.
        mode_mvcc_auto_vacuum,
        /// Returns the current flag value.
        get_mode_mvcc_auto_vacuum,
        mco_db_mode_mask::MCO_DB_MODE_MVCC_AUTO_VACUUM as u32
    );
    bitmask_flag!(
        /// Causes the runtime to pre-sort transaction data based on the first
        /// index declared for the class prior to including them in indexes.
        mode_smart_index_insert,
        /// Returns the current flag value.
        get_mode_smart_index_insert,
        mco_db_mode_mask::MCO_DB_MODE_SMART_INDEX_INSERT as u32
    );
    bitmask_flag!(
        /// Instructs the runtime not to initialize an all-in-memory database,
        /// but rather treat the database memory device as an already
        /// initialized memory space.
        open_existing,
        /// Returns the current flag value.
        get_open_existing,
        mco_db_mode_mask::MCO_DB_OPEN_EXISTING as u32
    );
    bitmask_flag!(
        /// Instructs the runtime to calculate and store the CRC for each
        /// database page.
        use_crc_check,
        /// Returns the current flag value.
        get_use_crc_check,
        mco_db_mode_mask::MCO_DB_USE_CRC_CHECK as u32
    );
    bitmask_flag!(
        /// Forces all database classes to be "in-memory" (`transient`) even if
        /// the classes are declared `persistent` in the database schema.
        transient,
        /// Returns the current flag value.
        get_transient,
        mco_db_mode_mask::MCO_DB_TRANSIENT as u32
    );
    bitmask_flag!(
        /// Enables on-demand initialization of memory pages.
        lazy_mem_initialization,
        /// Returns the current flag value.
        get_lazy_mem_initialization,
        mco_db_mode_mask::MCO_DB_LAZY_MEM_INITIALIZATION as u32
    );
    bitmask_flag!(
        /// When the MURSIW transaction manager is used, this flag causes the
        /// runtime to save modified pages to the transaction log file and
        /// flush the log outside of a critical section, allowing read-only
        /// transactions to process.
        mursiw_disk_commit_optimization,
        /// Returns the current flag value.
        get_mursiw_disk_commit_optimization,
        mco_db_mode_mask::MCO_DB_MURSIW_DISK_COMMIT_OPTIMIZATION as u32
    );
    bitmask_flag!(
        /// Causes the runtime to sort and write all non-pinned dirty pages,
        /// instead of writing just a single dirty page thrown away from
        /// the disk page pool when loading a new page from disk.
        bulk_write_modified_pages,
        /// Returns the current flag value.
        get_bulk_write_modified_pages,
        mco_db_mode_mask::MCO_DB_BULK_WRITE_MODIFIED_PAGES as u32
    );
    bitmask_flag!(
        /// Causes the runtime to try to preload the next B-Tree page using
        /// the `fadvise()` system call.
        index_preload,
        /// Returns the current flag value.
        get_index_preload,
        mco_db_mode_mask::MCO_DB_INDEX_PRELOAD as u32
    );
    bitmask_flag!(
        /// Disables nested transactions.
        disable_nested_transactions,
        /// Returns the current flag value.
        get_disable_nested_transactions,
        mco_db_mode_mask::MCO_DB_DISABLE_NESTED_TRANSACTIONS as u32
    );
    bitmask_flag!(
        /// Disables implicit rollback of active transactions on disconnect.
        disable_implicit_rollback,
        /// Returns the current flag value.
        get_disable_implicit_rollback,
        mco_db_mode_mask::MCO_DB_DISABLE_IMPLICIT_ROLLBACK as u32
    );
    bitmask_flag!(
        /// Enables page level encryption for in-memory databases.
        inmemory_protection,
        /// Returns the current flag value.
        get_inmemory_protection,
        mco_db_mode_mask::MCO_DB_INMEMORY_PROTECTION as u32
    );
    bitmask_flag!(
        /// Allows the runtime to exploit CPU cache memory for data structures
        /// and algorithms, which in many cases can improve performance
        /// dramatically over normal RAM access.
        inclusive_btree,
        /// Returns the current flag value.
        get_inclusive_btree,
        mco_db_mode_mask::MCO_DB_INCLUSIVE_BTREE as u32
    );
    bitmask_flag!(
        /// Enables page compression for in-memory databases.
        inmemory_compression,
        /// Returns the current flag value.
        get_inmemory_compression,
        mco_db_mode_mask::MCO_DB_INMEMORY_COMPRESSION as u32
    );
    bitmask_flag!(
        /// Allows the runtime to use separate memory allocation bitmap pages
        /// for aligned and unaligned memory objects.
        separate_bitmap,
        /// Returns the current flag value.
        get_separate_bitmap,
        mco_db_mode_mask::MCO_DB_SEPARATE_BITMAP as u32
    );
    bitmask_flag!(
        /// Disables B-Tree rebalancing when deleting objects.
        disable_btree_rebalance_on_delete,
        /// Returns the current flag value.
        get_disable_btree_rebalance_on_delete,
        mco_db_mode_mask::MCO_DB_DISABLE_BTREE_REBALANCE_ON_DELETE as u32
    );
    bitmask_flag!(
        /// If the first stage of a two-phase commit fails, then this flag
        /// causes the runtime to implicitly rollback the transaction,
        /// not waiting for the application to rollback the transaction
        /// explicitly.
        auto_rollback_first_phase,
        /// Returns the current flag value.
        get_auto_rollback_first_phase,
        mco_db_mode_mask::MCO_DB_AUTO_ROLLBACK_FIRST_PHASE as u32
    );
    bitmask_flag!(
        /// Opens a persistent database that was created while using the MVCC
        /// transaction manager with an application using the MURSIW
        /// transaction manager.
        mvcc_compatibility_mode,
        /// Returns the current flag value.
        get_mvcc_compatibility_mode,
        mco_db_mode_mask::MCO_DB_MVCC_COMPATIBILITY_MODE as u32
    );
    bitmask_flag!(
        /// Allows transactions larger than the page pool size (Redo log only).
        redo_log_optimization,
        /// Returns the current flag value.
        get_redo_log_optimization,
        mco_db_mode_mask::MCO_DB_REDO_LOG_OPTIMIZATION as u32
    );
    bitmask_flag!(
        /// Allows skipping the update of indexes if index keys are modified
        /// by an update operation.
        disable_hot_updates,
        /// Returns the current flag value.
        get_disable_hot_updates,
        mco_db_mode_mask::MCO_DB_DISABLE_HOT_UPDATES as u32
    );
    bitmask_flag!(
        /// Causes all newly created database objects to be inserted into the
        /// indexes so that they can be accessed immediately without an
        /// explicit *checkpoint* call.
        sql_autocheckpoint,
        /// Returns the current flag value.
        get_sql_autocheckpoint,
        mco_db_mode_mask::MCO_DB_SQL_AUTOCHECKPOINT as u32
    );
    bitmask_flag!(
        /// Opens the database in read-only mode.
        mode_read_only,
        /// Returns the current flag value.
        get_mode_read_only,
        mco_db_mode_mask::MCO_DB_MODE_READ_ONLY as u32
    );
    bitmask_flag!(
        /// Use Asynchronous IO for persistent database writes.
        use_aio,
        /// Returns the current flag value.
        get_use_aio,
        mco_db_mode_mask::MCO_DB_USE_AIO as u32
    );
    bitmask_flag!(
        /// Enables the marking of pages for incremental backup.
        incremental_backup,
        /// Returns the current flag value.
        get_incremental_backup,
        mco_db_mode_mask::MCO_DB_INCREMENTAL_BACKUP_ENABLED as u32
    );
    bitmask_flag!(
        /// Directs the runtime to block the entire table space instead of
        /// selected instances during database operations (MVCC only).
        mvcc_table_level_locking,
        /// Returns the current flag value.
        get_mvcc_table_level_locking,
        mco_db_mode_mask::MCO_DB_MVCC_TABLE_LEVEL_LOCKING as u32
    );
    bitmask_flag!(
        /// Instructs the runtime to allocate space for the average object
        /// size, calculated from statistics.
        disable_smart_alloc,
        /// Returns the current flag value.
        get_disable_smart_alloc,
        mco_db_mode_mask::MCO_DB_DISABLE_SMART_ALLOC as u32
    );
}

/// Database parameters.
///
/// For detailed information on the individual parameters and their
/// allowed values, refer to the *e*X*treme*DB reference pages.
///
/// # Examples
///
/// Setting the database log type (persistent databases only):
///
/// ```
/// # use extremedb::database::{Params, LogType};
/// # use extremedb::runtime::Runtime;
/// # let runtime = Runtime::start(vec![]);
/// let mut params = Params::new();
/// params.db_log_type(LogType::Redo);
/// ```
pub struct Params {
    p: exdb_sys::mco_db_params_t,
}

impl Params {
    /// Returns a new parameters structure initialized with default values.
    pub fn new() -> Self {
        let mut p = MaybeUninit::uninit();
        unsafe {
            exdb_sys::mco_db_params_init(p.as_mut_ptr());
        }

        Params {
            p: unsafe { p.assume_init() },
        }
    }

    fn replace_c_string(p: *mut *mut i8, s: Option<&str>) -> Result<()> {
        let new_p = match s {
            Some(s) => CString::new(s)
                .or(Err(Error::new_core(mco_ret::MCO_E_ILLEGAL_PARAM)))?
                .into_raw(),
            None => ptr::null_mut(),
        };

        unsafe {
            Params::drop_c_string_if_not_null(*p);
            *p = new_p;
        }

        Ok(())
    }

    fn drop_c_string_if_not_null(p: *mut i8) {
        unsafe {
            if !p.is_null() {
                let cs = CString::from_raw(p);
                drop(cs);
            }
        }
    }

    fn get_c_string(&self, p: *mut i8) -> Result<Option<&str>> {
        if p.is_null() {
            Ok(None)
        } else {
            unsafe {
                let s = CStr::from_ptr(p)
                    .to_str()
                    .or(Err(Error::new_core(mco_ret::MCO_E_ILLEGAL_PARAM)))?;
                Ok(Some(s))
            }
        }
    }

    db_param_scalar!(
        /// Sets the size of the memory page size for conventional and shared
        /// memory devices.
        mem_page_size,
        /// Returns the current parameter value.
        get_mem_page_size,
        u16
    );

    db_param_scalar!(
        /// Sets the size of the persistent storage page.
        disk_page_size,
        /// Returns the current parameter value.
        get_disk_page_size,
        u32
    );

    db_param_scalar!(
        /// Sets the maximum number of connections for this database.
        db_max_connections,
        /// Returns the current parameter value.
        get_db_max_connections,
        u32
    );

    db_param_scalar!(
        /// Sets the maximum size of the persistent database.
        disk_max_database_size,
        /// Returns the current parameter value.
        get_disk_max_database_size,
        usize,
        exdb_sys::mco_offs_t
    );

    db_param_scalar!(
        /// Sets the allocation block size for the database runtime to use when
        /// increasing the size of the database file.
        file_extension_quantum,
        /// Returns the current parameter value.
        get_file_extension_quantum,
        usize,
        exdb_sys::mco_offs_t
    );

    /// Sets the database log type.
    pub fn db_log_type(&mut self, db_log_type: LogType) -> &mut Self {
        self.p.db_log_type = db_log_type as mco_log_type::Type;
        self
    }

    /// Returns the current parameter value.
    pub fn get_db_log_type(&self) -> Option<LogType> {
        LogType::from_mco(self.p.db_log_type)
    }

    /// Indicates by how much the runtime increases the size of the connection
    /// structure to hold any application-specific data (referred to as the
    /// application's context). The context is required for the recovery
    /// process to verify that the task which has created the connection
    /// is alive.
    pub fn connection_context_size_for_recovery(&mut self, enable: bool) -> &mut Self {
        if enable {
            self.p.connection_context_size = mem::size_of::<*const c_void>() as u16;
        } else {
            self.p.connection_context_size = 0;
        }

        self
    }

    /// Returns the current parameter value.
    pub fn get_connection_context_size(&self) -> u16 {
        self.p.connection_context_size
    }

    db_param_scalar!(
        /// Sets the factor used to determine when a hash index table
        /// is extended.
        hash_load_factor,
        /// Returns the current parameter value.
        get_hash_load_factor,
        u16
    );

    db_param_scalar!(
        /// Sets the maximum number of active write transactions when
        /// optimistic locking of B-Tree indexes is being performed.
        index_optimistic_lock_threshold,
        /// Returns the current parameter value.
        get_index_optimistic_lock_threshold,
        u16
    );

    /// Sets the log parameters.
    pub fn log_params(&mut self, log_params: LogParams) -> &mut Self {
        self.p.log_params = log_params.p;
        self
    }

    /// Returns the current parameter value.
    pub fn get_log_params(&self) -> LogParams {
        LogParams::with_params(&self.p.log_params)
    }

    /// Sets the mode mask.
    pub fn mode_mask(&mut self, mode_mask: ModeMask) -> &mut Self {
        self.p.mode_mask = mode_mask.0.bit_mask() as i32;
        self
    }

    /// Returns the current parameter value.
    pub fn get_mode_mask(&self) -> ModeMask {
        ModeMask::from_mco(self.p.mode_mask as u32)
    }

    db_param_scalar!(
        /// Sets the minimum number of pages held by the per-connection
        /// allocator in MVCC mode.
        min_conn_local_pages,
        /// Returns the current parameter value.
        get_min_conn_local_pages,
        u32,
        i32
    );

    db_param_scalar!(
        /// Sets the maximum number of pages held by the per-connection
        /// allocator in MVCC mode.
        max_conn_local_pages,
        /// Returns the current parameter value.
        get_max_conn_local_pages,
        u32,
        i32
    );

    db_param_scalar!(
        /// Sets the cache priority for allocation bitmap pages.
        allocation_bitmap_caching_priority,
        /// Returns the current parameter value.
        get_allocation_bitmap_caching_priority,
        u32,
        i32
    );

    db_param_scalar!(
        /// Sets the cache priority for index pages.
        index_caching_priority,
        /// Returns the current parameter value.
        get_index_caching_priority,
        u32,
        i32
    );

    db_param_scalar!(
        /// Sets the cache priority for database objects (excluding blobs).
        object_caching_priority,
        /// Returns the current parameter value.
        get_object_caching_priority,
        u32,
        i32
    );

    // ddl_dict - unused, do not expose

    db_param_scalar!(
        /// Sets the amount of space reserved for the dictionary in the
        /// database header to allow dynamic schema modification.
        ddl_dict_size,
        /// Returns the current parameter value.
        get_ddl_dict_size,
        usize,
        exdb_sys::mco_size_t
    );

    // ddl_dict_flags - can be adjusted when creating the database; do not expose

    /// Enables database encryption.
    pub fn cipher_key(&mut self, cipher_key: Option<&str>) -> Result<()> {
        Params::replace_c_string(&mut self.p.cipher_key, cipher_key)
    }

    /// Returns the current parameter value.
    pub fn get_cipher_key(&self) -> Result<Option<&str>> {
        self.get_c_string(self.p.cipher_key)
    }

    /// Enables the dynamic hash table extension.
    pub fn dynamic_hash(&mut self, dynamic_hash: bool) -> &mut Self {
        self.p.dynamic_hash = dynamic_hash as i32;
        self
    }

    /// Returns the current parameter value.
    pub fn get_dynamic_hash(&self) -> bool {
        match self.p.dynamic_hash {
            0 => false,
            _ => true,
        }
    }

    /// Sets the *e*X*treme*DB license key.
    pub fn license_key(&mut self, license_key: Option<&str>) -> Result<()> {
        Params::replace_c_string(&mut self.p.license_key, license_key)
    }

    /// Returns the current parameter value.
    pub fn get_license_key(&self) -> Result<Option<&str>> {
        self.get_c_string(self.p.license_key)
    }

    db_param_scalar!(
        /// Sets the amount of space reserved for classes in the database
        /// header to allow dynamic schema modification.
        max_classes,
        /// Returns the current parameter value.
        get_max_classes,
        i32
    );

    db_param_scalar!(
        /// Sets the amount of space reserved for indexes in the database
        /// header to allow dynamic schema modification.
        max_indexes,
        /// Returns the current parameter value.
        get_max_indexes,
        i32
    );

    db_param_scalar!(
        /// Sets the object size threshold (in bytes) that triggers automatic
        /// de-fragmentation during a transaction commit.
        autocompact_threshold,
        /// Returns the current parameter value.
        get_autocompact_threshold,
        usize,
        exdb_sys::mco_size_t
    );

    /// Sets the transaction scheduling policy for transactions with the same
    /// priority.
    pub fn trans_sched_policy(&mut self, trans_sched_policy: TransSchedPolicy) -> &mut Self {
        self.p.trans_sched_policy = trans_sched_policy as mco_trans_sched_policy::Type;
        self
    }

    /// Returns the current parameter value.
    pub fn get_trans_sched_policy(&self) -> Option<TransSchedPolicy> {
        TransSchedPolicy::from_mco(self.p.trans_sched_policy)
    }

    db_param_scalar!(
        /// Sets the maximum transaction time for debugging; has no effect
        /// unless used with a custom-built *e*X*treme*DB runtime.
        max_trans_time,
        /// Returns the current parameter value.
        get_max_trans_time,
        u64
    );

    db_param_scalar!(
        /// Sets the maximum number of pages in the page hash used internally
        /// by the *e*X*treme*DB runtime when encryption or data compression
        /// is enabled.
        max_active_pages,
        /// Returns the current parameter value.
        get_max_active_pages,
        u32,
        i32
    );

    db_param_scalar!(
        /// Sets the number of bundles in the page hash used internally by the
        /// *e*X*treme*DB runtime when encryption or data compression
        /// is enabled.
        page_hash_bundles,
        /// Returns the current parameter value.
        get_page_hash_bundles,
        u32,
        i32
    );

    db_param_scalar!(
        /// Sets the compression level.
        compression_level,
        /// Returns the current parameter value.
        get_compression_level,
        i32
    );

    /// Defines the bitmap of page types to be compressed.
    pub fn compression_mask(&mut self, compression_mask: CompressionMask) -> &mut Self {
        self.p.compression_mask = compression_mask.0.bit_mask() as i32;
        self
    }

    /// Returns the current parameter value.
    pub fn get_compression_mask(&self) -> CompressionMask {
        CompressionMask::from_mco(self.p.compression_mask as u32)
    }

    db_param_scalar!(
        /// Controls the page map allocation.
        expected_compression_ratio,
        /// Returns the current parameter value.
        get_expected_compression_ratio,
        u32,
        i32
    );

    db_param_scalar!(
        /// Sets the number of keys taken from a leaf of a B-Tree page on each
        /// access.
        btree_cursor_read_ahead_size,
        /// Returns the current parameter value.
        get_btree_cursor_read_ahead_size,
        u8
    );

    db_param_scalar!(
        /// Sets the size of a bitmap used internally to accelerate
        /// the performance of the MVCC transaction manager in some cases.
        mvcc_bitmap_size,
        /// Returns the current parameter value.
        get_mvcc_bitmap_size,
        u32,
        i32
    );

    db_param_scalar!(
        /// Sets the amount of heap memory used by various internal
        /// DB functions.
        additional_heap_size,
        /// Returns the current parameter value.
        get_additional_heap_size,
        u32,
        i32
    );

    db_param_scalar!(
        /// Sets the size of the copy-on-write MVCC page map.
        cow_pagemap_size,
        /// Returns the current parameter value.
        get_cow_pagemap_size,
        usize,
        exdb_sys::mco_size_t
    );

    db_param_scalar!(
        /// Sets the size of the backup counters array.
        backup_map_size,
        /// Returns the current parameter value.
        get_backup_map_size,
        usize,
        exdb_sys::mco_size_t
    );

    db_param_scalar!(
        /// Sets the number of pages for the last exclusive pass of
        /// the backup procedure.
        backup_min_pages,
        /// Returns the current parameter value.
        get_backup_min_pages,
        u32
    );

    db_param_scalar!(
        /// Sets the max number of passes before the exclusive pass of
        /// the backup procedure.
        backup_max_passes,
        /// Returns the current parameter value.
        get_backup_max_passes,
        u32
    );

    /// Sets the name of the temporary backup data storage file.
    pub fn backup_map_filename(&mut self, backup_map_filename: &str) -> Result<()> {
        if backup_map_filename.len() >= self.p.backup_map_filename.len() {
            return Err(Error::new_core(mco_ret::MCO_E_ILLEGAL_PARAM));
        }

        unsafe {
            ptr::copy_nonoverlapping(
                backup_map_filename.as_ptr(),
                self.p.backup_map_filename.as_mut_ptr() as *mut u8,
                backup_map_filename.len(),
            )
        }

        Ok(())
    }

    /// Returns the current parameter value.
    pub fn get_backup_map_filename(&self) -> Result<&str> {
        if self.p.backup_map_filename[0] == 0 {
            Ok("")
        } else {
            let cstr = unsafe { CStr::from_ptr(self.p.backup_map_filename.as_ptr()) };
            cstr.to_str()
                .or(Err(Error::new_core(mco_ret::MCO_E_ILLEGAL_PARAM)))
        }
    }

    db_param_scalar!(
        /// Sets the IoT Agent identifier.
        iot_agent_id,
        /// Returns the current parameter value.
        get_iot_agent_id,
        u64
    );

    db_param_scalar!(
        /// Overrides the IoT router level defined in the schema.
        iot_level,
        /// Returns the current parameter value.
        get_iot_level,
        u16
    );

    db_param_scalar!(
        /// Sets the delay in milliseconds between writing backup blocks.
        file_backup_delay,
        /// Returns the current parameter value.
        get_file_backup_delay,
        u32
    );
}

impl Drop for Params {
    fn drop(&mut self) {
        Params::drop_c_string_if_not_null(self.p.cipher_key);
        Params::drop_c_string_if_not_null(self.p.license_key);
    }
}

/// A database instance.
///
/// A database instance cannot be used directly to manipulate the database
/// data; a [`Connection`] must be established and used for operations on the
/// database instead.
///
/// [`Connection`]: ../connection/index.html
pub struct Database<'a> {
    runtime: PhantomData<&'a Runtime>,
    devices: PhantomData<&'a mut Vec<Device>>, // Devices are mutated within eXtremeDB code
    name: CString,
}

impl<'a> Database<'a> {
    /// Opens a new database instance.
    ///
    /// `name` must be an ASCII string; strings containing other characters
    /// will be rejected.
    ///
    /// `dict` is not currently used and must be set to `None`.
    pub fn open(
        _runtime: &'a Runtime,
        name: &str,
        dict: Option<&'a dict::Dictionary>,
        devs: &'a mut Vec<Device>,
        params: Params,
    ) -> Result<Self> {
        if !name.is_ascii() {
            return Err(Error::new_core(mco_ret::MCO_E_ILLEGAL_PARAM));
        }

        let cname = CString::new(name).unwrap();
        let mut params = params;
        let dict_p = match dict {
            Some(d) => &d.nested as *const exdb_sys::mco_dictionary_t,
            None => ptr::null_mut(),
        };

        result_from_code(unsafe {
            exdb_sys::mco_db_open_dev(
                cname.as_ptr(),
                dict_p as *mut exdb_sys::mco_dictionary_t,
                devs.as_mut_ptr() as *mut exdb_sys::mco_device_t,
                devs.len() as exdb_sys::mco_size_t,
                &mut params.p,
            )
        })?;

        Ok(Database {
            runtime: PhantomData,
            devices: PhantomData,
            name: cname,
        })
    }

    /// Removes a shared memory segment associated with a database.
    ///
    /// Also removes `name` from the registry.
    ///
    /// This function is a no-op if the specified database does not exist.
    ///
    /// # Safety
    ///
    /// If there are active database sessions using this database segment,
    /// the behavior is undefined.
    pub unsafe fn kill(name: &str) -> Result<()> {
        if !name.is_ascii() {
            Err(Error::new_core(mco_ret::MCO_E_ILLEGAL_PARAM))
        } else {
            let cname = CString::new(name).unwrap();
            result_from_code(exdb_sys::mco_db_kill(cname.as_ptr()))
        }
    }

    /// Returns the database name.
    pub fn name(&self) -> &CStr {
        &self.name
    }
}

impl<'a> Drop for Database<'a> {
    fn drop(&mut self) {
        let rc = unsafe { exdb_sys::mco_db_close(self.name.as_ptr()) };
        debug_assert_eq!(mco_ret::MCO_S_OK, rc);
    }
}
