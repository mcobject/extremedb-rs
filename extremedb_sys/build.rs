// build.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

use bindgen::EnumVariation;
use std::env;
use std::io::{LineWriter, Write};
use std::path::{Path, PathBuf};

enum TransactionManager {
    Exclusive,
    MURSIW,
    MVCC,
}

const TMGR_EXCL: &str = "excl";
const TMGR_MURSIW: &str = "mursiw";
const TMGR_MVCC: &str = "mvcc";

const ENV_MCO_ROOT: &str = "MCO_ROOT";
const ENV_CFG_DYLIB: &str = "MCORS_CFG_DYLIB";
const ENV_CFG_DPTR: &str = "MCORS_CFG_DPTR";
const ENV_CFG_DISK: &str = "MCORS_CFG_DISK";
const ENV_CFG_SHMEM: &str = "MCORS_CFG_SHMEM";
const ENV_CFG_TMGR: &str = "MCORS_CFG_TMGR";

struct BuildConfig {
    debug: bool,
    x64: bool,
    direct_ptr: bool,
    link_shared: bool,
    trans_mgr: TransactionManager,
    persistent: bool,
    shared_mem: bool,
    sequences: bool,
    sql: bool,
}

impl BuildConfig {
    fn create() -> Self {
        let link_shared = BuildConfig::get_env_bool(ENV_CFG_DYLIB);
        let direct_ptr = BuildConfig::get_env_bool(ENV_CFG_DPTR);
        let persistent = BuildConfig::get_env_bool(ENV_CFG_DISK);
        let shared_mem = BuildConfig::get_env_bool(ENV_CFG_SHMEM);
        let trans_mgr_s = BuildConfig::get_env_enum(
            ENV_CFG_TMGR,
            vec![
                TMGR_EXCL.to_string(),
                TMGR_MURSIW.to_string(),
                TMGR_MVCC.to_string(),
            ],
        );

        if direct_ptr && persistent {
            panic!("{} conflicts with {}", ENV_CFG_DPTR, ENV_CFG_DISK)
        }

        let trans_mgr = match trans_mgr_s.as_str() {
            TMGR_EXCL => TransactionManager::Exclusive,
            TMGR_MURSIW => TransactionManager::MURSIW,
            TMGR_MVCC => TransactionManager::MVCC,
            _ => panic!("Unexpected transaction manager"),
        };

        BuildConfig {
            debug: cfg!(debug_assertions),
            x64: cfg!(target_pointer_width = "64"),
            direct_ptr,
            link_shared,
            trans_mgr,
            persistent,
            shared_mem,
            sequences: cfg!(feature = "sequences"),
            sql: cfg!(feature = "sql"),
        }
    }

    fn get_env_bool(name: &str) -> bool {
        let val = BuildConfig::get_env(name);
        match val.as_str() {
            "0" => false,
            "1" => true,
            _ => panic!("${}: not a boolean value", name),
        }
    }

    fn get_env_enum(name: &str, values: Vec<String>) -> String {
        let val = BuildConfig::get_env(name);
        if values.contains(&val) {
            val
        } else {
            panic!("${}: unexpected value {}", name, val)
        }
    }

    fn get_env(name: &str) -> String {
        env::var(name).unwrap_or_else(|_| panic!("environment variable not set: {}", name))
    }
}

fn mco_libraries_subdir(cfg: &BuildConfig) -> String {
    String::from(match (cfg.direct_ptr, cfg.link_shared) {
        (false, false) => "target/bin",
        (false, true) => "target/bin.so",
        (true, false) => "target/bin.dptr",
        (true, true) => "target/bin.dptr.so",
    })
}

fn mco_libraries(cfg: &BuildConfig) -> Vec<String> {
    let mut ret = vec![];

    if cfg.sql {
        ret.push("mcosql");
        ret.push("mcorsql");
    }

    if cfg.sequences {
        ret.push("mcoseq");
        ret.push("mcoseqmath");
    }

    let tmgr_lib = match cfg.trans_mgr {
        TransactionManager::Exclusive => "mcotexcl",
        TransactionManager::MURSIW => "mcotmursiw",
        TransactionManager::MVCC => "mcotmvcc",
    };

    ret.push(tmgr_lib);

    ret.extend(vec![
        "mcoseri",
        "mcolib",
        "mcocryptstub",
        "mcotrace",
        "mcosallatches",
        "mcosalatomic",
        "mcosaltimer",
        "mcosalsmp",
        "mcosalmem",
        "mcosaldload",
        "mconet",
        "mcobackup",
        "mcouwrt",
    ]);

    let disk_lib = if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        "mcofu98zip"
    } else if cfg!(target_os = "windows") {
        "mcofw32"
    } else {
        panic!("No shared memory library for this operating system")
    };

    if cfg.persistent {
        ret.push("mcovtdsk");
        ret.push(disk_lib);
    } else {
        ret.push("mcovtmem");
    }

    let shmem_lib = if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        "mcomipc"
    } else if cfg!(target_os = "windows") {
        "mcomw32"
    } else {
        panic!("No shared memory library for this operating system")
    };

    if cfg.shared_mem {
        ret.push(shmem_lib);
    } else {
        ret.push("mcomconv");
    }

    let sync_lib = if cfg!(target_os = "macos") {
        "mcosmacos"
    } else if cfg!(target_os = "linux") {
        "mcoslnxp"
    } else if cfg!(target_os = "windows") {
        "mcosw32"
    } else {
        panic!("No sync library for this operating system");
    };

    ret.push(sync_lib);

    let suffix = String::from(if cfg.debug { "_debug" } else { "" });

    ret.into_iter()
        .map(|lib| lib.to_string() + &suffix)
        .collect()
}

fn cpp_stdlib() -> Option<String> {
    if cfg!(target_os = "linux") {
        Some("stdc++".to_string())
    } else if cfg!(target_os = "macos") {
        Some("c++".to_string())
    } else {
        None
    }
}

fn mco_defines(cfg: &BuildConfig) -> Vec<(String, Option<String>)> {
    let mut ret = Vec::new();

    ret.push((String::from("MCO_DICT_CONST"), Some(String::from("const"))));

    if cfg.x64 {
        ret.push((String::from("MCO_PLATFORM_X64"), None));
    }

    if cfg.direct_ptr {
        ret.push((String::from("MCO_CFG_USE_DIRECT_POINTERS"), None));
    }

    ret
}

fn generate_bindings_header(build_cfg: &BuildConfig) -> String {
    let buf = Vec::new();

    let mut writer = LineWriter::new(buf);

    writer.write_all(b"#ifndef BINDGEN_H_\n").unwrap();

    let defines = mco_defines(build_cfg);
    for (def, val) in defines {
        let s = match val {
            Some(v) => format!("#define {} {}\n", def, v),
            None => format!("#define {}\n", def),
        };
        writer.write_all(s.as_bytes()).unwrap();
    }

    writer.write_all(b"#include \"mco.h\"\n").unwrap();
    writer.write_all(b"#include \"mcocomp.h\"\n").unwrap();

    if build_cfg.sql {
        writer.write_all(b"#include \"sql/mcoapic.h\"\n").unwrap();
        writer.write_all(b"#include \"sql/sqlcln_c.h\"\n").unwrap();
        writer.write_all(b"#include \"sql/sqlsrv_c.h\"\n").unwrap();
        writer.write_all(b"#include \"sql/sqlrs.h\"\n").unwrap();
    }

    writer.write_all(b"#endif /* BINDGEN_H_ */\n").unwrap();

    writer.flush().unwrap();

    String::from_utf8(writer.into_inner().unwrap()).unwrap()
}

fn generate_bindings(build_cfg: &BuildConfig, out_dir: &Path, mco_inc_dir: &Path) {
    let bindings_header = generate_bindings_header(build_cfg);

    let builder = bindgen::Builder::default()
        .clang_arg(String::from("-I") + mco_inc_dir.to_str().unwrap())
        .header_contents("bindgen.h", &bindings_header)
        .default_enum_style(EnumVariation::ModuleConsts)
        .generate_comments(false)
        .whitelist_function("mco_.*")
        .whitelist_type("mco_.*")
        .whitelist_type("MCO_.*");

    let builder = if build_cfg.sql {
        builder
            .whitelist_function("mcoapi_.*")
            .whitelist_function("mcosql_.*")
            .whitelist_function("sqlsrv_.*")
            .whitelist_function("sqlcln_.*")
            .whitelist_function("mcors_.*")
            .whitelist_type("mcors_.*")
            .whitelist_type("MCORS_.*")
    } else {
        builder
    };

    let bindings = builder.generate().expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn output_libraries(build_cfg: &BuildConfig, mco_lib_dir: &Path) {
    println!(
        "cargo:rustc-link-search=native={}",
        mco_lib_dir.to_str().unwrap()
    );

    let libs = mco_libraries(build_cfg);
    for lib in libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    if let Some(cpplib) = cpp_stdlib() {
        println!("cargo:rustc-link-lib={}", cpplib);
    }
}

fn config_cargo_rerun() {
    println!("cargo:rerun-if-env-changed={}", ENV_MCO_ROOT);
    println!("cargo:rerun-if-env-changed={}", ENV_CFG_DYLIB);
    println!("cargo:rerun-if-env-changed={}", ENV_CFG_DPTR);
    println!("cargo:rerun-if-env-changed={}", ENV_CFG_DISK);
    println!("cargo:rerun-if-env-changed={}", ENV_CFG_SHMEM);
    println!("cargo:rerun-if-env-changed={}", ENV_CFG_TMGR);
}

fn main() {
    config_cargo_rerun();

    let build_cfg = BuildConfig::create();

    let mco_root = PathBuf::from(env::var(ENV_MCO_ROOT).unwrap());
    let mco_lib = mco_root.join(mco_libraries_subdir(&build_cfg));
    let mco_inc = mco_root.join("include");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    generate_bindings(&build_cfg, &out_dir, &mco_inc);
    output_libraries(&build_cfg, &mco_lib);
}
