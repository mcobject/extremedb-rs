// sql.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

use crate::*;

#[cfg(feature = "rsql")]
mod rsql;
#[cfg(feature = "rsql")]
pub use rsql::*;

pub mod mcosql_error_code {
    pub type Type = u32;
    pub const SQL_OK: Type = 0;
    pub const NO_MORE_ELEMENTS: Type = 1;
    pub const INVALID_TYPE_CAST: Type = 2;
    pub const COMPILE_ERROR: Type = 3;
    pub const NOT_SINGLE_VALUE: Type = 4;
    pub const INVALID_OPERATION: Type = 5;
    pub const INDEX_OUT_OF_BOUNDS: Type = 6;
    pub const NOT_ENOUGH_MEMORY: Type = 7;
    pub const NOT_UNIQUE: Type = 8;
    pub const NOT_PREPARED: Type = 9;
    pub const RUNTIME_ERROR: Type = 10;
    pub const COMMUNICATION_ERROR: Type = 11;
    pub const UPGRAGE_NOT_POSSIBLE: Type = 12;
    pub const SQL_CONFLICT: Type = 13;
    pub const SQL_NULL_REFERENCE: Type = 14;
    pub const SQL_INVALID_STATE: Type = 15;
    pub const SQL_INVALID_OPERAND: Type = 16;
    pub const SQL_NULL_VALUE: Type = 17;
    pub const SQL_BAD_CSV_FORMAT: Type = 18;
    pub const SQL_SYSTEM_ERROR: Type = 19;
}

pub use self::mcosql_error_code::Type as status_t;

pub mod mcosql_column_type {
    pub type Type = u32;
    pub const CT_NULL: Type = 0;
    pub const CT_BOOL: Type = 1;
    pub const CT_INT1: Type = 2;
    pub const CT_UINT1: Type = 3;
    pub const CT_INT2: Type = 4;
    pub const CT_UINT2: Type = 5;
    pub const CT_INT4: Type = 6;
    pub const CT_UINT4: Type = 7;
    pub const CT_INT8: Type = 8;
    pub const CT_UINT8: Type = 9;
    pub const CT_REAL4: Type = 10;
    pub const CT_REAL8: Type = 11;
    pub const CT_TIME: Type = 12;
    pub const CT_NUMERIC: Type = 13;
    pub const CT_UNICODE: Type = 14;
    pub const CT_STRING: Type = 15;
    pub const CT_BINARY: Type = 16;
    pub const CT_REFERENCE: Type = 17;
    pub const CT_ARRAY: Type = 18;
    pub const CT_STRUCT: Type = 19;
    pub const CT_BLOB: Type = 20;
    pub const CT_DATA_SOURCE: Type = 21;
    pub const CT_LIST: Type = 22;
    pub const CT_SEQUENCE: Type = 23;
}

pub use self::mcosql_column_type::Type as type_t;

pub mod mcosql_transaction_mode {
    pub type Type = u32;
    pub const TM_READ_ONLY: Type = 0;
    pub const TM_UPDATE: Type = 1;
    pub const TM_READ_WRITE: Type = 2;
    pub const TM_EXCLUSIVE: Type = 3;
}

pub mod mcosql_transaction_isolation_level {
    pub type Type = u32;
    pub const TL_DEFAULT: Type = 0;
    pub const TL_READ_COMMITTED: Type = 1;
    pub const TL_REPEATABLE_READ: Type = 2;
    pub const TL_SERIALIZABLE: Type = 3;
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mco_sql {
    _unused: [u8; 0],
}
pub type database_t = *mut mco_sql;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mco_storage {
    _unused: [u8; 0],
}

pub type storage_t = *mut mco_storage;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sql_cursor {
    _unused: [u8; 0],
}

pub type cursor_t = *mut sql_cursor;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct data_source {
    _unused: [u8; 0],
}

pub type data_source_t = *mut data_source;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct record {
    _unused: [u8; 0],
}

pub type record_t = *mut record;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct transaction {
    _unused: [u8; 0],
}

pub type transaction_t = *mut transaction;

pub type mcosql_rs_allocator = *mut ::std::os::raw::c_void;

pub type mcosql_rs_session = *mut ::std::os::raw::c_void;

pub type mcosql_rs_value = *mut ::std::os::raw::c_void;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mcosql_rs_value_ref_ {
    pub allocator: mcosql_rs_allocator,
    pub ref_: mcosql_rs_value,
}

pub type mcosql_rs_value_ref = mcosql_rs_value_ref_;

extern "C" {
    pub fn mcoapi_create_engine(db: mco_db_h, database: *mut database_t) -> status_t;

    pub fn mcoapi_destroy_engine(storage: database_t) -> status_t;

    pub fn mcosql_begin_transaction(
        database: database_t,
        trans: *mut transaction_t,
        mode: mcosql_transaction_mode::Type,
        priority: ::std::os::raw::c_int,
    ) -> status_t;

    pub fn mcosql_commit_transaction(trans: transaction_t) -> status_t;

    pub fn mcosql_rollback_transaction(trans: transaction_t) -> status_t;

    pub fn mcosql_release_transaction(trans: transaction_t) -> status_t;

    pub fn mcosql_get_number_of_columns(
        data_source: data_source_t,
        n_columns: *mut size_t,
    ) -> status_t;

    pub fn mcosql_get_column_info(
        data_source: data_source_t,
        col_no: size_t,
        type_: *mut type_t,
        name: *mut *mut ::std::os::raw::c_char,
    ) -> status_t;

    pub fn mcosql_get_cursor(data_source: data_source_t, cursor: *mut cursor_t) -> status_t;

    pub fn mcosql_cursor_move_next(cursor: cursor_t, record: *mut record_t) -> status_t;

    pub fn mcosql_release_query_result(data_source: data_source_t) -> status_t;
}

extern "C" {
    pub fn mcosql_rs_session_create(
        database: database_t,
        session: *mut mcosql_rs_session,
    ) -> status_t;

    pub fn mcosql_rs_session_destroy(session: mcosql_rs_session) -> status_t;

    pub fn mcosql_rs_database_allocator(
        database: database_t,
        allocator: *mut mcosql_rs_allocator,
    ) -> status_t;

    pub fn mcosql_rs_transaction_allocator(
        transaction: transaction_t,
        allocator: *mut mcosql_rs_allocator,
    ) -> status_t;

    pub fn mcosql_rs_allocator_create(allocator: *mut mcosql_rs_allocator) -> status_t;

    pub fn mcosql_rs_allocator_destroy(allocator: mcosql_rs_allocator) -> status_t;

    pub fn mcosql_rs_value_create_null(out: *mut mcosql_rs_value) -> status_t;

    pub fn mcosql_rs_value_create_bool(
        val: ::std::os::raw::c_int,
        out: *mut mcosql_rs_value,
    ) -> status_t;

    pub fn mcosql_rs_value_create_int(
        allocator: mcosql_rs_allocator,
        val: mco_int8,
        out: *mut mcosql_rs_value,
    ) -> status_t;

    pub fn mcosql_rs_value_create_real(
        allocator: mcosql_rs_allocator,
        val: f64,
        out: *mut mcosql_rs_value,
    ) -> status_t;

    pub fn mcosql_rs_value_create_datetime(
        allocator: mcosql_rs_allocator,
        val: mco_datetime,
        out: *mut mcosql_rs_value,
    ) -> status_t;

    pub fn mcosql_rs_value_create_numeric(
        allocator: mcosql_rs_allocator,
        val: mco_int8,
        prec: size_t,
        out: *mut mcosql_rs_value,
    ) -> status_t;

    pub fn mcosql_rs_value_create_string(
        allocator: mcosql_rs_allocator,
        p: *const ::std::os::raw::c_char,
        len: size_t,
        out: *mut mcosql_rs_value,
    ) -> status_t;

    pub fn mcosql_rs_value_create_binary(
        allocator: mcosql_rs_allocator,
        p: *const ::std::os::raw::c_void,
        len: size_t,
        out: *mut mcosql_rs_value,
    ) -> status_t;

    pub fn mcosql_rs_value_create_array(
        allocator: mcosql_rs_allocator,
        elem_type: type_t,
        size: size_t,
        out: *mut mcosql_rs_value,
    ) -> status_t;

    pub fn mcosql_rs_value_type(val: mcosql_rs_value, out: *mut type_t) -> status_t;

    pub fn mcosql_rs_value_size(val: mcosql_rs_value, out: *mut size_t) -> status_t;

    pub fn mcosql_rs_value_is_null(val: mcosql_rs_value) -> ::std::os::raw::c_int;

    pub fn mcosql_rs_value_is_true(val: mcosql_rs_value) -> ::std::os::raw::c_int;

    pub fn mcosql_rs_value_int(val: mcosql_rs_value, out: *mut mco_int8) -> status_t;

    pub fn mcosql_rs_value_real(val: mcosql_rs_value, out: *mut f64) -> status_t;

    pub fn mcosql_rs_value_datetime(val: mcosql_rs_value, out: *mut mco_datetime) -> status_t;

    pub fn mcosql_rs_value_numeric(
        val: mcosql_rs_value,
        out_value: *mut mco_int8,
        out_precision: *mut size_t,
    ) -> status_t;

    pub fn mcosql_rs_value_string_ref(
        val: mcosql_rs_value,
        allocator: mcosql_rs_allocator,
        out: *mut mcosql_rs_value_ref,
    ) -> status_t;

    pub fn mcosql_rs_value_binary(
        val: mcosql_rs_value,
        allocator: mcosql_rs_allocator,
        out: *mut mcosql_rs_value_ref,
    ) -> status_t;

    pub fn mcosql_rs_value_ptr(
        val: mcosql_rs_value,
        out: *mut *mut ::std::os::raw::c_void,
    ) -> status_t;

    pub fn mcosql_rs_array_is_plain(
        array: mcosql_rs_value,
        out: *mut ::std::os::raw::c_int,
    ) -> status_t;

    pub fn mcosql_rs_array_allocator(
        array: mcosql_rs_value,
        allocator: *mut mcosql_rs_allocator,
    ) -> status_t;

    pub fn mcosql_rs_array_elem_type(array: mcosql_rs_value, out: *mut type_t) -> status_t;

    pub fn mcosql_rs_array_get_at(
        array: mcosql_rs_value,
        at: size_t,
        out: *mut mcosql_rs_value_ref,
    ) -> status_t;

    pub fn mcosql_rs_array_set_at(
        array: mcosql_rs_value,
        at: size_t,
        value: mcosql_rs_value,
    ) -> status_t;

    pub fn mcosql_rs_array_set_body(
        array: mcosql_rs_value,
        elems: *const ::std::os::raw::c_void,
        n_elems: size_t,
    ) -> status_t;

    pub fn mcosql_rs_seq_allocator(
        sequence: mcosql_rs_value,
        allocator: *mut mcosql_rs_allocator,
    ) -> status_t;

    pub fn mcosql_rs_seq_count(sequence: mcosql_rs_value, out: *mut size_t) -> status_t;

    pub fn mcosql_rs_seq_elem_type(sequence: mcosql_rs_value, out: *mut type_t) -> status_t;

    pub fn mcosql_rs_seq_get_iterator(sequence: mcosql_rs_value) -> status_t;

    pub fn mcosql_rs_seq_reset(sequence: mcosql_rs_value) -> status_t;

    pub fn mcosql_rs_seq_next(sequence: mcosql_rs_value, out: *mut mcosql_rs_value) -> status_t;

    pub fn mcosql_rs_blob_available(blob: mcosql_rs_value, out: *mut size_t) -> status_t;

    pub fn mcosql_rs_blob_get(
        blob: mcosql_rs_value,
        buf: *mut ::std::os::raw::c_void,
        size: size_t,
        out: *mut size_t,
    ) -> status_t;

    pub fn mcosql_rs_blob_reset(blob: mcosql_rs_value, pos: size_t) -> status_t;

    pub fn mcosql_rs_value_release(
        allocator: mcosql_rs_allocator,
        sql_value: mcosql_rs_value,
    ) -> status_t;

    pub fn mcosql_rs_statement_execute(
        database: database_t,
        transaction: transaction_t,
        n_records: *mut mco_int8,
        sql: *const ::std::os::raw::c_char,
        values: *mut mcosql_rs_value,
        n_values: size_t,
    ) -> status_t;

    pub fn mcosql_rs_query_execute(
        database: database_t,
        transaction: transaction_t,
        data_source: *mut data_source_t,
        sql: *const ::std::os::raw::c_char,
        values: *mut mcosql_rs_value,
        n_values: size_t,
    ) -> status_t;

    pub fn mcosql_rs_record_allocator(
        record: record_t,
        allocator: *mut mcosql_rs_allocator,
    ) -> status_t;

    pub fn mcosql_rs_record_get_column_value_ref(
        record: record_t,
        column_no: size_t,
        out: *mut mcosql_rs_value_ref,
    ) -> status_t;
}
