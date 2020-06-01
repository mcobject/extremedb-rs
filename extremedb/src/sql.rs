// sql.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

//! The SQL API.
//!
//! This module requires the `sql` feature to be enabled.
//!
//! For information about *e*X*treme*DB SQL syntax and features, refer to the
//! *e*X*treme*DB SQL reference documentation.
//!
//! # Local and Remote Operation
//!
//! *e*X*treme*DB SQL supports both *local* and *remote* operation.
//!
//! In the former case, SQL statements are executed locally, using a *local SQL
//! engine* working over a database connection.
//!
//! In the latter case, a *remote SQL server* is bound to a local engine and
//! exposes a network API for remote clients  to use. The clients can submit
//! arbitrary SQL statements, which are executed by the server, and the results
//! are passed back to the clients.
//!
//! Remote SQL functionality is enabled using the `rsql` feature.
//!
//! # Examples
//!
//! Currently, SQL can only be used with dynamic *e*X*treme*DB dictionary,
//! which requires setting certain parameters when the database is created:
//!
//! ```
//! # use extremedb::database;
//! # fn main() {
//!     let mut db_params = database::Params::new();
//!     db_params
//!         .ddl_dict_size(32768)
//!         .max_classes(100)
//!         .max_indexes(1000);
//! #     drop(db_params);
//! # }
//! ```
//!
//! A local engine can be created using an existing database connection:
//!
//! ```
//! # use extremedb::{connection, database, device, runtime, sql};
//! # fn main() -> extremedb::Result<()> {
//! #    let runtime = runtime::Runtime::start(vec![]);
//! #    let mut db_params = database::Params::new();
//! #    db_params
//! #        .ddl_dict_size(32768)
//! #        .max_classes(100)
//! #        .max_indexes(1000);
//! #    let mut devs = vec![device::Device::new_mem_conv(
//! #        device::Assignment::Database,
//! #        1024 * 1024,
//! #    )?];
//!     let db = database::Database::open(
//! #         &runtime,
//! #         "test_db",
//! #         None,
//! #         &mut devs,
//! #         db_params,
//!         // ...
//!     )?;
//!
//!     let conn = connection::Connection::new(&db)?;
//!
//!     let engine = sql::engine::LocalEngine::new(&conn)?;
//! #
//! #     drop(engine);
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! The engine can be used to execute both DDL and DML statements:
//!
//! ```
//! # use extremedb::sql::engine::Engine;
//! # use extremedb::{connection, database, device, runtime, sql};
//! #
//! # fn main() -> extremedb::Result<()> {
//! #     let runtime = runtime::Runtime::start(vec![]);
//! #     let mut db_params = database::Params::new();
//! #     db_params
//! #         .ddl_dict_size(32768)
//! #         .max_classes(100)
//! #         .max_indexes(1000);
//! #     let mut devs = vec![device::Device::new_mem_conv(
//! #         device::Assignment::Database,
//! #         1024 * 1024,
//! #     )?];
//! #
//! #     let db = database::Database::open(&runtime, "test_db", None, &mut devs, db_params)?;
//! #     let conn = connection::Connection::new(&db)?;
//! #     let engine = sql::engine::LocalEngine::new(&conn)?;
//! #
//!     engine.execute_statement("CREATE TABLE TestTable(i integer, s string);", &[])?;
//!
//!     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(1, 'Hello');", &[])?;
//!     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(2, 'World');", &[])?;
//!
//!     engine.execute_query("SELECT i, s FROM TestTable ORDER BY i;", &[])?;
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! Any type that implements the [`ToValue`] trait can be passed as a
//! statement parameter. This module provides implementations of this
//! trait for many common types.
//!
//! [`ToValue`]: ./value/trait.ToValue.html
//!
//! ```
//! # use extremedb::sql::engine::Engine;
//! # use extremedb::{connection, database, device, runtime, sql};
//! # fn main() -> extremedb::Result<()> {
//! #     let runtime = runtime::Runtime::start(vec![]);
//! #     let mut db_params = database::Params::new();
//! #     db_params
//! #         .ddl_dict_size(32768)
//! #         .max_classes(100)
//! #         .max_indexes(1000);
//! #     let mut devs = vec![device::Device::new_mem_conv(
//! #         device::Assignment::Database,
//! #         1024 * 1024,
//! #     )?];
//! #
//! #     let db = database::Database::open(&runtime, "test_db", None, &mut devs, db_params)?;
//! #     let conn = connection::Connection::new(&db)?;
//! #     let engine = sql::engine::LocalEngine::new(&conn)?;
//! #     engine.execute_statement("CREATE TABLE TestTable(i integer, s string);", &[])?;
//! #
//!     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&1, &"Hello"])?;
//!     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&2, &"World"])?;
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! Queries usually return a *data source*, and *cursors* can be used to
//! iterate over the *records* in the data source. Individual column values
//! in a record can be accessed by their column numbers.
//!
//! ```
//! # use extremedb::sql::engine::Engine;
//! # use extremedb::{connection, database, device, runtime, sql};
//! # fn main() -> extremedb::Result<()> {
//! #     let runtime = runtime::Runtime::start(vec![]);
//! #     let mut db_params = database::Params::new();
//! #     db_params
//! #         .ddl_dict_size(32768)
//! #         .max_classes(100)
//! #         .max_indexes(1000);
//! #     let mut devs = vec![device::Device::new_mem_conv(
//! #         device::Assignment::Database,
//! #         1024 * 1024,
//! #     )?];
//! #     let db = database::Database::open(&runtime, "test_db", None, &mut devs, db_params)?;
//! #     let conn = connection::Connection::new(&db)?;
//! #     let engine = sql::engine::LocalEngine::new(&conn)?;
//! #     // Create and populate the table.
//! #     engine.execute_statement("CREATE TABLE TestTable(i integer, s string);", &[])?;
//! #     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&1, &"Hello"])?;
//! #     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&2, &"World"])?;
//! #
//!     // Execute the query and fetch the data source.
//!     let ds = engine.execute_query("SELECT i, s FROM TestTable ORDER BY i;", &[])?;
//!
//!     // `ds` is not expected to be `None` in this example.
//!     assert!(ds.is_some());
//!     let ds = ds.unwrap();
//!
//!     // Get the cursor.
//!     let mut cur = ds.cursor()?;
//!
//!     // The cursor is initially positioned before the first element.
//!     // We expect `advance()` to return true, indicating availability
//!     // of a record.
//!     assert_eq!(cur.advance()?, true);
//!
//!     // Fetch the record and assert it is not `None`.
//!     let rec = cur.current_record();
//!     assert!(rec.is_some());
//!     let rec = rec.unwrap();
//!
//!     // Get the values from the columns 0 and 1.
//!     let i_val = rec.get_at(0)?;
//!     let s_val = rec.get_at(1)?;
//!
//!     assert_eq!(i_val.to_i64()?, 1);
//!     assert_eq!(s_val.to_string()?, "Hello");
//! #     Ok(())
//! # }
//! ```

use crate::exdb_sys;

use std::error;
use std::fmt::{Display, Error as FmtError, Formatter};

pub mod allocator;
pub mod data_source;
pub mod engine;
pub mod trans;
pub mod value;

#[cfg(feature = "rsql")]
pub mod rsql;

mod stmt;

/// SQL return codes (generated by bindgen from `mcosql_error_code` in
/// *sql/sqlc.h*).
pub use exdb_sys::mcosql_error_code;

use crate::Error;

/// Type alias for the *e*X*treme*DB SQL status codes.
///
/// The actual codes are imported by `bindgen` from the *sql/sqlc.h* header,
/// and are available in the [`mcosql_error_code`] module.
///
/// [`mcosql_error_code`]: ./mcosql_error_code/index.html
pub type McoSqlStatusCode = exdb_sys::status_t;

/// An `Error`-compatible wrapper for the *e*X*treme*DB SQL status codes.
#[derive(Debug)]
pub struct SqlError(McoSqlStatusCode);

impl SqlError {
    pub(crate) fn new(rc: McoSqlStatusCode) -> Self {
        SqlError(rc)
    }

    /// Returns the underlying *e*X*treme*DB SQL status code (one of the
    /// constants defined in the [`mcosql_error_code`] module).
    ///
    /// [`mcosql_error_code`]: ./mcosql_error_code/index.html
    pub fn code(&self) -> McoSqlStatusCode {
        self.0
    }

    fn code_str(&self) -> &'static str {
        match self.0 {
            mcosql_error_code::SQL_OK => "SQL_OK",
            mcosql_error_code::NO_MORE_ELEMENTS => "NO_MORE_ELEMENTS",
            mcosql_error_code::INVALID_TYPE_CAST => "INVALID_TYPE_CAST",
            mcosql_error_code::COMPILE_ERROR => "COMPILE_ERROR",
            mcosql_error_code::NOT_SINGLE_VALUE => "NOT_SINGLE_VALUE",
            mcosql_error_code::INVALID_OPERATION => "INVALID_OPERATION",
            mcosql_error_code::INDEX_OUT_OF_BOUNDS => "INDEX_OUT_OF_BOUNDS",
            mcosql_error_code::NOT_ENOUGH_MEMORY => "NOT_ENOUGH_MEMORY",
            mcosql_error_code::NOT_UNIQUE => "NOT_UNIQUE",
            mcosql_error_code::NOT_PREPARED => "NOT_PREPARED",
            mcosql_error_code::RUNTIME_ERROR => "RUNTIME_ERROR",
            mcosql_error_code::COMMUNICATION_ERROR => "COMMUNICATION_ERROR",
            mcosql_error_code::UPGRAGE_NOT_POSSIBLE => "UPGRAGE_NOT_POSSIBLE",
            mcosql_error_code::SQL_CONFLICT => "SQL_CONFLICT",
            mcosql_error_code::SQL_NULL_REFERENCE => "SQL_NULL_REFERENCE",
            mcosql_error_code::SQL_INVALID_STATE => "SQL_INVALID_STATE",
            mcosql_error_code::SQL_INVALID_OPERAND => "SQL_INVALID_OPERAND",
            mcosql_error_code::SQL_NULL_VALUE => "SQL_NULL_VALUE",
            mcosql_error_code::SQL_BAD_CSV_FORMAT => "SQL_BAD_CSV_FORMAT",
            mcosql_error_code::SQL_SYSTEM_ERROR => "SQL_SYSTEM_ERROR",
            _ => "<unknown>",
        }
    }
}

impl error::Error for SqlError {}

impl Display for SqlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), FmtError> {
        write!(f, "{}", self.code_str())
    }
}

fn result_from_code(return_code: McoSqlStatusCode) -> crate::Result<()> {
    match return_code {
        mcosql_error_code::SQL_OK => Ok(()),
        _ => Err(Error::Sql(SqlError::new(return_code))),
    }
}
