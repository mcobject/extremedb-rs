// engine.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

//! SQL engine and related types.
//!
//! SQL engines implement the SQL functionality for *e*X*treme*DB.
//!
//! This module defines the local SQL engine type, as well as the [`Engine`]
//! trait, which is implemented by both the local SQL engine and the remote
//! SQL client.
//!
//! [`Engine`]: ./trait.Engine.html
//!
//! # Local Sessions and Engine References
//!
//! An SQL engine itself is not thread-safe. *SQL sessions* are used to access
//! an engine in a thread-safe manner.
//!
//! Sessions are created from the engine references, [`LocalEngineRef`].
//! The references are `Send` (but not `Sync`).
//! However, the references cannot outlive the engine they reference. Because
//! of that, it is impossible to pass an engine reference to a thread, since
//! values captured by the thread closures are expected to have the `'static`
//! lifetime, and the references are bounded by their target engine's lifetime,
//! which is always shorter than `'static` (since it is impossible to create
//! a static engine).
//!
//! The workaround is to create an unbounded engine reference, but the calling
//! code is responsible for making sure that the threads do not outlive
//! the engine, i.e. they are joined explicitly before the engine is dropped.
//! An example below illustrates this idea.
//!
//! [`LocalEngineRef`]: ./struct.LocalEngineRef.html
//!
//! # Examples
//!
//! In a single-threaded application, the SQL engine can be used as is
//! to execute SQL statements:
//!
//! ```
//! # use extremedb::sql::engine::Engine;
//! # use extremedb::{connection, database, device, runtime, sql};
//! # use extremedb::device::util;
//! # fn main() -> extremedb::Result<()> {
//! #     let runtime = runtime::Runtime::start(vec![]);
//! #     let mut db_params = database::Params::new();
//! #     db_params
//! #         .ddl_dict_size(32768)
//! #         .max_classes(100)
//! #         .max_indexes(1000);
//! #     let mut devs = util::DeviceContainer::new();
//!     let db = database::Database::open(
//! #         &runtime,
//! #         "test_db",
//! #         None,
//! #         devs.devices(),
//! #         db_params,
//!         // ...
//!     )?;
//!     let conn = connection::Connection::new(&db)?;
//!
//!     let engine = sql::engine::LocalEngine::new(&conn)?;
//!
//!     engine.execute_statement("CREATE TABLE TestTable(i integer, s string);", &[])?;
//!
//!     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&1, &"Hello"])?;
//!     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&2, &"World"])?;
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! Multi-threaded applications require threads to access the engine using
//! sessions. To start a session in a thread, it is possible to pass an
//! unbounded engine reference to the thread. However, the calling code must
//! be careful to join the threads before the engine is dropped:
//!
//! ```
//! # use extremedb::sql::engine::{Engine, LocalEngineRef, LocalEngineSession};
//! # use extremedb::{connection, database, device, runtime, sql};
//! # use extremedb::device::util;
//! # use std::thread;
//! # fn main() -> extremedb::Result<()> {
//! #     let runtime = runtime::Runtime::start(vec![]);
//! #     let mut db_params = database::Params::new();
//! #     db_params
//! #         .ddl_dict_size(32768)
//! #         .max_classes(100)
//! #         .max_indexes(1000);
//! #     let mut devs = util::DeviceContainer::new();
//! #     let db = database::Database::open(&runtime, "test_db", None, devs.devices(), db_params)?;
//! #     let conn = connection::Connection::new(&db)?;
//! #     let engine = sql::engine::LocalEngine::new(&conn)?;
//!     engine.execute_statement("CREATE TABLE TestTable(i integer);", &[])?;
//!
//!     // Creating unbounded engine references is unsafe.
//!     let r1 = unsafe { LocalEngineRef::new_unbounded(&engine) };
//!     let r2 = unsafe { LocalEngineRef::new_unbounded(&engine) };
//!
//!     // Launch the first thread.
//!     let t1 = thread::spawn(move || {
//!         let s1 = LocalEngineSession::new(r1).unwrap();
//!
//!         s1.execute_statement("INSERT INTO TestTable VALUES(101);", &[]).unwrap();
//!         s1.execute_statement("INSERT INTO TestTable VALUES(102);", &[]).unwrap();
//!         s1.execute_statement("INSERT INTO TestTable VALUES(103);", &[]).unwrap();
//!     });
//!
//!     // Launch the second thread.
//!     let t2 = thread::spawn(move || {
//!         let s2 = LocalEngineSession::new(r2).unwrap();
//!
//!         s2.execute_statement("INSERT INTO TestTable VALUES(201);", &[]).unwrap();
//!         s2.execute_statement("INSERT INTO TestTable VALUES(202);", &[]).unwrap();
//!     });
//!
//!     // Join the threads explicitly.
//!     t1.join().expect("Failed to join thread 1");
//!     t2.join().expect("Failed to join thread 2");
//! #
//! #     let ds = engine.execute_query("SELECT COUNT(*) FROM TestTable;", &[])?;
//! #     assert!(ds.is_some());
//! #     let ds = ds.unwrap();
//! #
//! #     let mut cur = ds.cursor()?;
//! #     assert_eq!(cur.advance()?, true);
//! #
//! #     let rec = cur.current_record();
//! #     assert!(rec.is_some());
//! #     let rec = rec.unwrap();
//! #
//! #     let count_val = rec.get_at(0)?;
//! #     assert_eq!(count_val.to_i64()?, 5);
//! #
//! #     Ok(())
//! # }
//! ```

use std::marker::PhantomData;
use std::mem::MaybeUninit;

use crate::connection::Connection;
use crate::sql::data_source::DataSource;
use crate::sql::stmt::{ExecutionContext, Statement};
use crate::sql::value::ToValue;
use crate::sql::{mcosql_error_code, result_from_code};
use crate::{exdb_sys, Result};

/// The common SQL Engine trait.
///
/// This trait is implemented by both the local SQL engine and the remote
/// SQL client.
pub trait Engine {
    /// Returns the internal engine handle.
    fn get_engine(&self) -> exdb_sys::database_t;

    /// Executes the SQL statement in the context of the engine.
    ///
    /// Returns the number of affected rows, if available.
    fn execute_statement(&self, sql: &str, args: &[&dyn ToValue]) -> Result<i64> {
        Statement::execute_statement(ExecutionContext::with_engine(self), sql, args)
    }

    /// Executes the SQL query in the context of the engine.
    ///
    /// Returns the produced data source if available, otherwise `None`.
    fn execute_query<'a>(
        &'a self,
        sql: &str,
        args: &[&dyn ToValue],
    ) -> Result<Option<DataSource<'a>>> {
        Statement::execute_query(ExecutionContext::with_engine(self), sql, args)
    }
}

/// A local SQL engine.
///
/// A local engine can be used as is, or accessed through a
/// [`LocalEngineSession`] in multi-threaded applications.
///
/// [`LocalEngineSession`]: ./struct.LocalEngineSession.html
pub struct LocalEngine<'a> {
    conn: PhantomData<&'a Connection<'a>>,
    pub(crate) h: exdb_sys::database_t,
}

impl<'a> LocalEngine<'a> {
    /// Creates a new local SQL engine using the database connection `conn`.
    pub fn new(conn: &'a Connection) -> Result<Self> {
        let mut h = MaybeUninit::uninit();

        // Create and initialize McoSqlEngine.
        result_from_code(unsafe { exdb_sys::mcoapi_create_engine(conn.handle(), h.as_mut_ptr()) })
            .and(Ok(LocalEngine {
                conn: PhantomData,
                h: unsafe { h.assume_init() },
            }))
    }
}

impl<'a> Drop for LocalEngine<'a> {
    fn drop(&mut self) {
        unsafe {
            let rc = exdb_sys::mcoapi_destroy_engine(self.h);
            debug_assert_eq!(mcosql_error_code::SQL_OK, rc);
        }
    }
}

impl<'a> Engine for LocalEngine<'a> {
    fn get_engine(&self) -> exdb_sys::database_t {
        self.h
    }
}

/// A local SQL engine reference.
///
/// The references are intended to be passed to threads in multi-threaded
/// applications. The threads can use them to create sessions for safe
/// shared usage of the local engine.
///
/// However, standard library threads require all captured variables to have
/// `'static` lifetime. Because of this, it is impossible to pass a reference
/// bounded by the engine's lifetime to a thread. An unsafe method,
/// [`new_unbounded()`], is provided to work around this limitation.
///
/// [`new_unbounded()`]: #method.new_unbounded
pub struct LocalEngineRef<'a> {
    engine: PhantomData<&'a LocalEngine<'a>>,
    pub(crate) h: exdb_sys::database_t,
}

impl<'a> LocalEngineRef<'a> {
    /// Creates a new local SQL engine reference. The returned value is bounded
    /// by the target engine's lifetime, making it unsuitable for use with the
    /// standard library threads.
    pub fn new(engine: &'a LocalEngine) -> Self {
        LocalEngineRef {
            engine: PhantomData,
            h: engine.h,
        }
    }

    /// Creates an unbounded local SQL engine reference. The returned value
    /// can be passed to the standard library threads.
    ///
    /// # Safety
    ///
    /// The calling code is responsible for explicitly joining any threads
    /// that receive an unbounded engine reference prior to dropping the
    /// engine.
    pub unsafe fn new_unbounded(engine: &LocalEngine) -> Self {
        LocalEngineRef {
            engine: PhantomData,
            h: engine.h,
        }
    }
}

unsafe impl Send for LocalEngineRef<'_> {}

/// A local SQL engine session.
///
/// An SQL engine is not thread-safe. Threads can use sessions for safe
/// concurrent access to the engine.
///
/// A session can be used in the same manner as the local engine.
pub struct LocalEngineSession<'a> {
    engine: PhantomData<LocalEngineRef<'a>>,
    h: exdb_sys::mcosql_rs_session,
}

impl<'a> LocalEngineSession<'a> {
    /// Creates a new session.
    pub fn new(engine_ref: LocalEngineRef<'a>) -> Result<Self> {
        let mut h = MaybeUninit::uninit();

        result_from_code(unsafe {
            exdb_sys::mcosql_rs_session_create(engine_ref.h, h.as_mut_ptr())
        })
        .and(Ok(LocalEngineSession {
            engine: PhantomData,
            h: unsafe { h.assume_init() },
        }))
    }
}

impl<'a> Engine for LocalEngineSession<'a> {
    fn get_engine(&self) -> exdb_sys::database_t {
        // McoSqlSession is a subclass of McoSqlEngine; it is safe to return
        // its pointer here.
        self.h as exdb_sys::database_t
    }
}

impl<'a> Drop for LocalEngineSession<'a> {
    fn drop(&mut self) {
        let rc = unsafe { exdb_sys::mcosql_rs_session_destroy(self.h) };
        debug_assert_eq!(mcosql_error_code::SQL_OK, rc);
    }
}
