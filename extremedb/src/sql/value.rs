// value.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

//! SQL values.
//!
//! This module implements the types necessary for passing the values to and
//! from the *e*X*treme*DB SQL engine.
//!
//! # Passing Values to the SQL Engine
//!
//! The easiest way to pass values to queries and statements is to embed them
//! in the SQL code itself:
//!
//! `INSERT INTO TestTable VALUES(1, 'SomeString');`
//!
//! However, this is often inconvenient. Furthermore, this approach requires the
//! arguments to be sanitized to prevent trivial SQL injection attacks.
//!
//! Another approach is to pass the values as query parameters. The actual
//! values in the SQL code are replaced with placeholders (`?`), and the
//! arguments are passed by reference:
//!
//! ```
//! # use extremedb::connection::Connection;
//! # use extremedb::database::{Database, Params};
//! # use extremedb::device::{Assignment, Device};
//! # use extremedb::runtime::Runtime;
//! # use extremedb::sql::engine::{Engine, LocalEngine};
//! # use extremedb::Result;
//! # use extremedb::device::util;
//! # fn main() -> Result<()> {
//! #     let runtime = Runtime::start(vec![]);
//! #     let mut db_params = Params::new();
//! #     db_params
//! #         .ddl_dict_size(32768)
//! #         .max_classes(100)
//! #         .max_indexes(1000);
//! #     let mut devs = util::DeviceContainer::new();
//! #     let db = Database::open(&runtime, "test_db", None, devs.devices(), db_params)?;
//! #     let conn = Connection::new(&db)?;
//! #     let engine = LocalEngine::new(&conn)?;
//!     engine.execute_statement("CREATE TABLE TestTable(i integer, s string);", &[])?;
//!     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&1, &"Hello"])?;
//!     engine.execute_statement("INSERT INTO TestTable(i, s) VALUES(?, ?);", &[&2, &"World"])?;
//! #     Ok(())
//! # }
//! ```
//!
//! Any type that implements the [`ToValue`] trait can be passed as a query
//! argument. This module provides implementations of this trait for many
//! common Rust types, as well as some helper types.
//!
//! ## Basic Types
//!
//! Basic types (booleans, integer and real numbers) are simply passed by
//! reference:
//!
//! ```
//! # use extremedb::connection::Connection;
//! # use extremedb::database::{Database, Params};
//! # use extremedb::device::{Assignment, Device};
//! # use extremedb::runtime::Runtime;
//! # use extremedb::sql::engine::{Engine, LocalEngine};
//! # use extremedb::Result;
//! # use extremedb::device::util;
//! # fn main() -> Result<()> {
//! #     let runtime = Runtime::start(vec![]);
//! #     let mut db_params = Params::new();
//! #     db_params
//! #         .ddl_dict_size(32768)
//! #         .max_classes(100)
//! #         .max_indexes(1000);
//! #     let mut devs = util::DeviceContainer::new();
//! #     let db = Database::open(&runtime, "test_db", None, devs.devices(), db_params)?;
//! #     let conn = Connection::new(&db)?;
//! #     let engine = LocalEngine::new(&conn)?;
//!     engine.execute_statement(
//!         "CREATE TABLE TestTable(i integer, f float, b boolean);",
//!         &[],
//!     )?;
//!     engine.execute_statement(
//!         "INSERT INTO TestTable(i, f, b) VALUES(?, ?, ?);",
//!         &[&1, &123.45, &false],
//!     )?;
//! #     Ok(())
//! # }
//! ```
//!
//! ## Timestamps
//!
//! Timestamps can be passed as integers, as well as `std::time::SystemTime`.
//! For example, this will create two rows with identical values:
//!
//! ```
//! # use extremedb::connection::Connection;
//! # use extremedb::database::{Database, Params};
//! # use extremedb::device::{Assignment, Device};
//! # use extremedb::runtime::Runtime;
//! # use extremedb::sql::engine::{Engine, LocalEngine};
//! # use extremedb::Result;
//! # use extremedb::device::util;
//! # use std::time::{self, SystemTime};
//! # fn main() -> Result<()> {
//! #     let runtime = Runtime::start(vec![]);
//! #     let mut db_params = Params::new();
//! #     db_params
//! #         .ddl_dict_size(32768)
//! #         .max_classes(100)
//! #         .max_indexes(1000);
//! #     let mut devs = util::DeviceContainer::new();
//! #     let db = Database::open(&runtime, "test_db", None, devs.devices(), db_params)?;
//! #     let conn = Connection::new(&db)?;
//! #     let engine = LocalEngine::new(&conn)?;
//!     engine.execute_statement("CREATE TABLE TestTable(ts timestamp);", &[])?;
//!
//!     let now = SystemTime::now();
//!     let now_int = now.duration_since(time::UNIX_EPOCH).unwrap().as_secs();
//!
//!     engine.execute_statement("INSERT INTO TestTable VALUES(?), (?);", &[&now, &now_int])?;
//! #     Ok(())
//! # }
//! ```
//!
//! ## Fixed-Width Numeric Values
//!
//! *e*X*treme*DB supports fixed-width numeric values. This module provides
//! a wrapper type, [`Numeric`], to handle them. A fixed-width value `12.345`
//! could be inserted like this:
//!
//! ```
//! # use extremedb::connection::Connection;
//! # use extremedb::database::{Database, Params};
//! # use extremedb::device::{Assignment, Device};
//! # use extremedb::runtime::Runtime;
//! # use extremedb::sql::engine::{Engine, LocalEngine};
//! # use extremedb::sql::value::Numeric;
//! # use extremedb::Result;
//! # use extremedb::device::util;
//! # fn main() -> Result<()> {
//! #     let runtime = Runtime::start(vec![]);
//! #     let mut db_params = Params::new();
//! #     db_params
//! #         .ddl_dict_size(32768)
//! #         .max_classes(100)
//! #         .max_indexes(1000);
//! #     let mut devs = util::DeviceContainer::new();
//! #     let db = Database::open(&runtime, "test_db", None, devs.devices(), db_params)?;
//! #     let conn = Connection::new(&db)?;
//! #     let engine = LocalEngine::new(&conn)?;
//!     engine.execute_statement("CREATE TABLE TestTable(n numeric(5, 3));", &[])?;
//!
//!     engine.execute_statement(
//!         "INSERT INTO TestTable VALUES(?);",
//!         &[&Numeric::new(12345, 3)],
//!     )?;
//! #     Ok(())
//! # }
//! ```
//!
//! ## Nullable Values
//!
//! Nullable values of type `T` can be passed as `Option<T>`:
//!
//! ```
//! # use extremedb::connection::Connection;
//! # use extremedb::database::{Database, Params};
//! # use extremedb::device::{Assignment, Device};
//! # use extremedb::runtime::Runtime;
//! # use extremedb::sql::engine::{Engine, LocalEngine};
//! # use extremedb::Result;
//! # use extremedb::device::util;
//! # fn main() -> Result<()> {
//! #     let runtime = Runtime::start(vec![]);
//! #     let mut db_params = Params::new();
//! #     db_params
//! #         .ddl_dict_size(32768)
//! #         .max_classes(100)
//! #         .max_indexes(1000);
//! #     let mut devs = util::DeviceContainer::new();
//! #     let db = Database::open(&runtime, "test_db", None, devs.devices(), db_params)?;
//! #     let conn = Connection::new(&db)?;
//! #     let engine = LocalEngine::new(&conn)?;
//!     engine.execute_statement("CREATE TABLE TestTable(i int);", &[])?;
//!
//!     let non_null = Some(1);
//!     let null: Option<i32> = None;
//!
//!     engine.execute_statement("INSERT INTO TestTable VALUES(?), (?);", &[&non_null, &null])?;
//! #     Ok(())
//! # }
//! ```
//!
//! ## Strings and Binary
//!
//! Strings and binary values can be passed as string slices and [`Binary`],
//! respectively. Note that it is necessary to use the `Binary` helper type
//! instead of `u8` slices: the latter are converted to [`Array`] instead.
//!
//! ```
//! # use extremedb::connection::Connection;
//! # use extremedb::database::{Database, Params};
//! # use extremedb::device::{Assignment, Device};
//! # use extremedb::runtime::Runtime;
//! # use extremedb::sql::engine::{Engine, LocalEngine};
//! # use extremedb::sql::value::Binary;
//! # use extremedb::Result;
//! # use extremedb::device::util;
//! # fn main() -> Result<()> {
//! #     let runtime = Runtime::start(vec![]);
//! #     let mut db_params = Params::new();
//! #     db_params
//! #         .ddl_dict_size(32768)
//! #         .max_classes(100)
//! #         .max_indexes(1000);
//! #     let mut devs = util::DeviceContainer::new();
//! #     let db = Database::open(&runtime, "test_db", None, devs.devices(), db_params)?;
//! #     let conn = Connection::new(&db)?;
//! #     let engine = LocalEngine::new(&conn)?;
//!     engine.execute_statement("CREATE TABLE TestTable(s string, b varbinary);", &[])?;
//!
//!     let s = "A string";
//!     let b = Binary::new(b"Some binary data");
//!
//!     engine.execute_statement("INSERT INTO TestTable(s, b) VALUES(?, ?);", &[&s, &b])?;
//! #     Ok(())
//! # }
//! ```
//!
//! ## Arrays, Sequences, and Blobs
//!
//! Arrays, as well as sequences and blobs, are passed as slices of the
//! appropriate types:
//!
//! ```
//! # use extremedb::connection::Connection;
//! # use extremedb::database::{Database, Params};
//! # use extremedb::device::{Assignment, Device};
//! # use extremedb::runtime::Runtime;
//! # use extremedb::sql::engine::{Engine, LocalEngine};
//! # use extremedb::Result;
//! # use extremedb::device::util;
//! # fn main() -> Result<()> {
//! #     let runtime = Runtime::start(vec![]);
//! #     let mut db_params = Params::new();
//! #     db_params
//! #         .ddl_dict_size(32768)
//! #         .max_classes(100)
//! #         .max_indexes(1000);
//! #     let mut devs = util::DeviceContainer::new();
//! #     let db = Database::open(&runtime, "test_db", None, devs.devices(), db_params)?;
//! #     let conn = Connection::new(&db)?;
//! #     let engine = LocalEngine::new(&conn)?;
//!     engine.execute_statement("CREATE TABLE TestTable(a array(int), b blob);", &[])?;
//!
//!     let array: &[u32] = &[1, 2, 3, 4, 5];
//!     let blob: &[u8] = &[0x01, 0x02, 0x03, 0x04, 0x05];
//!
//!     engine.execute_statement(
//!         "INSERT INTO TestTable(a, b) VALUES(?, ?);",
//!         &[&array, &blob],
//!     )?;
//! #     Ok(())
//! # }
//! ```
//!
//! [`ToValue`]: ./trait.ToValue.html
//! [`Numeric`]: ./struct.Numeric.html
//! [`Binary`]: ./struct.Binary.html
//!
//! # Receiving Values from the SQL Engine
//!
//! `SELECT` SQL queries usually produce *data sources*, which contain records
//! (or rows). The records, in turn, contain *fields*, corresponding to the
//! columns of the original SQL query.
//!
//! The contents of these fields are returned as *value references*, or
//! instances of [`Ref`], whose lifetimes are bounded by their containing
//! records' lifetimes. This means that the value references are transitional
//! data structures, and the applications are not expected to try and retain
//! them. Instead, they should convert the references to the native data types
//! right away.
//!
//! [`Ref`] dereferences to [`Value`], which is a wrapper for the SQL engine's
//! generic value type. It has methods that return the type of the contained
//! value, as well as methods that convert it to native data types. The
//! applications are expected to inspect the inner value type and call the
//! appropriate conversion method.
//!
//! ## Basic Types
//!
//! Values can be converted to basic types using the appropriate conversion
//! methods. Notice that all integer values are stored internally as `i64`
//! by the SQL engine, and hence there is only one integer conversion method.
//!
//! ```
//! # use extremedb::connection::Connection;
//! # use extremedb::database::{Database, Params};
//! # use extremedb::device::{Assignment, Device};
//! # use extremedb::runtime::Runtime;
//! # use extremedb::sql::engine::{Engine, LocalEngine};
//! # use extremedb::sql::value;
//! # use extremedb::Result;
//! # use extremedb::device::util;
//! # fn use_value(val: value::Ref) -> Result<()> {
//! #    assert_eq!(val.value_type()?, value::Type::Int8);
//!     if val.value_type()? == value::Type::Int8 {
//!         let i = val.to_i64()?;
//!         assert_eq!(i, 1);
//!     }
//! #
//! #     Ok(())
//! # }
//! #
//! # fn main() -> Result<()> {
//! #     let runtime = Runtime::start(vec![]);
//! #     let mut db_params = Params::new();
//! #     db_params
//! #         .ddl_dict_size(32768)
//! #         .max_classes(100)
//! #         .max_indexes(1000);
//! #     let mut devs = util::DeviceContainer::new();
//! #     let db = Database::open(&runtime, "test_db", None, devs.devices(), db_params)?;
//! #     let conn = Connection::new(&db)?;
//! #     let engine = LocalEngine::new(&conn)?;
//! #     engine.execute_statement("CREATE TABLE TestTable(i int);", &[])?;
//! #
//! #     engine.execute_statement("INSERT INTO TestTable VALUES(1);", &[])?;
//! #
//! #     let ds = engine.execute_query("SELECT i FROM TestTable;", &[])?;
//! #     assert!(ds.is_some());
//! #     let ds = ds.unwrap();
//! #     let mut cur = ds.cursor()?;
//! #     {
//! #         assert_eq!(cur.advance()?, true);
//! #         assert!(cur.current_record().is_some());
//! #         let rec = cur.current_record().unwrap();
//! #         let val = rec.get_at(0)?;
//! #         use_value(val)?;
//! #     }
//! #     Ok(())
//! # }
//! ```
//!
//! ## Type Conversions
//!
//! SQL engine allows conversions between values of compatible types.
//! Furthermore, most values can be converted to strings. For more details,
//! refer to the *e*X*treme*DB C++ SQL API reference pages.
//!
//! ```
//! # use extremedb::connection::Connection;
//! # use extremedb::database::{Database, Params};
//! # use extremedb::device::{Assignment, Device};
//! # use extremedb::runtime::Runtime;
//! # use extremedb::sql::engine::{Engine, LocalEngine};
//! # use extremedb::sql::value;
//! # use extremedb::Result;
//! # use extremedb::device::util;
//! # fn use_value(val: value::Ref) -> Result<()> {
//! #    assert_eq!(val.value_type()?, value::Type::Int8);
//!     if val.value_type()? == value::Type::Int8 {
//!         let i = val.to_i64()?;
//!         assert_eq!(i, 1);
//!         let f = val.to_real()?;
//!         assert_eq!(f, 1.0);
//!         let s = val.to_string()?;
//!         assert_eq!(s, "1");
//!     }
//! #
//! #     Ok(())
//! # }
//! #
//! # fn main() -> Result<()> {
//! #     let runtime = Runtime::start(vec![]);
//! #     let mut db_params = Params::new();
//! #     db_params
//! #         .ddl_dict_size(32768)
//! #         .max_classes(100)
//! #         .max_indexes(1000);
//! #     let mut devs = util::DeviceContainer::new();
//! #     let db = Database::open(&runtime, "test_db", None, devs.devices(), db_params)?;
//! #     let conn = Connection::new(&db)?;
//! #     let engine = LocalEngine::new(&conn)?;
//! #     engine.execute_statement("CREATE TABLE TestTable(i int);", &[])?;
//! #
//! #     engine.execute_statement("INSERT INTO TestTable VALUES(1);", &[])?;
//! #
//! #     let ds = engine.execute_query("SELECT i FROM TestTable;", &[])?;
//! #     assert!(ds.is_some());
//! #     let ds = ds.unwrap();
//! #     let mut cur = ds.cursor()?;
//! #     {
//! #         assert_eq!(cur.advance()?, true);
//! #         assert!(cur.current_record().is_some());
//! #         let rec = cur.current_record().unwrap();
//! #         let val = rec.get_at(0)?;
//! #         use_value(val)?;
//! #     }
//! #     Ok(())
//! # }
//! ```
//!
//! ## Getting References to the String and Binary Data
//!
//! String and binary conversion methods return owned values that contain
//! copies of the value's data. This extra copying is not always desired,
//! and can be avoided by using methods that return references to the
//! underlying data.
//!
//! ```
//! # use extremedb::connection::Connection;
//! # use extremedb::database::{Database, Params};
//! # use extremedb::device::{Assignment, Device};
//! # use extremedb::runtime::Runtime;
//! # use extremedb::sql::engine::{Engine, LocalEngine};
//! # use extremedb::sql::value;
//! # use extremedb::Result;
//! # use extremedb::device::util;
//! # fn use_value(val: value::Ref) -> Result<()> {
//! #     assert_eq!(val.value_type()?, value::Type::String);
//!     if val.value_type()? == value::Type::String {
//!         let s = val.as_str()?;
//!         assert_eq!(s, "Some string");
//!     }
//! #
//! #     Ok(())
//! # }
//! #
//! # fn main() -> Result<()> {
//! #     let runtime = Runtime::start(vec![]);
//! #     let mut db_params = Params::new();
//! #     db_params
//! #         .ddl_dict_size(32768)
//! #         .max_classes(100)
//! #         .max_indexes(1000);
//! #     let mut devs = util::DeviceContainer::new();
//! #     let db = Database::open(&runtime, "test_db", None, devs.devices(), db_params)?;
//! #     let conn = Connection::new(&db)?;
//! #     let engine = LocalEngine::new(&conn)?;
//! #     engine.execute_statement("CREATE TABLE TestTable(s string);", &[])?;
//! #
//! #     engine.execute_statement("INSERT INTO TestTable VALUES('Some string');", &[])?;
//! #
//! #     let ds = engine.execute_query("SELECT s FROM TestTable;", &[])?;
//! #     assert!(ds.is_some());
//! #     let ds = ds.unwrap();
//! #     let mut cur = ds.cursor()?;
//! #     {
//! #         assert_eq!(cur.advance()?, true);
//! #         assert!(cur.current_record().is_some());
//! #         let rec = cur.current_record().unwrap();
//! #         let val = rec.get_at(0)?;
//! #         use_value(val)?;
//! #     }
//! #     Ok(())
//! # }
//! ```
//!
//! ## Arrays, Sequences, and Blobs
//!
//! Generic values that contain arrays, sequences, and blobs have to be
//! converted to the appropriate specific types — [`Array`], [`Sequence`],
//! and [`Blob`], respectively. For example, an array containing values `1`,
//! `2`, and `3`, could be handled like this:
//!
//! ```
//! # use extremedb::connection::Connection;
//! # use extremedb::database::{Database, Params};
//! # use extremedb::device::{Assignment, Device};
//! # use extremedb::runtime::Runtime;
//! # use extremedb::sql::engine::{Engine, LocalEngine};
//! # use extremedb::sql::value;
//! # use extremedb::Result;
//! # use extremedb::device::util;
//! # fn use_value(val: value::Ref) -> Result<()> {
//! #     assert_eq!(val.value_type()?, value::Type::Array);
//!     if val.value_type()? == value::Type::Array {
//!         let arr = val.as_array()?;
//! #         assert_eq!(arr.elem_type()?, value::Type::Int4);
//! #         assert_eq!(arr.len()?, 3);
//!         if arr.elem_type()? == value::Type::Int4 {
//!             assert_eq!(arr.get_at(0)?.to_i64()?, 1);
//!             assert_eq!(arr.get_at(1)?.to_i64()?, 2);
//!             assert_eq!(arr.get_at(2)?.to_i64()?, 3);
//!         }
//!     }
//! #
//! #     Ok(())
//! # }
//! #
//! # fn main() -> Result<()> {
//! #     let runtime = Runtime::start(vec![]);
//! #     let mut db_params = Params::new();
//! #     db_params
//! #         .ddl_dict_size(32768)
//! #         .max_classes(100)
//! #         .max_indexes(1000);
//! #     let mut devs = util::DeviceContainer::new();
//! #     let db = Database::open(&runtime, "test_db", None, devs.devices(), db_params)?;
//! #     let conn = Connection::new(&db)?;
//! #     let engine = LocalEngine::new(&conn)?;
//! #     engine.execute_statement("CREATE TABLE TestTable(a array(int));", &[])?;
//! #
//! #     engine.execute_statement("INSERT INTO TestTable VALUES([1, 2, 3]);", &[])?;
//! #
//! #     let ds = engine.execute_query("SELECT a FROM TestTable;", &[])?;
//! #     assert!(ds.is_some());
//! #     let ds = ds.unwrap();
//! #     let mut cur = ds.cursor()?;
//! #     {
//! #         assert_eq!(cur.advance()?, true);
//! #         assert!(cur.current_record().is_some());
//! #         let rec = cur.current_record().unwrap();
//! #         let val = rec.get_at(0)?;
//! #         use_value(val)?;
//! #     }
//! #     Ok(())
//! # }
//! ```
//!
//! [`Ref`]: ./struct.Ref.html
//! [`Value`]: ./struct.Value.html
//! [`Array`]: ./struct.Array.html
//! [`Sequence`]: ./struct.Sequence.html
//! [`Blob`]: ./struct.Blob.html
//!

use std::convert::TryFrom;
use std::ffi::c_void;
use std::fmt::{Display, Error as FmtError, Formatter};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::ptr;
use std::slice;
use std::str;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::runtime::options;
use crate::sql::allocator::{self, Ref as AllocatorRef};
use crate::sql::{mcosql_error_code, result_from_code};
use crate::{exdb_sys, Error, Result};

use exdb_sys::mcosql_column_type;

/// The type of a generic SQL value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Type {
    /// A `null` value.
    Null = mcosql_column_type::CT_NULL as isize,

    /// A boolean value.
    Bool = mcosql_column_type::CT_BOOL as isize,

    /// A signed 8-bit integer.
    Int1 = mcosql_column_type::CT_INT1 as isize,

    /// A signed 16-bit integer.
    Int2 = mcosql_column_type::CT_INT2 as isize,

    /// A signed 32-bit integer.
    Int4 = mcosql_column_type::CT_INT4 as isize,

    /// A signed 64-bit integer.
    Int8 = mcosql_column_type::CT_INT8 as isize,

    /// An unsigned 8-bit integer.
    UInt1 = mcosql_column_type::CT_UINT1 as isize,

    /// An unsigned 16-bit integer.
    UInt2 = mcosql_column_type::CT_UINT2 as isize,

    /// An unsigned 32-bit integer.
    UInt4 = mcosql_column_type::CT_UINT4 as isize,

    /// An unsigned 64-bit integer.
    UInt8 = mcosql_column_type::CT_UINT8 as isize,

    /// A 32-bit floating point value.
    Real4 = mcosql_column_type::CT_REAL4 as isize,

    /// A 64-bit floating point value.
    Real8 = mcosql_column_type::CT_REAL8 as isize,

    /// A timestamp.
    Time = mcosql_column_type::CT_TIME as isize,

    /// A fixed-width numeric value.
    Numeric = mcosql_column_type::CT_NUMERIC as isize,

    // Unicode = mcosql_column_type::CT_UNICODE as isize,
    /// A variable-length string.
    String = mcosql_column_type::CT_STRING as isize,

    /// A variable-length byte array.
    Binary = mcosql_column_type::CT_BINARY as isize,

    // Reference = mcosql_column_type::CT_REFERENCE as isize,
    /// An array of values.
    Array = mcosql_column_type::CT_ARRAY as isize,

    //Struct = mcosql_column_type::CT_STRUCT as isize,
    /// A blob.
    Blob = mcosql_column_type::CT_BLOB as isize,

    // DataSource = mcosql_column_type::CT_DATA_SOURCE as isize,

    // List = mcosql_column_type::CT_LIST as isize,
    /// A sequence.
    Sequence = mcosql_column_type::CT_SEQUENCE as isize,
}

impl Type {
    pub(crate) fn from_mco(v: mcosql_column_type::Type) -> Option<Self> {
        match v {
            mcosql_column_type::CT_NULL => Some(Type::Null),
            mcosql_column_type::CT_BOOL => Some(Type::Bool),
            mcosql_column_type::CT_INT1 => Some(Type::Int1),
            mcosql_column_type::CT_INT2 => Some(Type::Int2),
            mcosql_column_type::CT_INT4 => Some(Type::Int4),
            mcosql_column_type::CT_INT8 => Some(Type::Int8),
            mcosql_column_type::CT_UINT1 => Some(Type::UInt1),
            mcosql_column_type::CT_UINT2 => Some(Type::UInt2),
            mcosql_column_type::CT_UINT4 => Some(Type::UInt4),
            mcosql_column_type::CT_UINT8 => Some(Type::UInt8),
            mcosql_column_type::CT_REAL4 => Some(Type::Real4),
            mcosql_column_type::CT_REAL8 => Some(Type::Real8),
            mcosql_column_type::CT_TIME => Some(Type::Time),
            mcosql_column_type::CT_NUMERIC => Some(Type::Numeric),
            mcosql_column_type::CT_STRING => Some(Type::String),
            mcosql_column_type::CT_BINARY => Some(Type::Binary),
            mcosql_column_type::CT_ARRAY => Some(Type::Array),
            mcosql_column_type::CT_BLOB => Some(Type::Blob),
            mcosql_column_type::CT_SEQUENCE => Some(Type::Sequence),
            _ => None,
        }
    }
}

/// A generic SQL value.
///
/// This struct is a wrapper for the C++ SQL API's `Value` class. It is
/// currently impossible to instantiate this structure from the application
/// code. Instead, applications pass values of types that implement the
/// [`ToValue`] trait to the SQL engine. Applications will only deal with
/// `Value`s they receive from the SQL engine.
///
/// # Memory Management
///
/// Internally, SQL values are always produced by the SQL engine's custom
/// allocators, and thus cannot outlive them. Consequently, they do not quite
/// follow Rust's memory management rules. Dropping a `Value` does not
/// free its memory instantly; it is only done when the allocator
/// is destroyed. However, most `Value`s returned by the SQL API are wrapped in
/// a [`Ref`]. A `Ref` releases the `Value` it refers to when it goes
/// out of scope.
///
/// [`ToValue`]: ./trait.ToValue.html
/// [`Ref`]: ./struct.Ref.html
#[repr(transparent)]
pub struct Value<'a> {
    alloc: PhantomData<&'a AllocatorRef<'a>>,
    h: exdb_sys::mcosql_rs_value,
}

impl<'a> Value<'a> {
    fn from_handle(h: exdb_sys::mcosql_rs_value, _allocator: AllocatorRef<'a>) -> Self {
        Value {
            alloc: PhantomData,
            h,
        }
    }

    fn new_null() -> Result<Self> {
        let mut h = MaybeUninit::uninit();
        result_from_code(unsafe { exdb_sys::mcosql_rs_value_create_null(h.as_mut_ptr()) }).and(Ok(
            Value {
                alloc: PhantomData,
                h: unsafe { h.assume_init() },
            },
        ))
    }

    fn new_bool(val: bool) -> Result<Self> {
        let ival = if val { 1 } else { 0 };
        let mut h = MaybeUninit::uninit();
        result_from_code(unsafe { exdb_sys::mcosql_rs_value_create_bool(ival, h.as_mut_ptr()) })
            .and(Ok(Value {
                alloc: PhantomData,
                h: unsafe { h.assume_init() },
            }))
    }

    fn new_int(val: i64, alloc: AllocatorRef<'a>) -> Result<Self> {
        let mut h = MaybeUninit::uninit();
        result_from_code(unsafe {
            exdb_sys::mcosql_rs_value_create_int(alloc.h, val, h.as_mut_ptr())
        })
        .and(Ok(Value {
            alloc: PhantomData,
            h: unsafe { h.assume_init() },
        }))
    }

    fn new_real(val: f64, alloc: AllocatorRef<'a>) -> Result<Self> {
        let mut h = MaybeUninit::uninit();
        result_from_code(unsafe {
            exdb_sys::mcosql_rs_value_create_real(alloc.h, val, h.as_mut_ptr())
        })
        .and(Ok(Value {
            alloc: PhantomData,
            h: unsafe { h.assume_init() },
        }))
    }

    fn new_string(val: &str, alloc: AllocatorRef<'a>) -> Result<Self> {
        let mut h = MaybeUninit::uninit();
        result_from_code(unsafe {
            exdb_sys::mcosql_rs_value_create_string(
                alloc.h,
                val.as_ptr() as *const i8,
                val.len(),
                h.as_mut_ptr(),
            )
        })
        .and(Ok(Value {
            alloc: PhantomData,
            h: unsafe { h.assume_init() },
        }))
    }

    fn new_binary(val: &[u8], alloc: AllocatorRef<'a>) -> Result<Self> {
        let mut h = MaybeUninit::uninit();
        result_from_code(unsafe {
            exdb_sys::mcosql_rs_value_create_binary(
                alloc.h,
                val.as_ptr() as *const c_void,
                val.len(),
                h.as_mut_ptr(),
            )
        })
        .and(Ok(Value {
            alloc: PhantomData,
            h: unsafe { h.assume_init() },
        }))
    }

    fn new_date_time(val: &SystemTime, alloc: AllocatorRef<'a>) -> Result<Self> {
        let dur = val
            .duration_since(UNIX_EPOCH)
            .or(Err(Error::new_sql(mcosql_error_code::INVALID_TYPE_CAST)))?;

        let prec = unsafe {
            exdb_sys::mco_runtime_getoption(
                options::mco_rt_defines::keys::MCO_RT_OPTION_DATETIME_PRECISION as i32,
            )
        } as u128;

        let val;

        if prec >= 1_000_000_000 {
            val = dur.as_nanos() * (prec / 1_000_000_000);
        } else if prec >= 1_000_000 {
            val = dur.as_micros() * (prec / 1_000_000);
        } else if prec >= 1_000 {
            val = dur.as_millis() * (prec / 1_000);
        } else {
            val = (dur.as_secs() as u128) * prec;
        }

        let val =
            u64::try_from(val).or(Err(Error::new_sql(mcosql_error_code::INVALID_TYPE_CAST)))?;

        let mut h = MaybeUninit::uninit();
        result_from_code(unsafe {
            exdb_sys::mcosql_rs_value_create_datetime(alloc.h, val, h.as_mut_ptr())
        })
        .and(Ok(Value {
            alloc: PhantomData,
            h: unsafe { h.assume_init() },
        }))
    }

    fn new_numeric(val_scaled: i64, prec: usize, alloc: AllocatorRef<'a>) -> Result<Self> {
        let mut h = MaybeUninit::uninit();
        result_from_code(unsafe {
            exdb_sys::mcosql_rs_value_create_numeric(alloc.h, val_scaled, prec, h.as_mut_ptr())
        })
        .and(Ok(Value {
            alloc: PhantomData,
            h: unsafe { h.assume_init() },
        }))
    }

    /// Returns the type of the contained value.
    pub fn value_type(&self) -> Result<Type> {
        let mut ty = MaybeUninit::uninit();
        result_from_code(unsafe { exdb_sys::mcosql_rs_value_type(self.h, ty.as_mut_ptr()) }).and(
            Type::from_mco(unsafe { ty.assume_init() })
                .ok_or(Error::new_sql(mcosql_error_code::RUNTIME_ERROR)),
        )
    }

    /// For non-scalar types, returns the size of the data.
    ///
    /// Depending on the contained type, returns:
    ///
    /// - `String` and `Binary`: length of the string or binary string;
    /// - `Array`: number of elements.
    pub fn size(&self) -> Result<usize> {
        let mut ret = MaybeUninit::uninit();
        result_from_code(unsafe { exdb_sys::mcosql_rs_value_size(self.h, ret.as_mut_ptr()) })
            .and(Ok(unsafe { ret.assume_init() }))
    }

    /// Returns `true` if the value is an SQL `null` value.
    pub fn is_null(&self) -> bool {
        0 != unsafe { exdb_sys::mcosql_rs_value_is_null(self.h) }
    }

    /// Returns `true` if the value is a boolean `true` value, or a non-zero
    /// integer.
    pub fn is_true(&self) -> bool {
        0 != unsafe { exdb_sys::mcosql_rs_value_is_true(self.h) }
    }

    /// Casts the value to `i64`.
    ///
    /// Strings are parsed and converted, if possible; otherwise an error is
    /// returned.
    pub fn to_i64(&self) -> Result<i64> {
        let mut val = MaybeUninit::uninit();
        result_from_code(unsafe { exdb_sys::mcosql_rs_value_int(self.h, val.as_mut_ptr()) })
            .and(Ok(unsafe { val.assume_init() }))
    }

    /// Casts the value to `f64`.
    ///
    /// Strings are parsed and converted, if possible; otherwise an error is
    /// returned.
    pub fn to_real(&self) -> Result<f64> {
        let mut val = MaybeUninit::uninit();
        result_from_code(unsafe { exdb_sys::mcosql_rs_value_real(self.h, val.as_mut_ptr()) })
            .and(Ok(unsafe { val.assume_init() }))
    }

    /// Casts the value to the number of system ticks elapsed since
    /// the beginning of the epoch.
    ///
    /// String values are parsed using `strptime()` where available, or a
    /// compatible custom function.
    pub fn to_date_time(&self) -> Result<u64> {
        let mut val = MaybeUninit::uninit();
        result_from_code(unsafe { exdb_sys::mcosql_rs_value_datetime(self.h, val.as_mut_ptr()) })
            .and(Ok(unsafe { val.assume_init() }))
    }

    /// Casts the value to the number of system ticks elapsed since
    /// the beginning of the epoch, and converts the resulting value to
    /// `std::time::SystemTime`.
    pub fn to_system_time(&self) -> Result<SystemTime> {
        let prec = unsafe {
            exdb_sys::mco_runtime_getoption(
                options::mco_rt_defines::keys::MCO_RT_OPTION_DATETIME_PRECISION as i32,
            )
        } as u64;
        let dt = self.to_date_time()?;
        let dur;
        if prec >= 1_000_000_000 {
            dur = Duration::from_nanos(dt / (prec / 1_000_000_000));
        } else if prec >= 1_000_000 {
            dur = Duration::from_micros(dt / (prec / 1_000_000));
        } else if prec >= 1_000 {
            dur = Duration::from_millis(dt / (prec / 1_000));
        } else {
            dur = Duration::from_secs(dt / prec);
        }

        UNIX_EPOCH
            .checked_add(dur)
            .ok_or(Error::new_sql(mcosql_error_code::INVALID_TYPE_CAST))
    }

    /// Returns the contained fixed-width numeric value, if any, or an error.
    pub fn to_numeric(&self) -> Result<Numeric> {
        if self.value_type()? == Type::Numeric {
            let mut val = 0i64;
            let mut prec = 0usize;
            result_from_code(unsafe {
                exdb_sys::mcosql_rs_value_numeric(self.h, &mut val, &mut prec)
            })
            .and(
                Numeric::new(val, prec).ok_or(Error::new_sql(mcosql_error_code::INVALID_TYPE_CAST)),
            )
        } else {
            Err(Error::new_sql(mcosql_error_code::INVALID_TYPE_CAST))
        }
    }

    /// Converts the value to the string representation.
    pub fn to_string(&self) -> Result<String> {
        let alloc = allocator::Owned::new()?;
        let mut h = MaybeUninit::uninit();
        result_from_code(unsafe {
            exdb_sys::mcosql_rs_value_string_ref(self.h, alloc.h, h.as_mut_ptr())
        })?;
        let sref = Ref::from_handle(unsafe { h.assume_init() }, &alloc);
        let data = unsafe { slice::from_raw_parts(sref.pointer()? as *const u8, sref.size()?) };
        let v = data.to_vec();

        String::from_utf8(v).or(Err(Error::new_sql(mcosql_error_code::INVALID_TYPE_CAST)))
    }

    /// Returns a string slice pointing to the contents of a `String` value,
    /// or an error if the value is not a `String`.
    pub fn as_str(&self) -> Result<&str> {
        if self.value_type()? != Type::String {
            Err(Error::new_sql(mcosql_error_code::INVALID_TYPE_CAST))
        } else {
            let data = unsafe { slice::from_raw_parts(self.pointer()? as *const u8, self.size()?) };
            str::from_utf8(data).or(Err(Error::new_sql(mcosql_error_code::INVALID_TYPE_CAST)))
        }
    }

    /// Returns a copy of the bytes of a string or a binary value.
    pub fn to_binary(&self) -> Result<Vec<u8>> {
        let alloc = allocator::Owned::new()?;
        let mut h = MaybeUninit::uninit();
        result_from_code(unsafe {
            exdb_sys::mcosql_rs_value_binary(self.h, alloc.h, h.as_mut_ptr())
        })?;
        let sref = Ref::from_handle(unsafe { h.assume_init() }, &alloc);
        let data = unsafe { slice::from_raw_parts(sref.pointer()? as *const u8, sref.size()?) };
        Ok(data.to_vec())
    }

    /// Returns a byte slice pointing to the contents of a `Binary` value,
    /// or an error if the value is not a `Binary`.
    pub fn as_bytes(&self) -> Result<&[u8]> {
        if self.value_type()? != Type::Binary {
            Err(Error::new_sql(mcosql_error_code::INVALID_TYPE_CAST))
        } else {
            Ok(unsafe { slice::from_raw_parts(self.pointer()? as *const u8, self.size()?) })
        }
    }

    /// Casts the value to `Array` if it has the `Array` type; returns
    /// an error otherwise.
    pub fn as_array(&self) -> Result<&Array> {
        if let Type::Array = self.value_type()? {
            Ok(unsafe { &*(self as *const Value as *const Array) })
        } else {
            Err(Error::new_sql(mcosql_error_code::INVALID_TYPE_CAST))
        }
    }

    /// Casts the value to `Sequence` if it has the `Sequence` type; returns
    /// an error otherwise.
    pub fn as_sequence(&self) -> Result<&Sequence> {
        if let Type::Sequence = self.value_type()? {
            Ok(unsafe { &*(self as *const Value as *const Sequence) })
        } else {
            Err(Error::new_sql(mcosql_error_code::INVALID_TYPE_CAST))
        }
    }

    /// Casts the value to `Blob` if it has the `Blob` type; returns
    /// an error otherwise.
    pub fn as_blob(&self) -> Result<&Blob> {
        if let Type::Blob = self.value_type()? {
            Ok(unsafe { &*(self as *const Value as *const Blob) })
        } else {
            Err(Error::new_sql(mcosql_error_code::INVALID_TYPE_CAST))
        }
    }

    unsafe fn pointer(&self) -> Result<*const c_void> {
        let mut p = MaybeUninit::uninit();
        result_from_code(exdb_sys::mcosql_rs_value_ptr(self.h, p.as_mut_ptr()))
            .and(Ok(p.assume_init()))
    }

    // Highly unsafe: must be released using the allocator which was used to allocate the value;
    // leaves the value in the invalid state.
    unsafe fn release(&self, alloc: AllocatorRef) -> Result<()> {
        result_from_code(exdb_sys::mcosql_rs_value_release(alloc.h, self.h))
    }
}

impl<'a> From<Array<'a>> for Value<'a> {
    fn from(array: Array<'a>) -> Self {
        array.val
    }
}

impl<'a> From<Sequence<'a>> for Value<'a> {
    fn from(seq: Sequence<'a>) -> Self {
        seq.val
    }
}

impl<'a> From<Blob<'a>> for Value<'a> {
    fn from(blob: Blob<'a>) -> Self {
        blob.val
    }
}

/// An SQL value reference.
///
/// In addition to the value itself, a value reference holds a reference to the
/// value's allocator, making it possible to release the value when the
/// reference goes out of scope.
pub struct Ref<'a> {
    r: exdb_sys::mcosql_rs_value_ref,
    owner: PhantomData<&'a ()>,
}

impl<'a> Ref<'a> {
    pub(crate) fn from_handle<T>(r: exdb_sys::mcosql_rs_value_ref, _owner: &'a T) -> Self {
        Ref {
            r,
            owner: PhantomData,
        }
    }

    fn allocator(&'a self) -> AllocatorRef<'a> {
        AllocatorRef::from_handle(self.r.allocator, self)
    }

    fn defused_clone(&'a self) -> Ref<'a> {
        Ref {
            r: exdb_sys::mcosql_rs_value_ref {
                allocator: ptr::null_mut(),
                ref_: self.r.ref_,
            },
            owner: PhantomData,
        }
    }

    fn is_null_ref(&self) -> bool {
        self.r.ref_.is_null()
    }

    fn release_value(&mut self) {
        if !self.is_null_ref() && !self.r.allocator.is_null() {
            let alloc = self.allocator();
            let res = unsafe { self.release(alloc) };
            debug_assert!(res.is_ok());
        }
        self.r.ref_ = ptr::null_mut();
    }

    // Unsafe: new value must be allocated by the same allocator and produced by the same owner;
    // no other references to the value must be held.
    unsafe fn replace_value(&mut self, new_value: exdb_sys::mcosql_rs_value) {
        self.release_value();
        self.r.ref_ = new_value;
    }
}

impl<'a> Deref for Ref<'a> {
    type Target = Value<'a>;

    fn deref(&self) -> &Self::Target {
        assert!(!self.is_null_ref());

        unsafe { &*(&self.r.ref_ as *const exdb_sys::mcosql_rs_value as *const Value) }
    }
}

impl<'a> Drop for Ref<'a> {
    fn drop(&mut self) {
        self.release_value();
    }
}

/// An SQL array.
///
/// An *e*X*treme*DB SQL array contains [`Value`]s of the same type, and
/// exposes public methods to access them.
///
/// A [`Value`] can only be converted into an `Array` if it has the `Array`
/// type. The opposite conversion is always possible.
///
/// [`Value`]: ./struct.Value.html
// WARNING: must have same repr as Value! Value is cast to Array in Value::as_array
#[repr(transparent)]
pub struct Array<'a> {
    val: Value<'a>,
}

impl<'a> Array<'a> {
    fn new<T: ArrayElem>(items: &[T], alloc: AllocatorRef<'a>) -> Result<Self> {
        let mut h = MaybeUninit::uninit();

        result_from_code(unsafe {
            exdb_sys::mcosql_rs_value_create_array(
                alloc.h,
                T::static_type() as mcosql_column_type::Type,
                items.len(),
                h.as_mut_ptr(),
            )
        })?;

        let mut ret = Array {
            val: Value::from_handle(unsafe { h.assume_init() }, alloc),
        };

        ret.set_body(items).and(Ok(ret))
    }

    fn is_plain(&self) -> bool {
        let mut plain = 0i32;

        let rc = unsafe { exdb_sys::mcosql_rs_array_is_plain(self.val.h, &mut plain) };
        // Not expected to fail unless something is really wrong
        debug_assert_eq!(mcosql_error_code::SQL_OK, rc);

        plain != 0
    }

    /// Returns the type of the array's elements.
    pub fn elem_type(&self) -> Result<Type> {
        let mut ty = MaybeUninit::uninit();

        result_from_code(unsafe {
            exdb_sys::mcosql_rs_array_elem_type(self.val.h, ty.as_mut_ptr())
        })
        .and(
            Type::from_mco(unsafe { ty.assume_init() })
                .ok_or(Error::new_sql(mcosql_error_code::RUNTIME_ERROR)),
        )
    }

    /// Returns the length of the array.
    pub fn len(&self) -> Result<usize> {
        self.val.size()
    }

    /// Returns the element at the given index.
    pub fn get_at(&self, at: usize) -> Result<Ref> {
        let mut h = MaybeUninit::uninit();

        result_from_code(unsafe {
            exdb_sys::mcosql_rs_array_get_at(self.val.h, at, h.as_mut_ptr())
        })
        .and(Ok(Ref::from_handle(unsafe { h.assume_init() }, self)))
    }

    fn allocator(&'a self) -> Result<AllocatorRef<'a>> {
        let mut alloc_h = MaybeUninit::uninit();

        result_from_code(unsafe {
            exdb_sys::mcosql_rs_array_allocator(self.val.h, alloc_h.as_mut_ptr())
        })
        .and(Ok(AllocatorRef::from_handle(
            unsafe { alloc_h.assume_init() },
            self,
        )))
    }

    fn set_body<T: ArrayElem>(&mut self, body: &[T]) -> Result<()> {
        if body.len() != self.val.size()? {
            // The caller is currently required to set the entire body.
            Err(Error::new_sql(mcosql_error_code::RUNTIME_ERROR))
        } else {
            if self.is_plain() {
                self.set_body_plain(body)
            } else {
                self.set_body_values(body)
            }
        }
    }

    fn set_body_values<T: ArrayElem>(&mut self, body: &[T]) -> Result<()> {
        // Sanity check: must not be called for plain arrays.
        debug_assert!(!self.is_plain());

        // Get the reference to the array's own allocator to ensure that values live
        // as long as the array.
        let alloc = self.allocator()?;

        for i in 0..body.len() {
            // Values do not implement Drop. Hence, the handle of the allocated value will
            // remain valid when val goes out of scope.
            let val = body[i].to_value(alloc)?;
            result_from_code(unsafe { exdb_sys::mcosql_rs_array_set_at(self.val.h, i, val.h) })?;
        }

        Ok(())
    }

    fn set_body_plain<T: ArrayElem>(&mut self, body: &[T]) -> Result<()> {
        // Sanity check: must only be called for plain arrays.
        debug_assert!(self.is_plain());

        result_from_code(unsafe {
            exdb_sys::mcosql_rs_array_set_body(
                self.val.h,
                body.as_ptr() as *const c_void,
                body.len(),
            )
        })
    }
}

impl<'a> TryFrom<Value<'a>> for Array<'a> {
    type Error = Error;

    fn try_from(value: Value<'a>) -> std::result::Result<Self, Self::Error> {
        if let Type::Array = value.value_type()? {
            Ok(Array { val: value })
        } else {
            Err(Error::new_sql(mcosql_error_code::INVALID_TYPE_CAST))
        }
    }
}

/// A trait for converting a value to a SQL [`Value`].
///
/// Since it is currently impossible to instantiate a `Value` in the application
/// code, this trait should be considered sealed.
///
/// This module implements the `ToValue` trait for the common Rust types
/// supported by the *e*X*treme*DB SQL engine. Any type that implements this
/// trait can be passed as a parameter to the SQL statement execution methods.
///
/// [`Value`]: ./trait.ToValue.html
pub trait ToValue {
    #[doc(hidden)]
    fn to_value<'a>(&self, alloc: AllocatorRef<'a>) -> Result<Value<'a>>;
}

/// An SQL sequence.
///
/// An *e*X*treme*DB SQL sequence contains [`Value`]s of the same type, and
/// exposes public methods to access them.
///
/// Sequences are conceptually different from arrays. Because of a different
/// internal implementation, their elements are accessed using an iterator,
/// instead of the getter methods.
///
/// A [`Value`] can only be converted into a `Sequence` if it has the `Sequence`
/// type. The opposite conversion is always possible.
///
/// [`Value`]: ./struct.Value.html
// WARNING: must have same repr as Value! Value is cast to Sequence in Value::as_sequence
#[repr(transparent)]
pub struct Sequence<'a> {
    val: Value<'a>,
}

impl<'a> Sequence<'a> {
    /// Returns the type of the sequence elements.
    pub fn elem_type(&self) -> Result<Type> {
        let mut ty = MaybeUninit::uninit();

        result_from_code(unsafe { exdb_sys::mcosql_rs_seq_elem_type(self.val.h, ty.as_mut_ptr()) })
            .and(
                Type::from_mco(unsafe { ty.assume_init() })
                    .ok_or(Error::new_sql(mcosql_error_code::RUNTIME_ERROR)),
            )
    }

    /// Returns the number of elements in the sequence.
    pub fn count(&self) -> Result<usize> {
        let mut ret = MaybeUninit::uninit();

        result_from_code(unsafe { exdb_sys::mcosql_rs_seq_count(self.val.h, ret.as_mut_ptr()) })
            .and(Ok(unsafe { ret.assume_init() }))
    }

    /// Returns an iterator for the sequence.
    pub fn iterator(&'a self) -> Result<SequenceIterator<'a>> {
        self.get_iterator()
            .and(self.reset())
            .and(Ok(SequenceIterator::new(self)))
    }

    fn get_iterator(&self) -> Result<()> {
        result_from_code(unsafe { exdb_sys::mcosql_rs_seq_get_iterator(self.val.h) })
    }

    fn reset(&self) -> Result<()> {
        result_from_code(unsafe { exdb_sys::mcosql_rs_seq_reset(self.val.h) })
    }

    unsafe fn next(&self) -> Result<exdb_sys::mcosql_rs_value> {
        let mut ret = MaybeUninit::uninit();

        result_from_code(exdb_sys::mcosql_rs_seq_next(self.val.h, ret.as_mut_ptr()))
            .and(Ok(ret.assume_init()))
    }

    fn allocator(&'a self) -> Result<AllocatorRef<'a>> {
        let mut alloc = MaybeUninit::uninit();

        result_from_code(unsafe {
            exdb_sys::mcosql_rs_seq_allocator(self.val.h, alloc.as_mut_ptr())
        })
        .and(Ok(AllocatorRef::from_handle(
            unsafe { alloc.assume_init() },
            self,
        )))
    }
}

/// A sequence iterator.
///
/// This type is used to iterate through the items of the sequence. Like the
/// `Cursor` type, it does not follow the conventions of the standard Rust
/// iterators, because lifetimes of the values it returns are constrained
/// by the lifetime of the sequence.
///
/// The iterator is initially positioned before the first item.
pub struct SequenceIterator<'a> {
    seq: &'a Sequence<'a>,
    val_ref: Ref<'a>,
}

impl<'a> SequenceIterator<'a> {
    fn new(seq: &'a Sequence<'a>) -> Self {
        let alloc = seq.allocator().unwrap();
        let r = exdb_sys::mcosql_rs_value_ref {
            allocator: alloc.h,
            ref_: ptr::null_mut(),
        };

        SequenceIterator {
            seq,
            val_ref: Ref::from_handle(r, seq),
        }
    }

    /// Advances the iterator.
    ///
    /// If this function returns `true`, the current element can be accessed.
    /// `false` indicates that the iterator has been moved past the last
    /// element.
    pub fn advance(&mut self) -> Result<bool> {
        unsafe { self.val_ref.replace_value(self.seq.next()?) };

        if self.val_ref.is_null_ref() {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    /// Returns the element currently pointed at by the iterator.
    ///
    /// Returns `None` if the iterator hasn't been advanced at least once,
    /// or has reached the end of the sequence.
    pub fn current_value(&'a self) -> Option<Ref<'a>> {
        if self.val_ref.is_null_ref() {
            None
        } else {
            // Produce a defused reference
            Some(self.val_ref.defused_clone())
        }
    }
}

/// A fixed-width integer.
///
/// This type is used to pass fixed-width integers between the application code
/// and the SQL engine.
pub struct Numeric {
    val_scaled: i64,
    prec: usize,
}

impl Numeric {
    /// Constructs a new fixed-width integer from a scaled value and precision.
    ///
    /// Returns `None` if precision is greater or equal to 19.
    ///
    /// # Examples
    ///
    /// A numeric value of `12.345` can be constructed using a scaled value of
    /// `12345` and a precision of `3`:
    ///
    /// ```
    /// # use extremedb::sql::value::Numeric;
    /// let num = Numeric::new(12345, 3).unwrap();
    /// assert_eq!(num.int_part(), 12);
    /// assert_eq!(num.fract_part(), 345);
    /// ```
    pub fn new(val_scaled: i64, prec: usize) -> Option<Self> {
        if prec <= 19 {
            Some(Numeric { val_scaled, prec })
        } else {
            None
        }
    }

    /// Returns the scaled value.
    pub fn value_scaled(&self) -> i64 {
        self.val_scaled
    }

    /// Returns the precision.
    pub fn precision(&self) -> usize {
        self.prec
    }

    /// Returns the integer part of the numeric value.
    pub fn int_part(&self) -> i64 {
        self.val_scaled / self.scale() as i64
    }

    /// Returns the fractional part of the numeric value.
    pub fn fract_part(&self) -> u64 {
        (self.val_scaled.abs() as u64).wrapping_rem(self.scale() as u64)
    }

    /// Destructures the numeric value into the scaled value and the precision.
    pub fn destruct(self) -> (i64, usize) {
        (self.val_scaled, self.prec)
    }

    fn scale(&self) -> usize {
        10usize.pow(self.prec as u32)
    }
}

impl Into<f64> for Numeric {
    fn into(self) -> f64 {
        self.val_scaled as f64 / self.scale() as f64
    }
}

impl Display for Numeric {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), FmtError> {
        write!(f, "{}.{}", self.int_part(), self.fract_part())
    }
}

impl ToValue for Numeric {
    fn to_value<'a>(&self, alloc: AllocatorRef<'a>) -> Result<Value<'a>> {
        Value::new_numeric(self.val_scaled, self.prec, alloc)
    }
}

/// An SQL blob.
///
/// An *e*X*treme*DB SQL blob is a large binary object that can contain
/// arbitrary data.
///
/// This type contains public methods that allow the applications to read the
/// data. A blob keeps an internal read pointer; each read operation advances
/// it by the number of bytes read. To revert the pointer to the beginning of
/// the blob's data, use the [`reset()`] method.
///
/// A [`Value`] can only be converted into a `Blob` if it has the `Blob`
/// type. The opposite conversion is always possible.
///
/// [`Value`]: ./struct.Value.html
/// [`reset()`]: #method.reset
// WARNING: must have same repr as Value! Value is cast to Blob in Value::as_blob
#[repr(transparent)]
pub struct Blob<'a> {
    val: Value<'a>,
}

impl<'a> Blob<'a> {
    /// Returns the number of bytes available to be extracted with a single
    /// `get()` operation.
    ///
    /// This is *not* the total size of the blob. If the blob is split into
    /// segments, this can be equal to the size of one segment.
    pub fn available(&self) -> Result<usize> {
        let mut avail = 0usize;
        result_from_code(unsafe { exdb_sys::mcosql_rs_blob_available(self.val.h, &mut avail) })
            .and(Ok(avail))
    }

    /// Reads the blob data into the buffer.
    ///
    /// This method will fill the buffer up to its capacity. If the number
    /// of bytes available for reading is smaller than the buffer's capacity,
    /// this method will perform multiple reads.
    pub fn get_into(&self, buf: &mut Vec<u8>) -> Result<()> {
        unsafe {
            let new_len = self.get_raw(buf.as_mut_ptr() as *mut c_void, buf.capacity())?;
            buf.set_len(new_len)
        };
        Ok(())
    }

    /// Reads the given amount of bytes from the blob.
    ///
    /// If the number of bytes available for reading is smaller than the
    /// requested size, this method will perform multiple reads.
    pub fn get(&self, size: usize) -> Result<Vec<u8>> {
        let mut ret = Vec::with_capacity(size);
        self.get_into(&mut ret).and(Ok(ret))
    }

    /// Resets the blob's read pointer.
    pub fn reset(&self) -> Result<()> {
        result_from_code(unsafe { exdb_sys::mcosql_rs_blob_reset(self.val.h, 0) })
    }

    unsafe fn get_raw(&self, p: *mut c_void, l: usize) -> Result<usize> {
        let mut total = 0usize;

        while total < l {
            let mut nread = 0usize;

            result_from_code(exdb_sys::mcosql_rs_blob_get(
                self.val.h,
                p.add(total),
                l - total,
                &mut nread,
            ))?;

            if nread == 0 {
                break;
            } else {
                total += nread;
            }
        }

        Ok(total)
    }
}

/// A `Binary` value wrapper.
///
/// The sole purpose of this type is passing `Binary` values to the SQL engine.
///
/// *Note that slices of `u8` are converted to [`Array`] when passed as
/// statement parameters.*
///
/// [`Array`]: ./struct.Array.html
pub struct Binary<'a>(&'a [u8]);

impl<'a> Binary<'a> {
    /// Creates a new `Binary` value wrapper.
    pub fn new(bytes: &'a [u8]) -> Self {
        Binary(bytes)
    }
}

impl ToValue for bool {
    fn to_value<'a>(&self, _alloc: AllocatorRef<'a>) -> Result<Value<'a>> {
        Value::new_bool(*self)
    }
}

impl ToValue for u8 {
    fn to_value<'a>(&self, alloc: AllocatorRef<'a>) -> Result<Value<'a>> {
        Value::new_int(*self as i64, alloc)
    }
}

impl ToValue for u16 {
    fn to_value<'a>(&self, alloc: AllocatorRef<'a>) -> Result<Value<'a>> {
        Value::new_int(*self as i64, alloc)
    }
}

impl ToValue for u32 {
    fn to_value<'a>(&self, alloc: AllocatorRef<'a>) -> Result<Value<'a>> {
        Value::new_int(*self as i64, alloc)
    }
}

impl ToValue for u64 {
    fn to_value<'a>(&self, alloc: AllocatorRef<'a>) -> Result<Value<'a>> {
        Value::new_int(*self as i64, alloc)
    }
}

impl ToValue for i8 {
    fn to_value<'a>(&self, alloc: AllocatorRef<'a>) -> Result<Value<'a>> {
        Value::new_int(*self as i64, alloc)
    }
}

impl ToValue for i16 {
    fn to_value<'a>(&self, alloc: AllocatorRef<'a>) -> Result<Value<'a>> {
        Value::new_int(*self as i64, alloc)
    }
}

impl ToValue for i32 {
    fn to_value<'a>(&self, alloc: AllocatorRef<'a>) -> Result<Value<'a>> {
        Value::new_int(*self as i64, alloc)
    }
}

impl ToValue for i64 {
    fn to_value<'a>(&self, alloc: AllocatorRef<'a>) -> Result<Value<'a>> {
        Value::new_int(*self as i64, alloc)
    }
}

impl ToValue for f32 {
    fn to_value<'a>(&self, alloc: AllocatorRef<'a>) -> Result<Value<'a>> {
        Value::new_real(*self as f64, alloc)
    }
}

impl ToValue for f64 {
    fn to_value<'a>(&self, alloc: AllocatorRef<'a>) -> Result<Value<'a>> {
        Value::new_real(*self as f64, alloc)
    }
}

impl ToValue for &str {
    fn to_value<'a>(&self, alloc: AllocatorRef<'a>) -> Result<Value<'a>> {
        Value::new_string(self, alloc)
    }
}

impl ToValue for Binary<'_> {
    fn to_value<'a>(&self, alloc: AllocatorRef<'a>) -> Result<Value<'a>> {
        Value::new_binary(self.0, alloc)
    }
}

impl<T: ArrayElem> ToValue for &[T] {
    fn to_value<'a>(&self, alloc: AllocatorRef<'a>) -> Result<Value<'a>> {
        let array = Array::new(self, alloc)?;
        Ok(array.into())
    }
}

impl<T: ToValue> ToValue for Option<T> {
    fn to_value<'a>(&self, alloc: AllocatorRef<'a>) -> Result<Value<'a>> {
        match self {
            Some(val) => val.to_value(alloc),
            None => Value::new_null(),
        }
    }
}

impl ToValue for SystemTime {
    fn to_value<'a>(&self, alloc: AllocatorRef<'a>) -> Result<Value<'a>> {
        Value::new_date_time(self, alloc)
    }
}

/// A trait for retrieving the SQL type of the implementing Rust type.
pub trait StaticTypeInfo {
    fn static_type() -> Type;
}

macro_rules! impl_static_type_info {
    ($ty:ty, $col_ty:path) => {
        impl StaticTypeInfo for $ty {
            fn static_type() -> Type {
                $col_ty
            }
        }
    };
}

impl_static_type_info!(u8, Type::UInt1);
impl_static_type_info!(u16, Type::UInt2);
impl_static_type_info!(u32, Type::UInt4);
impl_static_type_info!(u64, Type::UInt8);
impl_static_type_info!(i8, Type::Int1);
impl_static_type_info!(i16, Type::Int2);
impl_static_type_info!(i32, Type::Int4);
impl_static_type_info!(i64, Type::Int8);
impl_static_type_info!(f32, Type::Real4);
impl_static_type_info!(f64, Type::Real8);
impl_static_type_info!(&str, Type::String);
impl_static_type_info!(SystemTime, Type::Time);

/// A marker trait for types that can be an element of an SQL array.
pub trait ArrayElem: ToValue + StaticTypeInfo {}

impl<T: ToValue + StaticTypeInfo> ArrayElem for T {}
