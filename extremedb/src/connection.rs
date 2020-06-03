// connection.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

//! Database connection management.
//!
//! A connection must be established to perform operations on the database,
//! including creating an SQL engine.
//!
//! # Example
//!
//! Connecting to an existing database instance:
//!
//! ```
//! # use extremedb::{connection, database, device, runtime, Result};
//! # use extremedb::device::util;
//! #
//! # fn main() -> Result<()> {
//! #     let runtime = runtime::Runtime::start(vec![]);
//! #
//! #     let mut devs = util::DeviceContainer::new();
//! #
//!     let db = database::Database::open(
//! #         &runtime,
//! #         "test_db",
//! #         None,
//! #         devs.devices(),
//! #         database::Params::new(),
//!         // ...
//!     )?;
//!
//!     let conn = connection::Connection::new(&db)?;
//!
//!     // ...
//! #
//! #     drop(conn);
//! #     drop(db);
//! #     drop(runtime);
//! #
//! #     Ok(())
//! # }
//! ```

use std::marker::PhantomData;
use std::mem::MaybeUninit;

use crate::database::Database;
use crate::{exdb_sys, mco_ret, result_from_code, Result};

/// A database connection.
///
/// Connections are neither `Sync` nor `Send`; they must be used only by
/// the threads that create them.
///
/// A connection is closed when it is dropped.
pub struct Connection<'a> {
    db: PhantomData<&'a Database<'a>>,
    pub(crate) h: exdb_sys::mco_db_h,
}

impl<'a> Connection<'a> {
    /// Establishes a new connection to `db`.
    pub fn new(db: &'a Database) -> Result<Self> {
        let mut h = MaybeUninit::uninit();

        result_from_code(unsafe { exdb_sys::mco_db_connect(db.name().as_ptr(), h.as_mut_ptr()) })?;

        Ok(Connection {
            db: PhantomData,
            h: unsafe { h.assume_init() },
        })
    }

    pub(crate) unsafe fn handle(&self) -> exdb_sys::mco_db_h {
        self.h
    }
}

impl<'a> Drop for Connection<'a> {
    fn drop(&mut self) {
        let rc = unsafe { exdb_sys::mco_db_disconnect(self.h) };
        debug_assert_eq!(mco_ret::MCO_S_OK, rc);
    }
}
