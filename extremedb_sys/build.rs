// build.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json;

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

const MCO_API_VER_CFG_KEY: &str = "mco_api_ver";

#[derive(Debug, Deserialize)]
struct Features {
    #[serde(rename = "MCO_API_VERSION")]
    ver_api: u32,
    #[serde(rename = "MCO_PRODUCT_BUILD")]
    build: u32,
    #[serde(rename = "MCO_PRODUCT_MAGIC")]
    magic: u32,
    #[serde(rename = "MCO_PRODUCT_REVISION")]
    rev: String,
    #[serde(rename = "MCO_PRODUCT_VERSION_MAJOR")]
    ver_major: u32,
    #[serde(rename = "MCO_PRODUCT_VERSION_MINOR")]
    ver_minor: u32,

    #[serde(rename = "MCO_CFG_WRAPPER_CLUSTER_SUPPORT")]
    cluster: bool,
    #[serde(rename = "MCO_CFG_WRAPPER_HA_SUPPORT")]
    ha: bool,
    #[serde(rename = "MCO_CFG_WRAPPER_IOT_SUPPORT")]
    iot: bool,
    #[serde(rename = "MCO_CFG_WRAPPER_LUAUDF_SUPPORT")]
    lua_udf: bool,
    #[serde(rename = "MCO_CFG_WRAPPER_PERFMON_SUPPORT")]
    perfmon: bool,
    #[serde(rename = "MCO_CFG_WRAPPER_SEQUENCE_SUPPORT")]
    seq: bool,
    #[serde(rename = "MCO_CFG_WRAPPER_SQL_SUPPORT")]
    sql: bool,
    #[serde(rename = "MCO_CFG_WRAPPER_TL_SUPPORT")]
    tl: bool,
}

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
    rsql: bool,
    features: Option<Features>,
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
            rsql: cfg!(feature = "rsql"),
            features: BuildConfig::read_features(),
        }
    }

    fn read_features() -> Option<Features> {
        let path = Path::new(&BuildConfig::get_env(ENV_MCO_ROOT)).join("include/mcofeatures.json");
        let f = File::open(path).ok()?;
        Some(serde_json::from_reader(f).unwrap())
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
    }

    if cfg.rsql {
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

fn output_api_ver_string(suffix: &str, api_ver: u32) {
    println!(
        "cargo:rustc-cfg={}_{}=\"{}\"",
        MCO_API_VER_CFG_KEY, suffix, api_ver
    );
}

fn output_api_ver_ge(api_ver: u32) {
    output_api_ver_string("ge", api_ver);
}

fn output_api_ver_config(api_ver: u32) {
    output_api_ver_string("eq", api_ver);

    if api_ver >= 13 {
        output_api_ver_ge(13);
    }
}

fn main() {
    if env::var("DOCS_RS").unwrap_or(String::from("")) == "1" {
        return;
    }

    config_cargo_rerun();

    let build_cfg = BuildConfig::create();

    let mco_root = PathBuf::from(env::var(ENV_MCO_ROOT).unwrap());
    let mco_lib = mco_root.join(mco_libraries_subdir(&build_cfg));

    output_libraries(&build_cfg, &mco_lib);

    build_cfg.features.map(|f| output_api_ver_config(f.ver_api));
}
