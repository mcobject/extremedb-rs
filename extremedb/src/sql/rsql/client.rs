// client.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

//! Remote SQL client functionality.
//!
//! This module implements the remote SQL engine, which can be used to connect
//! to the remote SQL servers and execute arbitrary SQL statements.
//!
//! # Examples
//!
//! Connecting to a server running on the localhost at a known port and
//! executing SQL statements:
//!
//! ```
//! # use extremedb::sql::engine::{Engine, LocalEngine, LocalEngineRef};
//! # use extremedb::sql::rsql::client::{self, RemoteEngine};
//! # use extremedb::sql::rsql::server::{self, Server};
//! # use extremedb::{connection, database, device, runtime, sql};
//! # use extremedb::device::util;
//! # use std::sync::Arc;
//! # use std::thread;
//! #
//! # fn server_try_create<'a>(engine: &'a LocalEngine, port: u16) -> Option<Server<'a>> {
//! #     let eref = LocalEngineRef::new(&engine);
//! #     let res = Server::create(eref, server::Params::new(port));
//! #     if res.is_ok() {
//! #         let mut srv = res.unwrap();
//! #         let res = srv.start();
//! #         if res.is_ok() {
//! #             Some(srv)
//! #         } else {
//! #             None
//! #         }
//! #     } else {
//! #         None
//! #     }
//! # }
//! #
//! # fn client_proc(runtime: &runtime::Runtime, port: u16) -> extremedb::Result<()> {
//!     // let port = ...
//!
//!     let rsql = RemoteEngine::connect(&runtime, client::Params::new("localhost", port))?;
//!
//!     rsql.execute_statement("CREATE TABLE TestTable(i int, s string);", &[])?;
//!     rsql.execute_statement("INSERT INTO TestTable(i, s) VALUES(1, 'Hello');", &[])?;
//!     rsql.execute_statement("INSERT INTO TestTable(i, s) VALUES(2, 'World');", &[])?;
//! #
//! #     Ok(())
//! # }
//! #
//! # fn main() -> extremedb::Result<()> {
//! #     let runtime = Arc::new(runtime::Runtime::start(vec![]));
//! #     let mut db_params = database::Params::new();
//! #     db_params
//! #         .ddl_dict_size(32768)
//! #         .max_classes(100)
//! #         .max_indexes(1000);
//! #     let mut devs = util::DeviceContainer::new();
//! #     let db = database::Database::open(&runtime, "test_db", None, devs.devices(), db_params)?;
//! #     let conn = connection::Connection::new(&db)?;
//! #     let engine = sql::engine::LocalEngine::new(&conn)?;
//! #
//! #     let mut port = 25123;
//! #     let mut attempts = 10;
//! #
//! #     let mut srv = None;
//! #     while srv.is_none() && attempts > 0 {
//! #         srv = server_try_create(&engine, port);
//! #
//! #         if srv.is_none() {
//! #             port += 1;
//! #             attempts -= 1;
//! #         }
//! #     }
//! #
//! #     let mut srv = srv.expect("Failed to create the server");
//! #
//! #     let rt = runtime.clone();
//! #     let t_cli = thread::spawn(move || {
//! #         client_proc(&rt, port).expect("Client failed");
//! #     });
//! #
//! #     t_cli.join().expect("Failed to join the client thread");
//! #
//! #     srv.stop()?;
//! #
//! #     // Run some data tests in test environment:
//! #     let ds = engine.execute_query("SELECT COUNT(*) FROM TestTable;", &[])?;
//! #     assert!(ds.is_some());
//! #     let ds = ds.unwrap();
//! #
//! #     let mut cur = ds.cursor()?;
//! #     assert!(cur.advance()?);
//! #
//! #     let rec = cur.current_record();
//! #     assert!(rec.is_some());
//! #     let rec = rec.unwrap();
//! #
//! #     let val = rec.get_at(0)?;
//! #     assert_eq!(val.to_i64()?, 2);
//! #
//! #     Ok(())
//! # }
//! ```

use std::marker::PhantomData;
use std::mem::MaybeUninit;

use crate::runtime::Runtime;

use crate::sql::engine::Engine;
use crate::sql::{mcosql_error_code, result_from_code};
use crate::{exdb_sys, Result};

/// Client connection parameters.
pub struct Params {
    tx_buf_size: usize,
    host: String,
    port: u16,
    max_conn_attempts: u32,
}

impl Params {
    /// Creates a new parameters structure initialized with default values.
    pub fn new(host: &str, port: u16) -> Self {
        Params {
            tx_buf_size: 64 * 1024,
            host: host.to_string(),
            port,
            max_conn_attempts: 10,
        }
    }

    /// Sets the transmit buffer size.
    ///
    /// Default value is 64 KiB.
    pub fn tx_buf_size(&mut self, tx_buf_size: usize) -> &mut Self {
        self.tx_buf_size = tx_buf_size;
        self
    }

    /// Sets the server host name.
    pub fn host(&mut self, host: &str) -> &mut Self {
        self.host = host.to_string();
        self
    }

    /// Sets the server port.
    pub fn port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    /// Sets the maximum number of connection attempts.
    ///
    /// Default value is 10.
    pub fn max_conn_attempts(&mut self, max_conn_attempts: u32) -> &mut Self {
        self.max_conn_attempts = max_conn_attempts;
        self
    }
}

/// Remote SQL engine.
///
/// The remote SQL engine is used in a manner similar to the local engine.
/// When the connection to the server is established, it is possible to run
/// the SQL DDL and DML queries as usual.
///
/// However, it is impossible to create sessions using the remote engine, and
/// the transactions must be managed using the SQL transaction management
/// statements; [`Transaction`] objects cannot be used.
///
/// [`Transaction`]: ../../trans/struct.Transaction.html
pub struct RemoteEngine<'a> {
    runtime: PhantomData<&'a Runtime>,
    h: exdb_sys::database_t,
}

impl<'a> RemoteEngine<'a> {
    pub fn connect(_runtime: &'a Runtime, params: Params) -> Result<Self> {
        let mut h = MaybeUninit::uninit();

        result_from_code(unsafe {
            exdb_sys::sqlcln_create(h.as_mut_ptr(), params.tx_buf_size as exdb_sys::size_t)
        })?;

        let h = unsafe { h.assume_init() };

        result_from_code(unsafe {
            let rc = exdb_sys::sqlcln_open(
                h,
                params.host.as_ptr() as *const i8,
                params.port as i32,
                params.max_conn_attempts as i32,
            );
            if rc != mcosql_error_code::SQL_OK {
                let rc2 = exdb_sys::sqlcln_destroy(h);
                debug_assert_eq!(mcosql_error_code::SQL_OK, rc2);
            }

            rc
        })?;

        Ok(RemoteEngine {
            runtime: PhantomData,
            h,
        })
    }
}

impl<'a> Drop for RemoteEngine<'a> {
    fn drop(&mut self) {
        unsafe {
            let rc = exdb_sys::sqlcln_close(self.h);
            debug_assert_eq!(mcosql_error_code::SQL_OK, rc);
            let rc = exdb_sys::sqlcln_destroy(self.h);
            debug_assert_eq!(mcosql_error_code::SQL_OK, rc);
        }
    }
}

impl<'a> Engine for RemoteEngine<'a> {
    fn get_engine(&self) -> exdb_sys::database_t {
        self.h
    }
}
