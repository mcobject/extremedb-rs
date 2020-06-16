use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io::{LineWriter, Write};
use std::path::{Path, PathBuf};

use bindgen::{self, EnumVariation};

mod api;

fn find_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let dir_iter = fs::read_dir(dir).expect("Failed to read dir");
    for entry in dir_iter {
        let entry = entry.expect("Failed to get dir entry");
        let p = entry.path();

        if p.is_dir() {
            find_rs_files(&p, files);
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            files.push(p.canonicalize().expect("Failed to canonicalize file path"));
        }
    }
}

fn discover_src_files() -> Vec<PathBuf> {
    let mut src = env::current_dir().expect("Failed to get cwd");
    src.push("src");
    let mut ret = Vec::new();
    find_rs_files(&src, &mut ret);
    ret
}

fn bindings_files() -> Vec<PathBuf> {
    let ignore: HashSet<PathBuf> = vec!["src/lib.rs"]
        .iter()
        .map(|f| fs::canonicalize(f).expect("Failed to canonicalize file path"))
        .collect();

    discover_src_files()
        .into_iter()
        .filter(|p| !ignore.contains(p))
        .collect()
}

fn generate_bindings_header() -> String {
    let buf = Vec::new();

    let mut writer = LineWriter::new(buf);

    writer.write_all(b"#ifndef BINDGEN_H_\n").unwrap();

    if cfg!(target_pointer_width = "64") {
        writer.write_all(b"#define MCO_PLATFORM_X64 1\n").unwrap();
    }

    writer.write_all(b"#include \"mco.h\"\n").unwrap();
    writer.write_all(b"#include \"mcocomp.h\"\n").unwrap();

    writer.write_all(b"#include \"sql/sqlrs.h\"\n").unwrap();
    writer.write_all(b"#include \"sql/mcoapic.h\"\n").unwrap();
    writer.write_all(b"#include \"sql/sqlcln_c.h\"\n").unwrap();
    writer.write_all(b"#include \"sql/sqlsrv_c.h\"\n").unwrap();

    writer.write_all(b"#endif /* BINDGEN_H_ */\n").unwrap();

    writer.flush().unwrap();

    String::from_utf8(writer.into_inner().unwrap()).unwrap()
}

fn generate_bindings(mco_inc_dir: &Path) -> String {
    let bindings_header = generate_bindings_header();

    let builder = bindgen::Builder::default()
        .clang_arg(String::from("-I") + mco_inc_dir.to_str().unwrap())
        .header_contents("bindgen.h", &bindings_header)
        .default_enum_style(EnumVariation::ModuleConsts)
        .generate_comments(false)
        .layout_tests(false)
        .whitelist_function("mco_.*")
        .whitelist_type("mco_.*")
        .whitelist_type("MCO_.*")
        .whitelist_function("mcoapi_.*")
        .whitelist_function("mcosql_.*")
        .whitelist_function("sqlsrv_.*")
        .whitelist_function("sqlcln_.*");

    builder
        .generate()
        .expect("Unable to generate bindings")
        .to_string()
}

#[test]
fn api_check() {
    // This test parses the extremedb_sys FFI declarations and matches them
    // against bindings generated dynamically by bindgen.
    //
    // The objective of this test is to make sure that no breaking changes
    // are introduced in the C APIs which are not reflected in the Rust
    // declarations.

    let mco_root = PathBuf::from(env::var("MCO_ROOT").unwrap());
    let mco_inc = mco_root.join("include");

    let files = bindings_files();
    assert_eq!(files.len(), 3);

    let bindings = generate_bindings(&mco_inc);

    let mut bld_inner = api::Builder::new();
    for fname in files {
        let f = File::open(&fname).expect("Failed to open input file");
        bld_inner.read(f).expect("Failed to parse file");
    }

    let mut bld_outer = api::Builder::new();
    bld_outer.no_strict();
    bld_outer
        .read_file_str(&bindings)
        .expect("Failed to parse bindings");

    let inner = bld_inner.finish().expect("Failed to parse static bindings");
    let outer = bld_outer
        .finish()
        .expect("Failed to parse generated bindings");

    let matcher = api::Matcher::new();
    matcher.match_apis(&inner, &outer).expect("API mismatch");
}
