// lib.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

//! `extremedb` is a wrapper for the [McObject]'s *e*X*treme*DB database
//! management system.
//!
//! This crate currently supports the *e*X*treme*DB SQL APIs, as well
//! as parts of the core functionality necessary for configuring the
//! *e*X*treme*DB runtime, creating and opening a database, and managing
//! database connections. Core data manipulation APIs are planned for a future
//! release.
//!
//! For more information on the *e*X*treme*DB database management system,
//! refer to the [documentation pages].
//!
//! [documentation pages]: https://www.mcobject.com/docs
//! [McObject]: https://www.mcobject.com
//!
//! # Building
//!
//! This crate depends on the `extremedb_sys` crate, which has some
//! prerequisites, as well as a few environment variables to be set.
//! Refer to the `extremedb_sys` crate reference for more information.
//!
//! # Optional Features
//!
//! - **`sql`** — SQL engine.
//! - **`rsql`** — Remote SQL engine (SQL server and client).
//! - **`sequences`** — Sequences (vertical storage).
//!
//! # SQL Example
//!
//! The example below demonstrates creating an in-memory database and using the
//! DDL and DML SQL statements to manipulate its schema and contents.
//!
//! ```
//! use extremedb::connection::Connection;
//! use extremedb::database::{Database, Params};
//! use extremedb::device::{Assignment, Device};
//! use extremedb::runtime::Runtime;
//! use extremedb::sql::engine::{Engine, LocalEngine};
//! use extremedb::Result;
//!
//! fn main() -> Result<()> {
//!     // Size of the memory region to allocate for the database.
//!     const DB_SIZE: usize = 1024 * 1024;
//!
//!     // eXtremeDB runtime must be started before any operations can be performed.
//!     let runtime = Runtime::start(vec![]);
//!
//! #     if runtime.info().multiprocess_access_supported() {
//! #         return Ok(());
//! #     }
//!     // This example creates a conventional memory device, and will not work with the
//!     // shared memory runtime.
//!     assert!(!runtime.info().multiprocess_access_supported());
//!
//!     // Initialize database parameters. SQL databases require these three
//!     // parameters to be explicitly set.
//!     let mut db_params = Params::new();
//!     db_params
//!         .ddl_dict_size(32768)
//!         .max_classes(100)
//!         .max_indexes(1000);
//!
//!     // Allocate one in-memory device for the database.
//!     let mut devs = vec![Device::new_mem_conv(Assignment::Database, DB_SIZE)?];
//!
//!     // Open the database using the parameters and devices defined above.
//!     let db = Database::open(&runtime, "test_db", None, &mut devs, db_params)?;
//!
//!     // Establish a connection to the database.
//!     let conn = Connection::new(&db)?;
//!
//!     // Create a local SQL engine on top of the database connection.
//!     let engine = LocalEngine::new(&conn)?;
//!
//!     // Use SQL DDL to create a new table.
//!     engine.execute_statement("CREATE TABLE TestTable(i integer, s string);", &[])?;
//!
//!     // Insert two rows into the table.
//!     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&1, &"Hello"])?;
//!     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&2, &"World"])?;
//!
//!     // Select the rows.
//!     let ds = engine.execute_query("SELECT i, s FROM TestTable ORDER BY i;", &[])?;
//!
//!     // The returned data source is not expected to be empty in case of a
//!     // SELECT statement.
//!     assert!(ds.is_some());
//!     let ds = ds.unwrap();
//!
//!     // Create a cursor to iterate over the data source.
//!     let mut cur = ds.cursor()?;
//!
//!     // Read the first row.
//!     {
//!         // Advance the cursor to the first row. It is positioned
//!         // before-the-first row initially.
//!         assert_eq!(cur.advance()?, true);
//!
//!         // Get reference to the record which the cursor points at.
//!         // Since cur.advance() above has returned true, the current record
//!         // must not be None.
//!         let rec = cur.current_record();
//!         assert!(rec.is_some());
//!         let rec = rec.unwrap();
//!
//!         // Get the references to the values in the columns at indexes 0 and 1.
//!         let i_val = rec.get_at(0)?;
//!         let s_val = rec.get_at(1)?;
//!
//!         // SQL value references must be explicitly converted to target types.
//!         assert_eq!(i_val.to_i64()?, 1);
//!         assert_eq!(s_val.to_string()?, "Hello");
//!     }
//!
//!     // The second row must be available.
//!     assert_eq!(cur.advance()?, true);
//!
//!     // No more rows are expected.
//!     assert_eq!(cur.advance()?, false);
//!
//!     Ok(())
//! }
//! ```

use extremedb_sys as exdb_sys;

use std::error;
use std::ffi::CStr;
use std::fmt::{Display, Error as FmtError, Formatter};
use std::str;

/// Core return codes (generated by bindgen from `MCO_RET` in *mco.h*).
pub use exdb_sys::MCO_RET_E_ as mco_ret;

pub mod connection;
pub mod database;
pub mod device;
pub mod dict;
pub mod runtime;

#[cfg(feature = "sql")]
pub mod sql;

mod util;

#[cfg(feature = "sql")]
use sql::{McoSqlStatusCode, SqlError};

/// Type alias for the *e*X*treme*DB status codes returned by most functions.
///
/// The actual codes are imported by `bindgen` from the *mco.h* header,
/// and are available in the [`mco_ret`] module.
///
/// [`mco_ret`]: ./mco_ret/index.html
pub type McoRetCode = mco_ret::Type;

/// An `Error`-compatible wrapper for the *e*X*treme*DB status codes.
#[derive(Debug)]
pub struct CoreError(McoRetCode);

impl CoreError {
    pub(crate) fn new(rc: McoRetCode) -> Self {
        CoreError(rc)
    }

    /// Returns the underlying *e*X*treme*DB status code (one of the constants
    /// defined in the [`mco_ret`] module).
    ///
    /// [`mco_ret`]: ./mco_ret/index.html
    pub fn code(&self) -> McoRetCode {
        self.0
    }

    fn strerror(&self) -> &'static str {
        let cstr = unsafe { CStr::from_ptr(exdb_sys::mco_strerror(self.0)) };
        let res = cstr.to_str();

        debug_assert!(res.is_ok(), "invalid string received from mco_strerror()");

        match res {
            Err(_) => "<malformed error string>",
            Ok(s) => s,
        }
    }
}

impl error::Error for CoreError {}

impl Display for CoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), FmtError> {
        write!(f, "{}", self.strerror())
    }
}

/// An error type for the core database management functions.
#[derive(Debug)]
pub enum Error {
    /// A core API error.
    Core(CoreError),

    /// An SQL API error.
    #[cfg(feature = "sql")]
    Sql(SqlError),
}

impl Error {
    pub(crate) fn new_core(rc: McoRetCode) -> Self {
        Error::Core(CoreError::new(rc))
    }

    #[cfg(feature = "sql")]
    pub(crate) fn new_sql(rc: McoSqlStatusCode) -> Self {
        Error::Sql(SqlError::new(rc))
    }
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), FmtError> {
        match self {
            Error::Core(e) => e.fmt(f),

            #[cfg(feature = "sql")]
            Error::Sql(e) => e.fmt(f),
        }
    }
}

/// Type alias for `std::result::Result` used by the *e*X*treme*DB APIs.
pub type Result<T> = std::result::Result<T, Error>;

fn result_from_code(return_code: McoRetCode) -> Result<()> {
    match return_code {
        mco_ret::MCO_S_OK => Ok(()),
        _ => Err(Error::Core(CoreError::new(return_code))),
    }
}
