// core.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

pub type size_t = usize;
pub type mco_iquad = i64;
pub type mco_uquad = u64;
pub type mco_uint1 = u8;
pub type mco_uint2 = u16;
pub type mco_uint4 = u32;
pub type mco_uint8 = mco_uquad;
pub type mco_int1 = i8;
pub type mco_int2 = i16;
pub type mco_int4 = i32;
pub type mco_int8 = mco_iquad;
pub type mco_bool = ::std::os::raw::c_int;
pub type mco_datetime = mco_uquad;
pub type uint1 = mco_uint1;
pub type uint2 = mco_uint2;
pub type uint4 = mco_uint4;
pub type uint8 = mco_uint8;
pub type int1 = mco_int1;
pub type int2 = mco_int2;
pub type int4 = mco_int4;
pub type mco_offs32_t = uint4;
pub type mco_offs32_sig_t = int4;
pub type mco_counter32_t = uint4;
pub type mco_size_t = usize;
pub type mco_offs_t = usize;
pub type mco_hash_counter_t = usize;

pub mod MCO_RET_E_ {
    pub type Type = u32;
    pub const MCO_S_OK: Type = 0;
    pub const MCO_S_BUSY: Type = 1;
    pub const MCO_S_OVERFLOW: Type = 2;
    pub const MCO_S_UNDERFLOW: Type = 3;
    pub const MCO_S_NOTFOUND: Type = 10;
    pub const MCO_S_CURSOR_END: Type = 11;
    pub const MCO_S_CURSOR_EMPTY: Type = 12;
    pub const MCO_S_DUPLICATE: Type = 13;
    pub const MCO_S_EVENT_RELEASED: Type = 14;
    pub const MCO_S_DEAD_CONNECTION: Type = 15;
    pub const MCO_S_NULL_VALUE: Type = 16;
    pub const MCO_S_TL_INVDATA: Type = 17;
    pub const MCO_S_TL_NOT_INITIALIZED: Type = 18;
    pub const MCO_S_DEFERRED_DELETE: Type = 19;
    pub const MCO_S_REST_CONN_ACCEPTED: Type = 20;
    pub const MCO_S_REST_CONN_FINISHED: Type = 21;
    pub const MCO_S_REST_TIMEOUT: Type = 22;
    pub const MCO_E_CORE: Type = 50;
    pub const MCO_E_INVALID_HANDLE: Type = 51;
    pub const MCO_E_NOMEM: Type = 52;
    pub const MCO_E_ACCESS: Type = 53;
    pub const MCO_E_TRANSACT: Type = 54;
    pub const MCO_E_INDEXLIMIT: Type = 55;
    pub const MCO_E_EMPTYVECTOREL: Type = 56;
    pub const MCO_E_UNSUPPORTED: Type = 57;
    pub const MCO_E_EMPTYOPTIONAL: Type = 58;
    pub const MCO_E_EMPTYBLOB: Type = 59;
    pub const MCO_E_CURSOR_INVALID: Type = 60;
    pub const MCO_E_ILLEGAL_TYPE: Type = 61;
    pub const MCO_E_ILLEGAL_PARAM: Type = 62;
    pub const MCO_E_CURSOR_MISMATCH: Type = 63;
    pub const MCO_E_DELETED: Type = 64;
    pub const MCO_E_LONG_TRANSACTION: Type = 65;
    pub const MCO_E_INSTANCE_DUPLICATE: Type = 66;
    pub const MCO_E_UPGRADE_FAILED: Type = 67;
    pub const MCO_E_NOINSTANCE: Type = 68;
    pub const MCO_E_OPENED_SESSIONS: Type = 69;
    pub const MCO_E_PAGESIZE: Type = 70;
    pub const MCO_E_WRITE_STREAM: Type = 71;
    pub const MCO_E_READ_STREAM: Type = 72;
    pub const MCO_E_LOAD_DICT: Type = 73;
    pub const MCO_E_LOAD_DATA: Type = 74;
    pub const MCO_E_VERS_MISMATCH: Type = 75;
    pub const MCO_E_VOLUNTARY_NOT_EXIST: Type = 76;
    pub const MCO_E_EXCLUSIVE_MODE: Type = 77;
    pub const MCO_E_MAXEXTENDS: Type = 78;
    pub const MCO_E_HIST_OBJECT: Type = 79;
    pub const MCO_E_SHM_ERROR: Type = 80;
    pub const MCO_E_NOTINIT: Type = 81;
    pub const MCO_E_SESLIMIT: Type = 82;
    pub const MCO_E_INSTANCES_LIMIT: Type = 83;
    pub const MCO_E_MAXTRANSSIZE_LOCKED: Type = 84;
    pub const MCO_E_DEPRECATED: Type = 85;
    pub const MCO_E_NOUSERDEF_FUNCS: Type = 86;
    pub const MCO_E_CONFLICT: Type = 87;
    pub const MCO_E_INMEM_ONLY_RUNTIME: Type = 88;
    pub const MCO_E_ISOLATION_LEVEL_NOT_SUPPORTED: Type = 89;
    pub const MCO_E_REGISTRY_UNABLE_CREATE_CONNECT: Type = 90;
    pub const MCO_E_REGISTRY_UNABLE_CONNECT: Type = 91;
    pub const MCO_E_REGISTRY_INVALID_SYNC: Type = 92;
    pub const MCO_E_MDEV_RUNTIME_START: Type = 93;
    pub const MCO_E_SYNC_RUNTIME_START: Type = 94;
    pub const MCO_E_ALIGN_ERROR: Type = 95;
    pub const MCO_E_PINNED_VERSION_LIMIT: Type = 96;
    pub const MCO_E_VERSION_NOT_PINNED: Type = 97;
    pub const MCO_E_CURSOR_CLOSED: Type = 98;
    pub const MCO_E_CONVERSION: Type = 99;
    pub const MCO_E_DISK: Type = 100;
    pub const MCO_E_DISK_OPEN: Type = 101;
    pub const MCO_E_DISK_ALREADY_OPENED: Type = 102;
    pub const MCO_E_DISK_NOT_OPENED: Type = 103;
    pub const MCO_E_DISK_INVALID_PARAM: Type = 104;
    pub const MCO_E_DISK_PAGE_ACCESS: Type = 105;
    pub const MCO_E_DISK_OPERATION_NOT_ALLOWED: Type = 106;
    pub const MCO_E_DISK_ALREADY_CONNECTED: Type = 107;
    pub const MCO_E_DISK_KEY_TOO_LONG: Type = 108;
    pub const MCO_E_DISK_TOO_MANY_INDICES: Type = 109;
    pub const MCO_E_DISK_TOO_MANY_CLASSES: Type = 110;
    pub const MCO_E_DISK_SPACE_EXHAUSTED: Type = 111;
    pub const MCO_E_DISK_INCOMPATIBLE_LOG_TYPE: Type = 112;
    pub const MCO_E_DISK_BAD_PAGE_SIZE: Type = 113;
    pub const MCO_E_DISK_SYNC: Type = 114;
    pub const MCO_E_DISK_PAGE_POOL_EXHAUSTED: Type = 115;
    pub const MCO_E_DISK_CLOSE: Type = 116;
    pub const MCO_E_DISK_TRUNCATE: Type = 117;
    pub const MCO_E_DISK_SEEK: Type = 118;
    pub const MCO_E_DISK_WRITE: Type = 119;
    pub const MCO_E_DISK_READ: Type = 120;
    pub const MCO_E_DISK_FLUSH: Type = 121;
    pub const MCO_E_DISK_TOO_HIGH_TREE: Type = 122;
    pub const MCO_E_DISK_VERSION_MISMATCH: Type = 123;
    pub const MCO_E_DISK_CONFLICT: Type = 124;
    pub const MCO_E_DISK_SCHEMA_CHANGED: Type = 125;
    pub const MCO_E_DISK_CRC_MISMATCH: Type = 126;
    pub const MCO_E_DISK_TM_MISMATCH: Type = 127;
    pub const MCO_E_DISK_DICT_LIMITS_MISMATCH: Type = 128;
    pub const MCO_E_DISK_BTREE_ALLOC: Type = 129;
    pub const MCO_E_DISK_CRC_CHECK_MODE_MATCH: Type = 130;

    #[cfg(mco_api_ver_ge = "13")]
    pub const MCO_E_DISK_FATAL_ERROR: Type = 131;
    #[cfg(mco_api_ver_ge = "13")]
    pub const MCO_E_DISK_ALLOC_MISMATCH: Type = 132;

    pub const MCO_E_XML: Type = 200;
    pub const MCO_E_XML_INVINT: Type = 201;
    pub const MCO_E_XML_INVFLT: Type = 202;
    pub const MCO_E_XML_INTOVF: Type = 203;
    pub const MCO_E_XML_INVBASE: Type = 204;
    pub const MCO_E_XML_BUFSMALL: Type = 205;
    pub const MCO_E_XML_VECTUNSUP: Type = 206;
    pub const MCO_E_XML_INVPOLICY: Type = 207;
    pub const MCO_E_XML_INVCLASS: Type = 208;
    pub const MCO_E_XML_NO_OID: Type = 209;
    pub const MCO_E_XML_INVOID: Type = 210;
    pub const MCO_E_XML_INVFLDNAME: Type = 211;
    pub const MCO_E_XML_FLDNOTFOUND: Type = 212;
    pub const MCO_E_XML_INVENDTAG: Type = 213;
    pub const MCO_E_XML_UPDID: Type = 214;
    pub const MCO_E_XML_INVASCII: Type = 215;
    pub const MCO_E_XML_INCOMPL: Type = 216;
    pub const MCO_E_XML_ARRSMALL: Type = 217;
    pub const MCO_E_XML_INVARREL: Type = 218;
    pub const MCO_E_XML_EXTRAXML: Type = 219;
    pub const MCO_E_XML_NOTWF: Type = 220;
    pub const MCO_E_XML_UNICODE: Type = 221;
    pub const MCO_E_XML_NOINDEX: Type = 222;
    pub const MCO_E_NW: Type = 300;
    pub const MCO_E_NW_FATAL: Type = 301;
    pub const MCO_E_NW_NOTSUPP: Type = 302;
    pub const MCO_E_NW_CLOSE_CHANNEL: Type = 303;
    pub const MCO_E_NW_BUSY: Type = 304;
    pub const MCO_E_NW_ACCEPT: Type = 305;
    pub const MCO_E_NW_TIMEOUT: Type = 306;
    pub const MCO_E_NW_INVADDR: Type = 307;
    pub const MCO_E_NW_NOMEM: Type = 308;
    pub const MCO_E_NW_CONNECT: Type = 309;
    pub const MCO_E_NW_SENDERR: Type = 310;
    pub const MCO_E_NW_RECVERR: Type = 311;
    pub const MCO_E_NW_CLOSED: Type = 312;
    pub const MCO_E_NW_HANDSHAKE: Type = 313;
    pub const MCO_E_NW_CLOSE_SOCKET: Type = 314;
    pub const MCO_E_NW_CREATEPIPE: Type = 315;
    pub const MCO_E_NW_SOCKET: Type = 316;
    pub const MCO_E_NW_SOCKOPT: Type = 317;
    pub const MCO_E_NW_BIND: Type = 318;
    pub const MCO_E_NW_SOCKIOCTL: Type = 319;
    pub const MCO_E_NW_MAGIC: Type = 320;
    pub const MCO_E_NW_INVMSGPARAM: Type = 321;
    pub const MCO_E_NW_WRONGSEQ: Type = 322;
    pub const MCO_E_NWMCAST_CLOSE_SOCKET: Type = 323;
    pub const MCO_E_NWMCAST_SOCKET: Type = 324;
    pub const MCO_E_NWMCAST_SOCKOPT: Type = 325;
    pub const MCO_E_NWMCAST_RECV: Type = 326;
    pub const MCO_E_NWMCAST_BIND: Type = 327;
    pub const MCO_E_NWMCAST_NBIO: Type = 328;
    pub const MCO_E_NW_KILLED_BY_REPLICA: Type = 329;
    pub const MCO_E_NW_WOULDBLOCK: Type = 330;
    pub const MCO_E_NW_SELECT: Type = 331;
    pub const MCO_E_NW_INVALID_PARAMETER: Type = 332;
    pub const MCO_E_HA: Type = 400;
    pub const MCO_E_HA_PROTOCOLERR: Type = 401;
    pub const MCO_E_HA_TIMEOUT: Type = 402;
    pub const MCO_E_HA_IOERROR: Type = 403;
    pub const MCO_E_HA_MAXREPLICAS: Type = 404;
    pub const MCO_E_HA_INIT: Type = 405;
    pub const MCO_E_HA_RECEIVE: Type = 406;
    pub const MCO_E_HA_NO_AUTO_OID: Type = 407;
    pub const MCO_E_HA_NOT_INITIALIZED: Type = 408;
    pub const MCO_E_HA_INVALID_MESSAGE: Type = 409;
    pub const MCO_E_HA_INVALID_PARAMETER: Type = 410;
    pub const MCO_E_HA_INVCHANNEL: Type = 411;
    pub const MCO_E_HA_INCOMPATIBLE_MODES: Type = 412;
    pub const MCO_E_HA_CLOSE_TEMP: Type = 413;
    pub const MCO_E_HA_MULTICAST_NOT_SUPP: Type = 414;
    pub const MCO_E_HA_HOTSYNCH_NOT_SUPP: Type = 415;
    pub const MCO_E_HA_ASYNCH_NOT_SUPP: Type = 416;
    pub const MCO_E_HA_NO_MEM: Type = 417;
    pub const MCO_E_HA_BAD_DESCRIPTOR: Type = 418;
    pub const MCO_E_HA_CANCEL: Type = 419;
    pub const MCO_E_HA_WRONG_DB_MAGIC: Type = 420;
    pub const MCO_E_HA_COMMIT: Type = 421;
    pub const MCO_E_HA_MANYREPLICAS: Type = 422;
    pub const MCO_E_NOT_MASTER: Type = 423;
    pub const MCO_E_HA_STOPPED: Type = 424;
    pub const MCO_E_HA_NOWRITETXN: Type = 425;
    pub const MCO_E_HA_PM_BUFFER: Type = 426;
    pub const MCO_E_HA_NOT_REPLICA: Type = 427;
    pub const MCO_E_HA_BAD_DICT: Type = 428;
    pub const MCO_E_HA_BINEV_NOT_SUPP: Type = 429;
    pub const MCO_E_HA_CHANNEL_NOT_REGISTERED: Type = 430;
    pub const MCO_E_HA_DDL_NOT_SUPPORTED: Type = 431;
    pub const MCO_E_HA_NO_QUORUM: Type = 432;
    pub const MCO_S_HA_REPLICA_DETACH: Type = 433;
    pub const MCO_E_UDA: Type = 500;
    pub const MCO_E_UDA_TOOMANY_ENTRIES: Type = 501;
    pub const MCO_E_UDA_NAME_TOO_LONG: Type = 502;
    pub const MCO_E_UDA_DUPLICATE: Type = 503;
    pub const MCO_E_UDA_DICT_NOTFOUND: Type = 504;
    pub const MCO_E_UDA_STRUCT_NOTFOUND: Type = 505;
    pub const MCO_E_UDA_FIELD_NOTFOUND: Type = 506;
    pub const MCO_E_UDA_INDEX_NOTFOUND: Type = 507;
    pub const MCO_E_UDA_IFIELD_NOTFOUND: Type = 508;
    pub const MCO_E_UDA_COLLATION_NOTFOUND: Type = 509;
    pub const MCO_E_UDA_STRUCT_NOT_CLASS: Type = 510;
    pub const MCO_E_UDA_WRONG_KEY_NUM: Type = 511;
    pub const MCO_E_UDA_WRONG_KEY_TYPE: Type = 512;
    pub const MCO_E_UDA_WRONG_OPCODE: Type = 513;
    pub const MCO_E_UDA_SCALAR: Type = 514;
    pub const MCO_E_UDA_NOT_DYNAMIC: Type = 515;
    pub const MCO_E_UDA_WRONG_VALUE_TYPE: Type = 516;
    pub const MCO_E_UDA_READONLY: Type = 517;
    pub const MCO_E_UDA_WRONG_CLASS_CODE: Type = 518;
    pub const MCO_E_UDA_DICT_NOT_DIRECT: Type = 519;
    pub const MCO_E_UDA_INDEX_NOT_USERDEF: Type = 520;
    pub const MCO_E_UDA_EVENT_NOTFOUND: Type = 521;
    pub const MCO_E_TL: Type = 600;
    pub const MCO_E_TL_INVAL: Type = 601;
    pub const MCO_E_TL_ALREADY_STARTED: Type = 602;
    pub const MCO_E_TL_NOT_STARTED: Type = 603;
    pub const MCO_E_TL_LOG_NOT_OPENED: Type = 604;
    pub const MCO_E_TL_INVFORMAT: Type = 605;
    pub const MCO_E_TL_NOT_INITIALIZED: Type = 606;
    pub const MCO_E_TL_IO_ERROR: Type = 607;
    pub const MCO_E_TL_NOT_ITERABLE: Type = 608;
    pub const MCO_E_TL_TRANS_STARTED: Type = 609;
    pub const MCO_E_TL_PIPE_USED: Type = 610;
    pub const MCO_E_TL_PIPE_LOST: Type = 611;
    pub const MCO_E_TL_PIPE_TERM: Type = 612;
    pub const MCO_E_TL_NO_AUTO_OID: Type = 613;
    pub const MCO_E_TL_NOT_APPLICABLE: Type = 614;
    pub const MCO_E_TL_NO_DYNAMIC_PIPE: Type = 615;
    pub const MCO_E_TL_SYNC: Type = 616;
    pub const MCO_E_TL_PLAY_STOPPED: Type = 617;
    pub const MCO_E_TL_PLAY_NOT_STARTED: Type = 618;
    pub const MCO_E_SEQ_OUT_OF_ORDER: Type = 700;
    pub const MCO_E_SEQ_BOUNDED: Type = 701;
    pub const MCO_E_SEQ_LENGTH_MISMATCH: Type = 702;
    pub const MCO_E_SEQ_NULL_VALUE: Type = 703;
    pub const MCO_E_DDL_NOMEM: Type = 800;
    pub const MCO_E_DDL_UNDEFINED_STRUCT: Type = 801;
    pub const MCO_E_DDL_INVALID_TYPE: Type = 802;
    pub const MCO_E_DDL_FIELD_NOT_FOUND: Type = 803;
    pub const MCO_E_DDL_INTERNAL_ERROR: Type = 804;
    pub const MCO_E_DDL_MCOCOMP_INCOMPATIBILITY: Type = 805;
    pub const MCO_E_DDL_TOO_MANY_CLASSES: Type = 806;
    pub const MCO_E_DDL_TOO_MANY_INDEXES: Type = 807;
    pub const MCO_E_DDL_TOO_MANY_EVENTS: Type = 808;
    pub const MCO_E_CLUSTER: Type = 900;
    pub const MCO_E_CLUSTER_NOT_INITIALIZED: Type = 901;
    pub const MCO_E_CLUSTER_INVALID_PARAMETER: Type = 902;
    pub const MCO_E_CLUSTER_STOPPED: Type = 903;
    pub const MCO_E_CLUSTER_PROTOCOLERR: Type = 904;
    pub const MCO_E_CLUSTER_NOQUORUM: Type = 905;
    pub const MCO_E_CLUSTER_BUSY: Type = 906;
    pub const MCO_E_CLUSTER_INCOMPATIBLE_MODE: Type = 907;
    pub const MCO_E_CLUSTER_SYNC: Type = 908;
    pub const MCO_E_CLUSTER_INCOMPATIBLE_ARCH: Type = 909;
    pub const MCO_E_CLUSTER_DUPLICATE_NODEID: Type = 910;
    pub const MCO_E_CLUSTER_DDL_NOT_SUPPORTED: Type = 911;
    pub const MCO_E_SAL_RUNTIME_START: Type = 912;
    pub const MCO_E_EVAL: Type = 999;
    pub const MCO_E_PERFMON: Type = 1000;
    pub const MCO_E_PERFMON_NOT_INITIALIZED: Type = 1001;
    pub const MCO_E_PERFMON_ALREADY_INITIALIZED: Type = 1002;
    pub const MCO_E_PERFMON_DB_NOT_DETACHED: Type = 1003;
    pub const MCO_E_PERFMON_DB_NOT_ATTACHED: Type = 1004;
    pub const MCO_E_SCHEMA_ERROR: Type = 1005;
    pub const MCO_E_NO_DIRECT_ACCESS: Type = 1006;
    pub const MCO_E_ENCRYPTION_NOT_SUPPORTED: Type = 1007;
    pub const MCO_E_NO_CIPHER_KEY: Type = 1008;
    pub const MCO_E_TOO_HIGH_TREE: Type = 1009;
    pub const MCO_E_KEY_TOO_LONG: Type = 1010;
    pub const MCO_E_PATRICIA_TOO_DEEP: Type = 1011;
    pub const MCO_E_BTREE_CONFLICT: Type = 1012;
    pub const MCO_E_TMGR_MISMATCH: Type = 1013;
    pub const MCO_E_SCHEMA_CHANGED: Type = 1014;
    pub const MCO_E_LICENSE_INVALID: Type = 1015;
    pub const MCO_E_BACKUP: Type = 1016;
    pub const MCO_E_BACKUP_PROTOCOL: Type = 1017;
    pub const MCO_E_BACKUP_NOMEM: Type = 1018;
    pub const MCO_E_BACKUP_INVALID_PARAM: Type = 1019;
    pub const MCO_E_BACKUP_INVALID_FILE: Type = 1020;
    pub const MCO_E_BACKUP_SNAPSHOT_ONLY: Type = 1021;
    pub const MCO_E_INTERRUPTED: Type = 1022;
    pub const MCO_E_TRANS_NOT_CLOSED: Type = 1023;
    pub const MCO_E_TRANS_NOT_ACTIVE: Type = 1024;
    pub const MCO_E_DATETIME_PRECISION_MISMATCH: Type = 1025;

    #[cfg(mco_api_ver_ge = "13")]
    pub const MCO_E_WRONG_CIPHER_KEY: Type = 1026;

    pub const MCO_E_VERIFICATION: Type = 1100;
    pub const MCO_E_IOT: Type = 1200;
    pub const MCO_E_IOT_NOT_INITIALIZED: Type = 1201;
    pub const MCO_E_IOT_INVALID_HANDLE: Type = 1202;
    pub const MCO_E_IOT_WRONG_AGENT_ID: Type = 1203;
    pub const MCO_E_IOT_AGENT_NOT_FOUND: Type = 1204;
    pub const MCO_E_IOT_PROTOCOLERR: Type = 1205;
    pub const MCO_E_IOT_TS_GAP: Type = 1206;
    pub const MCO_E_IOT_TS_OUTOFDATE: Type = 1207;
    pub const MCO_S_IOT_NO_NEW_DATA: Type = 1208;
    pub const MCO_E_IOT_TOO_MANY_CONTEXTS: Type = 1209;
    pub const MCO_E_IOT_DUPLICATE_CALLBACK: Type = 1210;
    pub const MCO_E_IOT_CALLBACK_NOT_FOUND: Type = 1211;
    pub const MCO_E_IOT_INCOMPATIBLE_MODE: Type = 1212;
    pub const MCO_E_IOT_INCOMPATIBLE_LEVEL: Type = 1213;
    pub const MCO_E_IOT_STOPPED: Type = 1214;
    pub const MCO_E_IOT_TIMEOUT: Type = 1215;
    pub const MCO_E_IOT_DDL_NOT_SUPPORTED: Type = 1216;
    pub const MCO_E_REST: Type = 1300;
    pub const MCO_E_REST_SYSTEM: Type = 1301;
    pub const MCO_E_REST_DB: Type = 1302;
    pub const MCO_E_REST_PARAM: Type = 1303;
    pub const MCO_E_REST_HTTP: Type = 1304;
    pub const MCO_E_REST_NOT_FOUND: Type = 1305;
    pub const MCO_E_REST_JSON: Type = 1306;
    pub const MCO_E_REST_INUSE: Type = 1307;
    pub const MCO_E_REST_EOF: Type = 1308;
    pub const MCO_E_REST_ADDRNOTAVAIL: Type = 1309;
    pub const MCO_E_JSER_NOINDEX: Type = 1400;
    pub const MCO_ERR_DB: Type = 100000;
    pub const MCO_ERR_DICT: Type = 110000;
    pub const MCO_ERR_CURSOR: Type = 120000;
    pub const MCO_ERR_PMBUF: Type = 130000;
    pub const MCO_ERR_COMMON: Type = 140000;
    pub const MCO_ERR_HEAP: Type = 150000;
    pub const MCO_ERR_OBJ: Type = 160000;
    pub const MCO_ERR_BLOB: Type = 170000;
    pub const MCO_ERR_FREC: Type = 180000;
    pub const MCO_ERR_VOLUNTARY: Type = 190000;
    pub const MCO_ERR_LOADSAVE: Type = 200000;
    pub const MCO_ERR_PGMEM: Type = 210000;
    pub const MCO_ERR_EV_SYN: Type = 220000;
    pub const MCO_ERR_EV_ASYN: Type = 230000;
    pub const MCO_ERR_EV_W: Type = 240000;
    pub const MCO_ERR_XML_W: Type = 250000;
    pub const MCO_ERR_XML_SC: Type = 260000;
    pub const MCO_ERR_BTREE: Type = 270000;
    pub const MCO_ERR_HASH: Type = 280000;
    pub const MCO_ERR_RECOV: Type = 290000;
    pub const MCO_ERR_FCOPY: Type = 300000;
    pub const MCO_ERR_INST: Type = 330000;
    pub const MCO_ERR_TRN: Type = 340000;
    pub const MCO_ERR_TMGR: Type = 370000;
    pub const MCO_ERR_SYNC: Type = 400000;
    pub const MCO_ERR_ORDER: Type = 450000;
    pub const MCO_ERR_SEM: Type = 460000;
    pub const MCO_ERR_SHM: Type = 470000;
    pub const MCO_ERR_SER: Type = 500000;
    pub const MCO_ERR_HA: Type = 510000;
    pub const MCO_ERR_DB_NOMEM: Type = 520000;
    pub const MCO_ERR_OBJECT_HANDLE: Type = 530000;
    pub const MCO_ERR_UNSUPPORTED_FLOAT: Type = 540000;
    pub const MCO_ERR_UNSUPPORTED_DOUBLE: Type = 550000;
    pub const MCO_ERR_DB_NOMEM_HASH: Type = 560000;
    pub const MCO_ERR_DB_NOMEM_HEAP: Type = 570000;
    pub const MCO_ERR_DB_NOMEM_TRANS: Type = 580000;
    pub const MCO_ERR_DB_NAMELONG: Type = 590000;
    pub const MCO_ERR_DB_VERS_MISMATCH: Type = 600000;
    pub const MCO_ERR_RUNTIME: Type = 610000;
    pub const MCO_ERR_INMEM_ONLY_RUNTIME: Type = 620000;
    pub const MCO_ERR_DISK: Type = 700000;
    pub const MCO_ERR_DISK_WRITE: Type = 710000;
    pub const MCO_ERR_DISK_READ: Type = 720000;
    pub const MCO_ERR_DISK_FLUSH: Type = 730000;
    pub const MCO_ERR_DISK_CLOSE: Type = 740000;
    pub const MCO_ERR_DISK_TRUNCATE: Type = 750000;
    pub const MCO_ERR_DISK_SEEK: Type = 760000;
    pub const MCO_ERR_DISK_OPEN: Type = 770000;
    pub const MCO_ERR_DISK_ALREADY_OPENED: Type = 780000;
    pub const MCO_ERR_DISK_NOT_OPENED: Type = 790000;
    pub const MCO_ERR_DISK_INVALID_PARAM: Type = 800000;
    pub const MCO_ERR_DISK_PAGE_ACCESS: Type = 810000;
    pub const MCO_ERR_DISK_INTERNAL_ERROR: Type = 820000;
    pub const MCO_ERR_DISK_OPERATION_NOT_ALLOWED: Type = 830000;
    pub const MCO_ERR_DISK_ALREADY_CONNECTED: Type = 840000;
    pub const MCO_ERR_DISK_TOO_MANY_INDICES: Type = 850000;
    pub const MCO_ERR_DISK_TOO_MANY_CLASSES: Type = 860000;
    pub const MCO_ERR_DISK_SPACE_EXHAUSTED: Type = 870000;
    pub const MCO_ERR_DISK_PAGE_POOL_EXHAUSTED: Type = 880000;
    pub const MCO_ERR_DISK_INCOMPATIBLE_LOG_TYPE: Type = 890000;
    pub const MCO_ERR_DISK_BAD_PAGE_SIZE: Type = 900000;
    pub const MCO_ERR_DISK_SYNC: Type = 910000;
    pub const MCO_ERR_DISK_CRC: Type = 920000;
    pub const MCO_ERR_DISK_FORMAT_MISMATCH: Type = 930000;
    pub const MCO_ERR_CHECKPIN: Type = 940000;
    pub const MCO_ERR_CONN: Type = 950000;
    pub const MCO_ERR_REGISTRY: Type = 960000;
    pub const MCO_ERR_INDEX: Type = 970000;
    pub const MCO_ERR_VTMEM: Type = 980000;
    pub const MCO_ERR_VTDSK: Type = 990000;
    pub const MCO_ERR_RTREE: Type = 1000000;
    pub const MCO_ERR_UDA: Type = 1010000;
    pub const MCO_ERR_PTREE: Type = 1020000;
    pub const MCO_ERR_TL: Type = 1030000;
    pub const MCO_ERR_CLUSTER: Type = 1040000;
    pub const MCO_ERR_CLNWTCP: Type = 1050000;
    pub const MCO_ERR_SEQ: Type = 1060000;
    pub const MCO_ERR_NESTED_TRANS_TRAP: Type = 1090000;
    pub const MCO_ERR_PERFMON: Type = 1100000;
    pub const MCO_ERR_AIO: Type = 1110000;
    pub const MCO_ERR_CLNWMPI: Type = 1120000;
    pub const MCO_ERR_DDL: Type = 1130000;
    pub const MCO_ERR_SQL_EXCEPTION: Type = 1140000;
    pub const MCO_ERR_BACKUP: Type = 1150000;
    pub const MCO_ERR_ACTIVE_TRANSACTION: Type = 1160000;
    pub const MCO_ERR_NETWORK: Type = 1170000;
    pub const MCO_ERR_IOT_COMM: Type = 1180000;
    pub const MCO_ERR_IOT_REPL: Type = 1190000;
    pub const MCO_ERR_LAST: Type = 1999999;
}

pub use MCO_RET_E_::Type as MCO_RET;

pub mod MCO_TRANS_SCHED_POLICY_E_ {
    pub type Type = u32;
    pub const MCO_SCHED_FIFO: Type = 0;
    pub const MCO_SCHED_READER_FAVOR: Type = 1;
    pub const MCO_SCHED_WRITER_FAVOR: Type = 2;
}

pub use MCO_TRANS_SCHED_POLICY_E_::Type as MCO_TRANS_SCHED_POLICY;

pub mod MCO_COMMIT_POLICY_E {
    pub type Type = u32;
    pub const MCO_COMMIT_SYNC_FLUSH: Type = 0;
    pub const MCO_COMMIT_BUFFERED: Type = 1;
    pub const MCO_COMMIT_DELAYED: Type = 2;
    pub const MCO_COMMIT_NO_SYNC: Type = 3;
}

pub use MCO_COMMIT_POLICY_E::Type as MCO_COMMIT_POLICY;

pub mod MCO_LOG_TYPE_ {
    pub type Type = u32;
    pub const NO_LOG: Type = 0;
    pub const REDO_LOG: Type = 1;
    pub const UNDO_LOG: Type = 2;
}

pub use MCO_LOG_TYPE_::Type as MCO_LOG_TYPE;

pub mod MCO_DB_MODE_MASK_ {
    pub type Type = u32;
    pub const MCO_DB_MODE_MVCC_AUTO_VACUUM: Type = 1;
    pub const MCO_DB_MODE_SMART_INDEX_INSERT: Type = 2;
    pub const MCO_DB_OPEN_EXISTING: Type = 4;
    pub const MCO_DB_USE_CRC_CHECK: Type = 8;
    pub const MCO_DB_TRANSIENT: Type = 16;
    pub const MCO_DB_LAZY_MEM_INITIALIZATION: Type = 32;
    pub const MCO_DB_MURSIW_DISK_COMMIT_OPTIMIZATION: Type = 64;
    pub const MCO_DB_BULK_WRITE_MODIFIED_PAGES: Type = 128;
    pub const MCO_DB_INDEX_PRELOAD: Type = 512;
    pub const MCO_DB_DISABLE_NESTED_TRANSACTIONS: Type = 1024;
    pub const MCO_DB_DISABLE_IMPLICIT_ROLLBACK: Type = 2048;
    pub const MCO_DB_INMEMORY_PROTECTION: Type = 4096;
    pub const MCO_DB_INCLUSIVE_BTREE: Type = 8192;
    pub const MCO_DB_INMEMORY_COMPRESSION: Type = 16384;
    pub const MCO_DB_SEPARATE_BITMAP: Type = 32768;
    pub const MCO_DB_DISABLE_BTREE_REBALANCE_ON_DELETE: Type = 65536;
    pub const MCO_DB_AUTO_ROLLBACK_FIRST_PHASE: Type = 131072;
    pub const MCO_DB_MVCC_COMPATIBILITY_MODE: Type = 262144;
    pub const MCO_DB_DISABLE_PAGE_POOL_RESERVE: Type = 524288;
    pub const MCO_DB_REDO_LOG_OPTIMIZATION: Type = 1048576;
    pub const MCO_DB_DISABLE_HOT_UPDATES: Type = 2097152;
    pub const MCO_DB_SQL_AUTOCHECKPOINT: Type = 4194304;
    pub const MCO_DB_MODE_READ_ONLY: Type = 8388608;
    pub const MCO_DB_USE_AIO: Type = 16777216;

    #[cfg(mco_api_ver_lt = "14")]
    pub const MCO_DB_INCREMENTAL_BACKUP: Type = 33554432;

    #[cfg(mco_api_ver_ge = "14")]
    pub const MCO_DB_INCREMENTAL_BACKUP_ENABLED: Type = 33554432;

    #[cfg(mco_api_ver_ge = "14")]
    pub const MCO_DB_INCREMENTAL_BACKUP_PROCESSING: Type = 67108864;

    #[cfg(mco_api_ver_lt = "14")]
    pub const MCO_DB_MVCC_TABLE_LEVEL_LOCKING: Type = 67108864;

    #[cfg(mco_api_ver_ge = "14")]
    pub const MCO_DB_MVCC_TABLE_LEVEL_LOCKING: Type = 134217728;

    #[cfg(mco_api_ver_lt = "14")]
    pub const MCO_DB_DISABLE_SMART_ALLOC: Type = 134217728;

    #[cfg(mco_api_ver_ge = "14")]
    pub const MCO_DB_DISABLE_SMART_ALLOC: Type = 268435456;

    #[cfg(all(mco_api_ver_eq = "13"))]
    pub const MCO_DB_DISABLE_DISK_SPACE_RESERVE: Type = 268435456;

    #[cfg(mco_api_ver_ge = "14")]
    pub const MCO_DB_DISABLE_DISK_SPACE_RESERVE: Type = 536870912;

    #[cfg(all(mco_api_ver_eq = "13"))]
    pub const MCO_DB_USE_ALLOCATION_MAP: Type = 536870912;

    #[cfg(mco_api_ver_ge = "14")]
    pub const MCO_DB_USE_ALLOCATION_MAP: Type = 1073741824;
}

pub use MCO_DB_MODE_MASK_::Type as MCO_DB_MODE_MASK;

pub mod MCO_COMPRESSION_MASK_ {
    pub type Type = u32;
    pub const MCO_COMPRESSION_OBJ_HEAD: Type = 1;
    pub const MCO_COMPRESSION_OBJ_NODE: Type = 2;
    pub const MCO_COMPRESSION_BLOB_HEAD: Type = 64;
    pub const MCO_COMPRESSION_BLOB_TAIL: Type = 128;
    pub const MCO_COMPRESSION_FIXEDRECSET: Type = 4096;
    pub const MCO_COMPRESSION_ALL: Type = 4291;
}

pub use MCO_COMPRESSION_MASK_::Type as MCO_COMPRESSION_MASK;

pub mod mco_file_open_flags {
    pub type Type = u32;
    pub const MCO_FILE_OPEN_DEFAULT: Type = 0;
    pub const MCO_FILE_OPEN_READ_ONLY: Type = 1;
    pub const MCO_FILE_OPEN_TRUNCATE: Type = 2;
    pub const MCO_FILE_OPEN_NO_BUFFERING: Type = 4;
    pub const MCO_FILE_OPEN_EXISTING: Type = 8;
    pub const MCO_FILE_OPEN_TEMPORARY: Type = 16;
    pub const MCO_FILE_OPEN_FSYNC_FIX: Type = 32;
    pub const MCO_FILE_OPEN_SUBPARTITION: Type = 64;
    pub const MCO_FILE_OPEN_FSYNC_AIO_BARRIER: Type = 128;
    pub const MCO_FILE_OPEN_COMPRESSED: Type = 256;
    pub const MCO_FILE_OPEN_LOCK: Type = 512;
    pub const MCO_FILE_OPEN_NO_READ_BUFFERING: Type = 1024;
    pub const MCO_FILE_OPEN_NO_WRITE_BUFFERING: Type = 2048;
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mco_db_t_ {
    _unused: [u8; 0],
}

pub type mco_db_h = *mut mco_db_t_;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mco_log_params_t_ {
    pub default_commit_policy: MCO_COMMIT_POLICY,
    pub redo_log_limit: mco_offs_t,
    pub delayed_commit_threshold: mco_offs_t,
    pub max_delayed_transactions: mco_counter32_t,
    pub max_commit_delay: uint4,
}

pub type mco_log_params_t = mco_log_params_t_;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mco_ddl_dictionary_t_ {
    pub _address: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct mco_db_params_t_ {
    pub mark: uint2,
    pub mem_page_size: uint2,
    pub disk_page_size: uint4,
    pub db_max_connections: uint4,
    pub disk_max_database_size: mco_offs_t,
    pub file_extension_quantum: mco_offs_t,
    pub db_log_type: MCO_LOG_TYPE,
    pub connection_context_size: uint2,
    pub hash_load_factor: uint2,
    pub index_optimistic_lock_threshold: uint2,
    pub log_params: mco_log_params_t,
    pub mode_mask: ::std::os::raw::c_int,
    pub min_conn_local_pages: ::std::os::raw::c_int,
    pub max_conn_local_pages: ::std::os::raw::c_int,
    pub allocation_bitmap_caching_priority: ::std::os::raw::c_int,
    pub index_caching_priority: ::std::os::raw::c_int,
    pub object_caching_priority: ::std::os::raw::c_int,
    pub ddl_dict: *mut mco_ddl_dictionary_t_,
    pub ddl_dict_size: mco_size_t,
    pub ddl_dict_flags: ::std::os::raw::c_int,
    pub cipher_key: *mut ::std::os::raw::c_char,
    pub dynamic_hash: mco_bool,
    pub license_key: *mut ::std::os::raw::c_char,
    pub max_classes: ::std::os::raw::c_int,
    pub max_indexes: ::std::os::raw::c_int,
    pub autocompact_threshold: mco_size_t,
    pub trans_sched_policy: MCO_TRANS_SCHED_POLICY,
    pub max_trans_time: uint8,
    pub max_gc_versions: ::std::os::raw::c_int,
    pub max_active_pages: ::std::os::raw::c_int,
    pub page_hash_bundles: ::std::os::raw::c_int,
    pub compression_level: ::std::os::raw::c_int,
    pub compression_mask: ::std::os::raw::c_int,
    pub expected_compression_ratio: ::std::os::raw::c_int,
    pub btree_cursor_read_ahead_size: uint1,
    pub mvcc_bitmap_size: ::std::os::raw::c_int,
    pub additional_heap_size: ::std::os::raw::c_int,
    pub cow_pagemap_size: mco_size_t,
    pub backup_map_size: mco_size_t,
    pub backup_min_pages: ::std::os::raw::c_uint,
    pub backup_max_passes: ::std::os::raw::c_uint,
    pub backup_map_filename: [::std::os::raw::c_char; 256usize],
    pub iot_agent_id: uint8,
    pub iot_level: uint2,
    pub file_backup_delay: uint4,
}

pub type mco_db_params_t = mco_db_params_t_;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mco_device_t___bindgen_ty_1__bindgen_ty_1 {
    pub ptr: *mut ::std::os::raw::c_void,
    pub flags: ::std::os::raw::c_int,
}

pub type mco_device_t_dev_conv = mco_device_t___bindgen_ty_1__bindgen_ty_1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct mco_device_t___bindgen_ty_1__bindgen_ty_2 {
    pub name: [::std::os::raw::c_char; 64usize],
    pub flags: ::std::os::raw::c_uint,
    pub hint: *mut ::std::os::raw::c_void,
}

pub type mco_device_t_dev_named = mco_device_t___bindgen_ty_1__bindgen_ty_2;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct mco_device_t___bindgen_ty_1__bindgen_ty_3 {
    pub flags: ::std::os::raw::c_int,
    pub name: [::std::os::raw::c_char; 256usize],
}

pub type mco_device_t_dev_file = mco_device_t___bindgen_ty_1__bindgen_ty_3;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct mco_device_t___bindgen_ty_1__bindgen_ty_4 {
    pub flags: ::std::os::raw::c_int,
    pub name: [::std::os::raw::c_char; 64usize],
    pub segment_size: mco_offs_t,
}

pub type mco_device_t_dev_multifile = mco_device_t___bindgen_ty_1__bindgen_ty_4;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct mco_device_t___bindgen_ty_1__bindgen_ty_5 {
    pub flags: ::std::os::raw::c_int,
    pub name: [::std::os::raw::c_char; 64usize],
    pub level: ::std::os::raw::c_int,
    pub offset: mco_offs_t,
}

pub type mco_device_t_dev_raid = mco_device_t___bindgen_ty_1__bindgen_ty_5;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mco_device_t___bindgen_ty_1__bindgen_ty_6 {
    pub handle: ::std::os::raw::c_ulong,
}

pub type mco_device_t_dev_idesc = mco_device_t___bindgen_ty_1__bindgen_ty_6;

#[repr(C)]
#[derive(Copy, Clone)]
pub union mco_device_t___bindgen_ty_1 {
    pub conv: mco_device_t___bindgen_ty_1__bindgen_ty_1,
    pub named: mco_device_t___bindgen_ty_1__bindgen_ty_2,
    pub file: mco_device_t___bindgen_ty_1__bindgen_ty_3,
    pub multifile: mco_device_t___bindgen_ty_1__bindgen_ty_4,
    pub raid: mco_device_t___bindgen_ty_1__bindgen_ty_5,
    pub idesc: mco_device_t___bindgen_ty_1__bindgen_ty_6,
}

pub type mco_device_t_dev = mco_device_t___bindgen_ty_1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct mco_device_t_ {
    pub type_: ::std::os::raw::c_uint,
    pub assignment: ::std::os::raw::c_uint,
    pub size: mco_size_t,
    pub dev: mco_device_t___bindgen_ty_1,
}

pub type mco_device_t = mco_device_t_;
pub type mco_device_h = *mut mco_device_t_;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mco_dict_class_info_t_ {
    pub first_index_num: int2,
    pub last_index_num: int2,
    pub list_index_num: int2,
    pub autoid_index_num: int2,
    pub fixedsize: uint4,
    pub autoid_offset: uint2,
    pub history_index_num: int2,
    pub history_length: uint2,
    pub history_offset: uint2,
    pub first_event_num: int2,
    pub last_event_num: int2,
    pub flags: uint2,
    pub struct_ptr: *const mco_dict_struct_t,
    pub init_size: mco_size_t,
    pub auto_oid_offset: uint2,
    pub reserved: uint2,
}

pub type mco_dict_class_info_t = mco_dict_class_info_t_;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mco_dict_index_field_t_ {
    pub field_offset: mco_offs32_t,
    pub vect_field_offset: mco_offs32_sig_t,
    pub indicator_offset: mco_offs32_t,
    pub field_size: uint4,
    pub field_type: uint1,
    pub fld_flags: uint1,
    pub fld_no: uint2,
    pub collation_id: int2,
}

pub type mco_dict_index_field_t = mco_dict_index_field_t_;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mco_dict_index_t_ {
    pub class_code: uint2,
    pub impl_no: uint2,
    pub numof_fields: uint2,
    pub vect_field_offset: mco_offs32_sig_t,
    pub flags: uint4,
    pub fields: *const mco_dict_index_field_t,
    pub numof_keys_estimate: mco_hash_counter_t,
    pub userdef_id: int2,
    pub reserved: int2,
}

pub type mco_dict_index_t = mco_dict_index_t_;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mco_dict_event_t_ {
    pub class_code: uint2,
    pub flags: uint2,
    pub field_offset: mco_offs32_t,
    pub field_size: uint4,
    pub field_type: uint1,
    pub fld_no: uint2,
}

pub type mco_dict_event_t = mco_dict_event_t_;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mco_dict_collation_t_ {
    pub name: *const ::std::os::raw::c_char,
    pub type_: uint1,
    pub pad1: uint1,
    pub pad2: uint2,
}

pub type mco_dict_collation_t = mco_dict_collation_t_;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mco_datalayout_t_ {
    pub c_size: uint2,
    pub c_align: uint2,
    pub c_offset: uint2,
    pub u_size: uint4,
    pub u_align: uint4,
    pub u_offset: uint4,
}

pub type mco_datalayout_t = mco_datalayout_t_;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mco_dict_field_t_ {
    pub name: *const ::std::os::raw::c_char,
    pub layout: mco_datalayout_t,
    pub field_el_type: uint1,
    pub flags: uint1,
    pub array_size: uint2,
    pub struct_num: int4,
    pub field_size: uint4,
    pub refto_class: int2,
    pub init_index: ::std::os::raw::c_uint,
    pub order_no: uint2,
    pub no: uint2,
    pub event_id: uint2,
    pub indicator: uint2,
    pub precision: int1,
    pub seq_order: int1,
    pub seq_elem_size: int1,
}

pub type mco_dict_field_t = mco_dict_field_t_;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mco_dict_struct_t_ {
    pub name: *const ::std::os::raw::c_char,
    pub flags: uint2,
    pub n_fields: uint2,
    pub fields: *const mco_dict_field_t,
    pub c_size: uint2,
    pub c_align: uint2,
    pub u_size: uint4,
    pub u_align: uint4,
}

pub type mco_dict_struct_t = mco_dict_struct_t_;

#[repr(C)]
#[derive(Copy, Clone)]
pub union mco_dictionary_t___bindgen_ty_1 {
    pub ptr: *const int4,
    pub offs: mco_offs_t,
}

pub type mco_dictionary_t_init_i_data = mco_dictionary_t___bindgen_ty_1;

#[repr(C)]
#[derive(Copy, Clone)]
pub union mco_dictionary_t___bindgen_ty_2 {
    pub ptr: *const f64,
    pub offs: mco_offs_t,
}

pub type mco_dictionary_t_init_d_data = mco_dictionary_t___bindgen_ty_2;

#[cfg(mco_api_ver_ge = "13")]
#[repr(C)]
#[derive(Copy, Clone)]
pub union mco_dictionary_t___bindgen_ty_3 {
    pub ptr: *mut ::std::os::raw::c_char,
    pub offs: mco_offs_t,
}

#[cfg(mco_api_ver_ge = "13")]
pub type mco_dictionary_t_init_s_data = mco_dictionary_t___bindgen_ty_3;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct mco_dictionary_t_ {
    pub str_class_names: *const *const ::std::os::raw::c_char,
    pub str_index_names: *const *const ::std::os::raw::c_char,
    pub version_major: uint2,
    pub version_minor: uint2,
    pub version_build: uint2,
    pub magic_number: uint2,
    pub flags: uint4,
    pub oid_is_supported: uint2,
    pub auto_oid_supported: uint2,
    pub n_class_codes: uint2,
    pub n_list_indexes: uint2,
    pub n_autoid_indexes: uint2,
    pub n_history_indexes: uint2,
    pub n_userdef_indexes: uint2,
    pub max_numof_indexes_per_obj: uint2,
    pub n_structs: uint2,
    pub pad: uint2,
    pub num_oid_estimation: mco_counter32_t,
    pub num_HA_estimation: mco_counter32_t,
    pub n_desc_indexes: uint2,
    pub n_desc_events: uint2,
    pub n_desc_colls: uint2,
    pub exact_OID_sizeof: uint2,
    pub layout_OID_size: uint2,
    pub v_class_info: *const mco_dict_class_info_t,
    pub v_desc_indexes: *const mco_dict_index_t,
    pub v_desc_events: *const mco_dict_event_t,
    pub v_all_struct: *const mco_dict_struct_t,
    pub v_desc_colls: *const mco_dict_collation_t,
    pub sizeof_mco_offs_t: [uint1; 2usize],
    pub sizeof_mco_size_t: [uint1; 2usize],
    pub init_i_data: mco_dictionary_t___bindgen_ty_1,
    pub init_i_data_n: uint4,
    pub init_d_data: mco_dictionary_t___bindgen_ty_2,
    pub init_d_data_n: uint4,
    pub class_code_origin: uint4,

    #[cfg(mco_api_ver_ge = "13")]
    pub init_s_data: mco_dictionary_t___bindgen_ty_3,
    #[cfg(mco_api_ver_ge = "13")]
    pub init_s_data_n: uint4,
}

pub type mco_dictionary_t = mco_dictionary_t_;
pub type mco_dictionary_h = *mut mco_dictionary_t_;
pub type mco_dict_h = *mut mco_dictionary_t_;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mco_runtime_info_t_ {
    pub mco_version_major: uint1,
    pub mco_version_minor: uint1,
    pub mco_build_number: uint2,
    pub mco_size_t: uint1,
    pub mco_offs_t: uint1,
    pub uint4_supported: uint1,
    pub float_supported: uint1,
    pub mco_checklevel: uint1,
    pub evaluation_version: uint1,
    pub large_database_supported: uint1,
    pub collation_supported: uint1,
    pub heap31_supported: uint1,
    pub bin_serialization_supported: uint1,
    pub fixedrec_supported: uint1,
    pub statistics_supported: uint1,
    pub events_supported: uint1,
    pub save_load_supported: uint1,
    pub object_initialization_supported: uint1,
    pub direct_index_field_access_supported: uint1,
    pub multiprocess_access_supported: uint1,
    pub object_repack_supported: uint1,
    pub transaction_logging_supported: uint1,
    pub cluster_supported: uint1,
    pub high_availability_supported: uint1,
    pub iot_supported: uint1,
    pub ha_multicast_supported: uint1,
    pub ha_incremental_replication_supported: uint1,
    pub binary_schema_evalution_supported: uint1,
    pub unicode_supported: uint1,
    pub wchar_supported: uint1,
    pub recovery_supported: uint1,
    pub disk_supported: uint1,
    pub direct_pointers_supported: uint1,
    pub persistent_object_supported: uint1,
    pub xml_import_export_supported: uint1,
    pub user_defined_index_supported: uint1,
    pub multifile_supported: uint1,
    pub multifile_descriptor_supported: uint1,
    pub two_phase_commit_supported: uint1,
    pub rtree_supported: uint1,
    pub tree_based_hash: uint1,
    pub tmgr_mvcc_async_cleanup: uint1,
    pub concurent_disk_btree: uint1,
    pub open_cursor_goto_first: uint1,
    pub smart_index_insert: uint1,
    pub btree_leaf_lock: uint1,
    pub null_statistics: uint1,
    pub implicit_runtime_start: uint1,
    pub bufferized_sync_iostream: uint1,
    pub async_replication: uint1,
    pub fast_transaction_list: uint1,
    pub extendable_dirty_page_bitmap: uint1,
    pub mursiw_policy: uint1,
    pub sync_capabilities: uint1,
    pub char_comparison_policy: uint1,
    pub stream_buffer_size: uint4,

    #[cfg(mco_api_ver_lt = "14")]
    pub max_db_instances: uint1,

    #[cfg(mco_api_ver_ge = "14")]
    pub max_db_instances: uint2,

    pub max_db_name_length: uint1,
    pub max_extends: ::std::os::raw::c_int,
    pub tl_page_buffer_size: uint4,
    pub ha_max_replicas: uint2,
    pub ha_transmit_buffer_size: uint4,
    pub ha_syncronization_buffer_size: uint4,
    pub default_redo_log_limit: uint4,
    pub mvcc_critical_sections: uint1,
    pub mvcc_per_index_locks: uint1,
    pub con_disk_page_cache_size: uint2,
    pub small_con_cache_threshold: uint1,
    pub extendable_dirty_page_bitmap_limit: uint4,
    pub max_vista_sessions: uint1,
    pub concurrent_write_transactions: uint1,
    pub encryption_support: uint1,
    pub backup_support: uint1,
    pub mco_revision: *const ::std::os::raw::c_char,
}

pub type mco_runtime_info_t = mco_runtime_info_t_;

extern "C" {
    pub fn mco_runtime_start() -> MCO_RET;

    pub fn mco_runtime_stop() -> MCO_RET;

    pub fn mco_runtime_getoption(option: ::std::os::raw::c_int) -> ::std::os::raw::c_int;

    pub fn mco_runtime_setoption(option: ::std::os::raw::c_int, value: ::std::os::raw::c_int);

    pub fn mco_get_runtime_info(pinf: *mut mco_runtime_info_t);

    pub fn mco_db_params_init(params: *mut mco_db_params_t);

    pub fn mco_db_open_dev(
        db_name: *const ::std::os::raw::c_char,
        dict: mco_dictionary_h,
        devs: *mut mco_device_t,
        n_devs: mco_size_t,
        params: *mut mco_db_params_t,
    ) -> MCO_RET;

    pub fn mco_db_close(db_name: *const ::std::os::raw::c_char) -> MCO_RET;

    pub fn mco_db_kill(db_name: *const ::std::os::raw::c_char) -> MCO_RET;

    pub fn mco_db_connect(db_name: *const ::std::os::raw::c_char, pdb: *mut mco_db_h) -> MCO_RET;

    pub fn mco_db_disconnect(db: mco_db_h) -> MCO_RET;

    pub fn mco_strerror(rc: MCO_RET) -> *const ::std::os::raw::c_char;
}
