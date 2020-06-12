// server.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

//! Remote SQL server functionality.
//!
//! This module implements the remote SQL server. The server is bound to a
//! local SQL engine instance, and can be used by the remote clients to
//! operate on the database.
//!
//! # Examples
//!
//! Creating, starting, and stopping an RSQL server bound to a local engine:
//!
//! ```
//! # use extremedb::sql::engine::{LocalEngine, LocalEngineRef};
//! # use extremedb::sql::rsql::server::{self, Server};
//! # use extremedb::{connection, database, device, runtime, sql};
//! # use extremedb::device::util;
//! # fn try_create_server(engine: &LocalEngine, port: u16) -> extremedb::Result<()> {
//!     let eref = LocalEngineRef::new(&engine);
//!
//!     // let port = ...;
//!     let mut srv = Server::create(eref, server::Params::new(port))?;
//!     srv.start()?;
//!
//!     // Now the server is ready to accept and handle the client connections...
//!
//!     srv.stop()?;
//! #
//! #     Ok(())
//! # }
//! #
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
//! #
//! #     let port = 25321u16;
//! #     let attempts = 10u16;
//! #     let mut success = false;
//! #
//! #     for attempt in 0..attempts {
//! #         if try_create_server(&engine, port + attempt).is_ok() {
//! #             success = true;
//! #             break;
//! #         }
//! #     }
//! #
//! #     // Expect success in test environment.
//! #     assert!(success);
//! #
//! #     Ok(())
//! # }
//! ```

use std::marker::PhantomData;
use std::mem::MaybeUninit;

use crate::sql::engine::LocalEngineRef;
use crate::sql::{mcosql_error_code, result_from_code};
use crate::{exdb_sys, Result};

/// Server parameters.
pub struct Params {
    port: u16,
    buf_size: usize,
    threads: usize,
    listen_queue_size: usize,
}

impl Params {
    /// Creates a new parameters structure initialized with default values.
    pub fn new(port: u16) -> Self {
        Params {
            port,
            buf_size: 64 * 1024,
            threads: 8,
            listen_queue_size: 5,
        }
    }

    /// Sets the port.
    pub fn port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    /// Sets the buffer size.
    pub fn buf_size(&mut self, buf_size: usize) -> &mut Self {
        self.buf_size = buf_size;
        self
    }

    /// Sets the number of worker threads.
    pub fn threads(&mut self, threads: usize) -> &mut Self {
        self.threads = threads;
        self
    }

    /// Sets the listen queue size.
    pub fn listen_queue_size(&mut self, listen_queue_size: usize) -> &mut Self {
        self.listen_queue_size = listen_queue_size;
        self
    }
}

/// Remote SQL server.
///
/// The remote SQL server binds to the local SQL engine and lets the remote
/// clients connect and execute SQL statements.
pub struct Server<'a> {
    h: exdb_sys::sqlsrv_t,
    engine: PhantomData<LocalEngineRef<'a>>,
}

impl<'a> Server<'a> {
    /// Creates a new server.
    ///
    /// The newly created server has to be started explicitly using the
    /// [`start()`] method.
    ///
    /// [`start()`]: #method.start
    pub fn create(engine: LocalEngineRef<'a>, params: Params) -> Result<Self> {
        let mut h = MaybeUninit::uninit();

        result_from_code(unsafe {
            exdb_sys::sqlsrv_create(
                h.as_mut_ptr(),
                engine.h as exdb_sys::storage_t,
                params.port as i32,
                params.buf_size as exdb_sys::size_t,
                params.threads as exdb_sys::size_t,
                params.listen_queue_size as i32,
                None,
            )
        })?;

        Ok(Server {
            h: unsafe { h.assume_init() },
            engine: PhantomData,
        })
    }

    /// Starts the server.
    ///
    /// This call does not block and returns immediately. Upon successful
    /// start, the server threads proceed working in the background.
    pub fn start(&mut self) -> Result<()> {
        result_from_code(unsafe { exdb_sys::sqlsrv_start(self.h) })
    }

    /// Stops the server.
    ///
    /// This method is expected to be called from the same thread that started
    /// the server.
    pub fn stop(&mut self) -> Result<()> {
        result_from_code(unsafe { exdb_sys::sqlsrv_stop(self.h) })
    }
}

impl<'a> Drop for Server<'a> {
    fn drop(&mut self) {
        let rc = unsafe { exdb_sys::sqlsrv_destroy(self.h) };
        debug_assert_eq!(mcosql_error_code::SQL_OK, rc);
    }
}
