// lib.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

//! `extremedb_sys` is a low-level FFI wrapper for the [McObject]'s
//! *e*X*treme*DB database management system libraries.
//!
//! This crate contains Rust declarations of the *e*X*treme*DB API functions and
//! a Cargo build script which locates and links the appropriate *e*X*treme*DB
//! libraries.
//!
//! The exact set of the linked *e*X*treme*DB libraries depends on the
//! configuration. See the discussion below for details.
//!
//! # Configuration
//!
//! Native *e*X*treme*DB applications have to link with a number of
//! *e*X*treme*DB libraries. Both static and dynamic linking is possible.
//! Some of these libraries are interchangeable, and the applications choose
//! the appropriate ones depending on their requirements. For example,
//! the applications can choose one of the supported transaction managers:
//! Exclusive, MURSIW, or MVCC. There are other features which require linking
//! specific libraries. Refer to the *e*X*treme*DB reference manual for more
//! details.
//!
//! This approach is beneficial for the performance of the applications, since
//! they can avoid linking libraries they do not need. Furthermore, there is no
//! dynamic dispatch required: if the application links with the MURSIW
//! transaction manager library, then its functions are called directly.
//!
//! However, this approach is not quite compatible with the Rust's typical build
//! process. Most packages choose to use Cargo *features* to enable optional
//! functionality. The features are designed to be additive, and cannot be
//! mutually exclusive. Because of this, `extremedb_sys` uses environment
//! variables for configuration.
//!
//! ## Environment Variables
//!
//! `extremedb_sys` requires all of the following variables to be set. Since the
//! selection of the appropriate features is critical for the correct
//! functioning of the applications, these variables have no default values.
//! If any of them is missing, the build process is aborted.
//!
//! - **`MCO_ROOT`**: path to the *e*X*treme*DB root directory.
//! - **`MCORS_CFG_DYLIB`**: defines how the *e*X*treme*DB libraries are to be
//! linked:
//!     - `0`: statically;
//!     - `1`: dynamically.
//! - **`MCORS_CFG_DPTR`**: selects the *e*X*treme*DB implementation libraries:
//!     - `0`: offset libraries;
//!     - `1`: direct pointer libraries.
//! - **`MCORS_CFG_DISK`**: configures database persistence:
//!     - `0`: in-memory database;
//!     - `1`: disk or mixed mode database.
//! - **`MCORS_CFG_SHMEM`**: configures shared memory support:
//!     - `0`: conventional memory;
//!     - `1`: shared memory.
//! - **`MCORS_CFG_TMGR`**: selects the transaction manager library:
//!     - `excl` (exclusive);
//!     - `mursiw`;
//!     - `mvcc`.
//!
//! ## Features
//!
//! Non-mutually-exclusive options can be enabled using the Cargo features.
//!
//! The following features are supported. Applications are not expected to set
//! them explicitly; they are configured by the `extremedb` crate.
//!
//! - **`sequences`** — Sequences (vertical storage).
//! - **`sql`** — SQL engine (local and remote).
//!
//! [McObject]: https://www.mcobject.com

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Fail if any structs or unions are not repr(C)/repr(transparent).
#![deny(improper_ctypes)]

mod core;
pub use crate::core::*;

#[cfg(feature = "sql")]
mod sql;
#[cfg(feature = "sql")]
pub use sql::*;
