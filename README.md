# Overview

This directory contains the `extremedb_sys` and `extremedb` Rust crates, which
implement the *e*X*treme*DB wrapper for Rust.

The `extremedb_sys` crate is a low-level FFI wrapper for the *e*X*treme*DB
public API functions.

The `extremedb` crate implements the higher-level wrapper types and functions.

# Building

There is no need to build the crates separately.

Rust applications are expected to import the `extremedb` crate only. The
`extremedb_sys` crate is imported and built as a dependency of `extremedb`.
It contains Rust declarations for the *e*X*treme*DB API functions, and
a Cargo build script which links the appropriate *e*X*treme*DB libraries.

A few prerequisites are required for `extremedb_sys` to be built successfully:

- *e*X*treme*DB must be installed and built;
- Certain environment variables have to be set.

For more information on the build configuration, refer to the `extremedb_sys`
crate documentation.
