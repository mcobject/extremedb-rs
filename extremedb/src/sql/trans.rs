// trans.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

//! An SQL transaction.
//!
//! In *e*X*treme*DB SQL, transactions can be started either using the
//! `START TRANSACTION;` SQL statement, or using the [`Transaction`]
//! type provided by this module. Notice that the latter option is only
//! available for the local SQL engine.
//!
//! After the transaction object is created, it can be used to execute
//! statements and queries. The transaction has to be committed explicitly
//! if needed; dropping the transaction object causes the transaction to be
//! rolled back implicitly.
//!
//! [`Transaction`]: ./struct.Transaction.html
//!
//! # Examples
//!
//! Starting a transaction, using it to insert some data, and committing it:
//!
//! ```
//! # use extremedb::sql::engine::Engine;
//! # use extremedb::sql::trans::{Mode, Transaction};
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
//! #     engine.execute_statement("CREATE TABLE TestTable(i integer, s string);", &[])?;
//!     let txn = Transaction::begin(&engine, Mode::ReadWrite, 0)?;
//!
//!     txn.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&1, &"Hello"])?;
//!     txn.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&2, &"World"])?;
//!
//!     txn.commit()?;
//! #     Ok(())
//! # }
//! ```

use std::mem::{self, MaybeUninit};
use std::ptr;

use crate::sql::data_source::DataSource;
use crate::sql::engine::{Engine, LocalEngine};
use crate::sql::stmt::{ExecutionContext, Statement};
use crate::sql::value::ToValue;
use crate::sql::{mcosql_error_code, result_from_code};
use crate::{exdb_sys, Result};

/// Transaction mode.
pub enum Mode {
    /// Read-only.
    ReadOnly = exdb_sys::mcosql_transaction_mode::TM_READ_ONLY as isize,
    /// Update (not currently supported).
    Update = exdb_sys::mcosql_transaction_mode::TM_UPDATE as isize,
    /// Read-write.
    ReadWrite = exdb_sys::mcosql_transaction_mode::TM_READ_WRITE as isize,
    /// Exclusive.
    ///
    /// Prohibits any other concurrent transactions, even if supported by the
    /// current transaction manager.
    Exclusive = exdb_sys::mcosql_transaction_mode::TM_EXCLUSIVE as isize,
}

/// A transaction.
///
/// This type allows for explicit transaction control when using the local
/// SQL engine.
///
/// Transactions have to be committed explicitly; dropping an active
/// transaction causes an implicit rollback.
pub struct Transaction<'a> {
    pub(crate) engine: &'a dyn Engine,
    pub(crate) h: exdb_sys::transaction_t,
}

impl<'a> Transaction<'a> {
    /// Starts a new transaction.
    pub fn begin(engine: &'a LocalEngine, mode: Mode, priority: i32) -> Result<Transaction<'a>> {
        let mut h = MaybeUninit::uninit();

        result_from_code(unsafe {
            exdb_sys::mcosql_begin_transaction(
                engine.h,
                h.as_mut_ptr(),
                mode as exdb_sys::mcosql_transaction_mode::Type,
                priority,
            )
        })
        .and(Ok(Transaction {
            engine,
            h: unsafe { h.assume_init() },
        }))
    }

    /// Executes the SQL statement in the context of the transaction.
    ///
    /// Returns the number of affected rows, if available.
    pub fn execute_statement(&self, sql: &str, args: &[&dyn ToValue]) -> Result<i64> {
        Statement::execute_statement(ExecutionContext::with_transaction(self), sql, args)
    }

    /// Executes the SQL query in the context of the transaction.
    ///
    /// Returns the produced data source if available, otherwise `None`.
    pub fn execute_query(
        &'a self,
        sql: &str,
        args: &[&dyn ToValue],
    ) -> Result<Option<DataSource<'a>>> {
        Statement::execute_query(ExecutionContext::with_transaction(self), sql, args)
    }

    /// Commits the transaction.
    pub fn commit(mut self) -> Result<()> {
        self.finalize(true)
    }

    /// Rolls back the transaction.
    pub fn rollback(mut self) -> Result<()> {
        self.finalize(false)
    }

    fn finalize(&mut self, commit: bool) -> Result<()> {
        result_from_code(unsafe {
            let rc = if commit {
                exdb_sys::mcosql_commit_transaction(self.h)
            } else {
                exdb_sys::mcosql_rollback_transaction(self.h)
            };

            let rc2 =
                exdb_sys::mcosql_release_transaction(mem::replace(&mut self.h, ptr::null_mut()));

            if rc != mcosql_error_code::SQL_OK {
                rc
            } else {
                rc2
            }
        })
    }
}

impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        if !self.h.is_null() {
            let ret = self.finalize(false);
            debug_assert!(ret.is_ok());
        }
    }
}
