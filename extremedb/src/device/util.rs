// util.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

//! Device utilities.
//!
//! This module is currently intended to be used by doctests only. It will be
//! compiled conditionally when (if) Rust [issue 67295] is fixed.
//!
//! [issue 67295]: https://github.com/rust-lang/rust/issues/67295

use std::env;
use std::fs;
use std::path::Path;

use crate::device::{Assignment, Device, FileOpenFlags, NamedMemFlags};
use crate::{runtime::Runtime, Result};

#[cfg(target_os = "macos")]
const MCO_DATABASE_DEFAULT_MAP_ADDRESS: usize = 0x2_0000_0000;

#[cfg(not(target_os = "macos"))]
const MCO_DATABASE_DEFAULT_MAP_ADDRESS: usize = 0x2000_0000;

const MAX_SHMEM_NAME_LEN: usize = 32;
const MAX_FILE_NAME_LEN: usize = 128;

/// A device container.
///
/// Creates and maintains a list of devices depending on the current runtime
/// configuration. Cleans up database files when dropped.
///
/// This type is not designed for production use. It will panic on failures.
pub struct DeviceContainer {
    devs: Vec<Device>,
}

impl DeviceContainer {
    /// Creates a new device container for the current runtime configuration.
    pub fn new() -> Self {
        let rt_info = Runtime::info_impl();

        let mut ret = DeviceContainer { devs: Vec::new() };

        if rt_info.disk_supported() {
            let stem = "rstest";

            // Delete any stale files.
            remove_db_files(&stem);

            ret.devs.push(new_test_mem_dev(Assignment::Database));
            ret.devs.push(new_test_mem_dev(Assignment::Cache));
            ret.devs.push(new_test_file_dev(
                Assignment::Persistent,
                Some(db_file_name(&stem)),
            ));
            ret.devs.push(new_test_file_dev(
                Assignment::Log,
                Some(log_file_name(&stem)),
            ));
        } else {
            ret.devs.push(new_test_mem_dev(Assignment::Database));
        }

        ret
    }

    /// Returns a reference to the contained devices.
    pub fn devices(&mut self) -> &mut Vec<Device> {
        &mut self.devs
    }
}

impl Drop for DeviceContainer {
    fn drop(&mut self) {
        for dev in &self.devs {
            if let Some(file_name) = dev.file_name() {
                fs::remove_file(file_name)
                    .expect(&format!("Failed to remove database file {}", file_name));
            }
        }
    }
}

/// Creates a new named memory device if shared memory is supported,
/// conventional memory device otherwise.
pub fn new_mem_dev(a: Assignment, s: usize, name: &str) -> Result<Device> {
    let info = Runtime::info_impl();

    if info.multiprocess_access_supported() {
        let hint = if info.direct_pointers_supported() {
            MCO_DATABASE_DEFAULT_MAP_ADDRESS
        } else {
            0usize
        };
        Device::new_mem_named(a, s, name, NamedMemFlags::new(), hint)
    } else {
        Device::new_mem_conv(a, s)
    }
}

/// Creates a new memory device which must be suitable for most doctests.
///
/// For named devices, the name is inferred from the executable name.
///
/// # Panics
///
/// This function will panic if the device cannot be created, or the name of
/// the shared memory segment cannot be inferred.
pub fn new_test_mem_dev(a: Assignment) -> Device {
    new_mem_dev(a, 1 * 1024 * 1024, &mem_dev_name(a))
        .expect("Failed to create the default memory device")
}

/// Creates a new file device which must be suitable for most doctests.
///
/// If `name` is `None`, the name of the file is inferred from the executable
/// name.
///
/// # Panics
///
/// This function will panic if the device cannot be created, or the name of
/// the file cannot be produced.
pub fn new_test_file_dev(a: Assignment, name: Option<String>) -> Device {
    let file_name = name.unwrap_or_else(|| file_name(&db_file_name_stem(), a));

    Device::new_file(a, FileOpenFlags::new(), &file_name)
        .expect("Failed to create the default disk device")
}

fn exec_name() -> String {
    let path = env::args_os().next().expect("Unknown executable path");
    let name = Path::new(&path)
        .file_name()
        .expect("Unknown executable name");
    name.to_str().expect("Invalid executable name").to_string()
}

fn mem_dev_name(a: Assignment) -> String {
    let mut name = exec_name();
    name.truncate(MAX_SHMEM_NAME_LEN);
    match a {
        Assignment::Database => format!("{}-db", name),
        Assignment::Cache => format!("{}-cache", name),
        _ => panic!("Unexpected memory device assignment"),
    }
}

fn db_file_name_stem() -> String {
    let mut exec = exec_name();
    exec.truncate(MAX_FILE_NAME_LEN);
    let mut stem = exec.clone();
    let mut i = 1usize;

    while Path::new(&db_file_name(&stem)).exists() || Path::new(&log_file_name(&stem)).exists() {
        stem = format!("{}-{}", exec, i);
        i += 1;
    }

    stem
}

fn file_name(stem: &str, a: Assignment) -> String {
    match a {
        Assignment::Persistent => db_file_name(stem),
        Assignment::Log => log_file_name(stem),
        _ => panic!("Unexpected file device assignment"),
    }
}

fn db_file_name(stem: &str) -> String {
    format!("{}.dbs", stem)
}

fn log_file_name(stem: &str) -> String {
    format!("{}.log", stem)
}

fn remove_db_files(stem: &str) {
    for a in &[Assignment::Persistent, Assignment::Log] {
        let name = file_name(stem, *a);
        if Path::new(&name).exists() {
            fs::remove_file(&name).expect(&format!("Failed to remove {}", name));
        }
    }
}
