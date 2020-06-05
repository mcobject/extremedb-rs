// rsql.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

use crate::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sqlsrv {
    _unused: [u8; 0],
}

pub type sqlsrv_t = *mut sqlsrv;

pub type sqlsrv_error_handler_t =
    ::std::option::Option<unsafe extern "C" fn(msg: *const ::std::os::raw::c_char)>;

extern "C" {
    pub fn sqlsrv_create(
        server: *mut sqlsrv_t,
        engine: storage_t,
        port: ::std::os::raw::c_int,
        bufferSize: usize,
        nThreads: usize,
        listenQueueSize: ::std::os::raw::c_int,
        handler: sqlsrv_error_handler_t,
    ) -> status_t;

    pub fn sqlsrv_start(server: sqlsrv_t) -> status_t;

    pub fn sqlsrv_stop(server: sqlsrv_t) -> status_t;

    pub fn sqlsrv_destroy(server: sqlsrv_t) -> status_t;
}

extern "C" {
    pub fn sqlcln_create(database: *mut database_t, txBufSize: usize) -> status_t;

    pub fn sqlcln_open(
        database: database_t,
        hostname: *const ::std::os::raw::c_char,
        port: ::std::os::raw::c_int,
        maxConnectAttempts: ::std::os::raw::c_int,
    ) -> status_t;

    pub fn sqlcln_close(database: database_t) -> status_t;

    pub fn sqlcln_destroy(database: database_t) -> status_t;
}
