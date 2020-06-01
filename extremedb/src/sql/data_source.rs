// data_source.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

//! Data source returned by queries.
//!
//! Execution of a query produces a *data source*, which is used to access
//! rows of data returned by the query. Since the underlying *e*X*treme*DB C++
//! SQL API returns a nullable pointer, the data sources returned by the query
//! execution methods are wrapped in an `Option`.
//!
//! # Cursors
//!
//! A *cursor* is used to iterate over the *records* in the data source.
//! Although it is similar to the standard library's iterator, it is impossible
//! to implement the `Iterator` trait on it. The records produced by the cursor
//! are invalidated when the cursor is advanced, and cannot outlive the cursor
//! or the data source. This contradicts the standard `Iterator`'s semantics.
//!
//! # Examples
//!
//! Execute a `SELECT` query to obtain a data source:
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
//! #
//!     engine.execute_statement("CREATE TABLE TestTable(i integer, s string);", &[])?;
//!     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&1, &"Hello"])?;
//!     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&2, &"World"])?;
//!
//!     let ds = engine.execute_query("SELECT i, s FROM TestTable ORDER BY i;", &[])?;
//!     assert!(ds.is_some());
//!     let ds = ds.unwrap();
//! #     drop(ds);
//! #     Ok(())
//! # }
//! ```
//!
//! A cursor can be used to iterate over the records. Notice that initially
//! the cursor is positioned before the first element, so it has to be advanced
//! prior to accessing the first record:
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
//! #     engine.execute_statement("CREATE TABLE TestTable(i integer, s string);", &[])?;
//! #     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&1, &"Hello"])?;
//! #     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&2, &"World"])?;
//! #     let ds = engine.execute_query("SELECT i, s FROM TestTable ORDER BY i;", &[])?;
//! #     assert!(ds.is_some());
//! #     let ds = ds.unwrap();
//!     let mut cur = ds.cursor()?;
//!
//!     assert_eq!(cur.advance()?, true);
//!
//!     let rec = cur.current_record();
//!     assert!(rec.is_some());
//!
//!     let rec = rec.unwrap();
//! #     drop(rec);
//! #     Ok(())
//! # }
//! ```
//!
//! Individual column values in a record can be accessed using the column
//! numbers:
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
//! #     engine.execute_statement("CREATE TABLE TestTable(i integer, s string);", &[])?;
//! #     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&1, &"Hello"])?;
//! #     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&2, &"World"])?;
//! #     let ds = engine.execute_query("SELECT i, s FROM TestTable ORDER BY i;", &[])?;
//! #     assert!(ds.is_some());
//! #     let ds = ds.unwrap();
//! #     let mut cur = ds.cursor()?;
//! #     assert_eq!(cur.advance()?, true);
//! #     let rec = cur.current_record();
//! #     assert!(rec.is_some());
//! #     let rec = rec.unwrap();
//!     let i_val = rec.get_at(0)?;
//!     let s_val = rec.get_at(1)?;
//!
//!     assert_eq!(i_val.to_i64()?, 1);
//!     assert_eq!(s_val.to_string()?, "Hello");
//! #
//! #     Ok(())
//! # }
//! ```

use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr;

use crate::sql::value::{Ref, Type};
use crate::sql::{mcosql_error_code, result_from_code};
use crate::{exdb_sys, Error, Result};

/// A data source.
///
/// A data source contains records (table rows) produced by a query.
pub struct DataSource<'a> {
    owner: PhantomData<&'a ()>,
    h: exdb_sys::data_source_t,
}

impl<'a> DataSource<'a> {
    pub(crate) fn new<T: ?Sized>(h: exdb_sys::data_source_t, _owner: &'a T) -> Self {
        DataSource {
            owner: PhantomData,
            h,
        }
    }

    /// Returns the number of columns in this data source.
    pub fn n_columns(&self) -> Result<usize> {
        let mut ret = MaybeUninit::uninit();

        result_from_code(unsafe {
            exdb_sys::mcosql_get_number_of_columns(self.h, ret.as_mut_ptr())
        })
        .and(Ok(unsafe { ret.assume_init() }))
    }

    /// Returns the type and the name of the column at index `col`.
    pub fn column_info(&self, col: usize) -> Result<(Type, String)> {
        let mut mco_ty = MaybeUninit::uninit();
        let mut pname = MaybeUninit::uninit();

        result_from_code(unsafe {
            exdb_sys::mcosql_get_column_info(self.h, col, mco_ty.as_mut_ptr(), pname.as_mut_ptr())
        })?;

        let ty = Type::from_mco(unsafe { mco_ty.assume_init() })
            .ok_or(Error::new_sql(mcosql_error_code::RUNTIME_ERROR))?;
        let cname = unsafe { CStr::from_ptr(pname.assume_init()) };

        let name = cname
            .to_str()
            .or(Err(Error::new_sql(mcosql_error_code::RUNTIME_ERROR)))?;

        Ok((ty, name.to_string()))
    }

    /// Creates a cursor for this data source.
    pub fn cursor(&self) -> Result<Cursor> {
        let mut cur = MaybeUninit::uninit();

        result_from_code(unsafe { exdb_sys::mcosql_get_cursor(self.h, cur.as_mut_ptr()) })
            .and(Ok(Cursor::new(self, unsafe { cur.assume_init() })))
    }
}

impl<'a> Drop for DataSource<'a> {
    fn drop(&mut self) {
        let rc = unsafe { exdb_sys::mcosql_release_query_result(self.h) };
        debug_assert_eq!(mcosql_error_code::SQL_OK, rc);
    }
}

/// A cursor.
///
/// A cursor is used to iterate over the records in a data source. It is
/// initially positioned before the first item.
pub struct Cursor<'a> {
    source: PhantomData<&'a DataSource<'a>>,
    h: exdb_sys::cursor_t,
    rec_h: exdb_sys::record_t,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(_source: &'a DataSource, h: exdb_sys::cursor_t) -> Self {
        Cursor {
            source: PhantomData,
            h,
            rec_h: ptr::null_mut(),
        }
    }

    /// Advances the cursor.
    ///
    /// If this function returns `true`, the current record can be accessed.
    /// `false` indicates that the cursor has been moved past the last record.
    pub fn advance(&mut self) -> Result<bool> {
        let rc = unsafe { exdb_sys::mcosql_cursor_move_next(self.h, &mut self.rec_h) };

        match rc {
            mcosql_error_code::SQL_OK => Ok(true),
            mcosql_error_code::NO_MORE_ELEMENTS => {
                self.rec_h = ptr::null_mut();
                Ok(false)
            }
            _ => {
                self.rec_h = ptr::null_mut();
                Err(Error::new_sql(rc))
            }
        }
    }

    /// Returns the record currently pointed at by the cursor.
    ///
    /// Returns `None` if the cursor hasn't been advanced at least once, or has
    /// reached the end of the data source.
    pub fn current_record(&self) -> Option<Record> {
        if self.rec_h.is_null() {
            None
        } else {
            Some(Record::new(self, self.rec_h))
        }
    }
}

/// A record.
///
/// Records are the actual rows of data produced by a `SELECT` SQL query.
pub struct Record<'a> {
    cursor: PhantomData<&'a Cursor<'a>>,
    h: exdb_sys::record_t,
}

impl<'a> Record<'a> {
    pub(crate) fn new(_cursor: &'a Cursor, h: exdb_sys::record_t) -> Self {
        Record {
            cursor: PhantomData,
            h,
        }
    }

    /*
    fn allocator(&'a self) -> Result<SqlAllocatorRef<'a>> {
        let mut alloc_h = MaybeUninit::uninit();

        new_empty_result(unsafe {
            exdb_sys::mcors_sql_record_allocator(self.h, alloc_h.as_mut_ptr())
        })
        .and(Ok(SqlAllocatorRef::from_handle(
            unsafe { alloc_h.assume_init() },
            self,
        )))
    }
    */

    /// Returns a reference to the value in the column `col`.
    pub fn get_at(&self, col: usize) -> Result<Ref> {
        let mut ret = MaybeUninit::uninit();

        result_from_code(unsafe {
            exdb_sys::mcors_sql_record_get_column_value_ref(self.h, col, ret.as_mut_ptr())
        })
        .and(Ok(Ref::from_handle(unsafe { ret.assume_init() }, self)))
    }
}
