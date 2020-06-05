// alloc.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

//! Internal SQL allocators.
//!
//! The *e*X*treme*DB SQL subsystem uses custom allocators to optimize
//! performance. These allocators are used to produce SQL values, data sources,
//! records, etc.
//!
//! It is currently impossible to create or reference an SQL allocator using
//! this API. Public allocator API is considered for a future release.

use std::marker::PhantomData;
use std::mem::MaybeUninit;

use crate::sql::{mcosql_error_code, result_from_code};
use crate::{exdb_sys, Result};

/// An allocator reference.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ref<'a> {
    owner: PhantomData<&'a ()>,
    pub(crate) h: exdb_sys::mcosql_rs_allocator,
}

impl<'a> Ref<'a> {
    pub(crate) fn new(alloc: &'a Owned) -> Self {
        Self::from_handle(alloc.h, alloc)
    }

    pub(crate) fn from_handle<T>(h: exdb_sys::mcosql_rs_allocator, _owner: &'a T) -> Self {
        Ref {
            owner: PhantomData,
            h,
        }
    }
}

pub(crate) struct Owned {
    pub(crate) h: exdb_sys::mcosql_rs_allocator,
}

impl Owned {
    pub(crate) fn new() -> Result<Self> {
        let mut h = MaybeUninit::uninit();

        result_from_code(unsafe { exdb_sys::mcosql_rs_allocator_create(h.as_mut_ptr()) }).and(Ok(
            Owned {
                h: unsafe { h.assume_init() },
            },
        ))
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        let rc = unsafe { exdb_sys::mcosql_rs_allocator_destroy(self.h) };
        debug_assert_eq!(mcosql_error_code::SQL_OK, rc);
    }
}
