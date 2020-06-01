// dict.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

//! *e*X*treme*DB dictionary wrapper types.
//!
//! The dictionary describes the schema of the database. It is either produced
//! statically and embedded in the application code, or created and
//! managed dynamically, e.g. using the SQL DDL language.
//!
//! Static definition of the dictionary is planned for a future release.
//! The types in this module are not used with the dynamic dictionary,
//! and will be documented later.

use std::ptr;

use crate::exdb_sys;

pub type McoDictionary = exdb_sys::mco_dictionary_t;
pub type McoDictClassInfo = exdb_sys::mco_dict_class_info_t;
pub type McoDictStruct = exdb_sys::mco_dict_struct_t;
pub type McoDictField = exdb_sys::mco_dict_field_t;
pub type McoDictIndex = exdb_sys::mco_dict_index_t;
pub type McoDictIndexField = exdb_sys::mco_dict_index_field_t;
pub type McoDictEvent = exdb_sys::mco_dict_event_t;
pub type McoDictCollation = exdb_sys::mco_dict_collation_t;
pub type McoDictLayout = exdb_sys::mco_datalayout_t;
pub type McoDictInitDataI = exdb_sys::mco_dictionary_t___bindgen_ty_1;
pub type McoDictInitDataD = exdb_sys::mco_dictionary_t___bindgen_ty_2;

pub mod mco_const {
    pub const MCO_DICT_F_LARGE_DATABASE: u32 = 0x01;
    pub const MCO_DICT_F_NO_SORT: u32 = 0x02;
    pub const MCO_DICT_F_IOT_SUPPORT: u32 = 0x04;

    pub const MCO_DB_INDF_UNIQUE: u32 = 0x0004;
    pub const MCO_DB_INDF_VSTRUCT_BASED: u32 = 0x0008;
    pub const MCO_DB_INDF_VTYPE_BASED: u32 = 0x0010;
    pub const MCO_DB_INDF_V_BASED: u32 = MCO_DB_INDF_VSTRUCT_BASED | MCO_DB_INDF_VTYPE_BASED;
    pub const MCO_DB_INDF_PERSISTENT: u32 = 0x0020;
    pub const MCO_DB_INDF_VOLUNTARY: u32 = 0x0040;
    pub const MCO_DB_INDF_ASTRUCT_BASED: u32 = 0x0080;
    pub const MCO_DB_INDF_ATYPE_BASED: u32 = 0x0100;
    pub const MCO_DB_INDF_VOLUNTARY_SAVED: u32 = 0x0200;
    pub const MCO_DB_INDF_T_LIST: u32 = 0x0400;
    pub const MCO_DB_INDF_T_REGULAR: u32 = 0x0800;
    pub const MCO_DB_INDF_T_AUTOID: u32 = 0x1000;
    pub const MCO_DB_INDF_T_HISTORY: u32 = 0x2000;
    pub const MCO_DB_INDF_T_ALL: u32 =
        MCO_DB_INDF_T_AUTOID | MCO_DB_INDF_T_HISTORY | MCO_DB_INDF_T_LIST | MCO_DB_INDF_T_REGULAR;
    pub const MCO_DB_INDF_UDF: u32 = 0x4000;
    pub const MCO_DB_INDF_INSERT: u32 = 0x8000;
    pub const MCO_DB_INDF_NULLABLE: u32 = 0x10000;
    pub const MCO_DB_INDF_THICK: u32 = 0x20000;
    pub const MCO_DB_INDF_COMPACT: u32 = 0x40000;
    pub const MCO_DB_INDF_POINT: u32 = 0x80000;
    pub const MCO_DB_INDF_TRIGRAM: u32 = 0x100000;
    pub const MCO_DB_INDF_TLIST: u32 = 0x200000;
    pub const MCO_DB_INDF_OPTIMIZED: u32 = 0x400000;
    pub const MCO_DB_INDF_VBIT_BASED: u32 = 0x1;
    pub const MCO_DB_INDF_ABIT_BASED: u32 = 0x2;
    pub const MCO_DB_INDF_A_BASED: u32 = MCO_DB_INDF_ASTRUCT_BASED | MCO_DB_INDF_ATYPE_BASED;
    pub const MCO_DB_INDF_VA_BASED: u32 = MCO_DB_INDF_A_BASED | MCO_DB_INDF_V_BASED;

    pub const MCO_DB_INDFLD_DESCENDING: u8 = 1;
    pub const MCO_DB_INDFLD_8BT: u8 = 2;
    pub const MCO_DB_INDFLD_CASE_INSENSITIVE: u8 = 4;
    pub const MCO_DB_INDFLD_NULLABLE: u8 = 8;
    pub const MCO_DB_INDFLD_BINARY: u8 = 16;

    pub const MCO_DB_FT_NONE: u8 = 0;
    pub const MCO_DB_FT_UINT1: u8 = 1;
    pub const MCO_DB_FT_UINT2: u8 = 2;
    pub const MCO_DB_FT_UINT4: u8 = 3;
    pub const MCO_DB_FT_INT1: u8 = 4;
    pub const MCO_DB_FT_INT2: u8 = 5;
    pub const MCO_DB_FT_INT4: u8 = 6;
    pub const MCO_DB_FT_CHARS: u8 = 7;
    pub const MCO_DB_FT_STRING: u8 = 8;
    pub const MCO_DB_FT_REF: u8 = 9;
    pub const MCO_DB_FT_FLOAT: u8 = 10;
    pub const MCO_DB_FT_DOUBLE: u8 = 11;
    pub const MCO_DB_FT_UINT8: u8 = 12;
    pub const MCO_DB_FT_INT8: u8 = 13;
    pub const MCO_DB_FT_AUTOID: u8 = 14;
    pub const MCO_DB_FT_OBJVERS: u8 = 15;
    pub const MCO_DB_FT_DATE: u8 = 16;
    pub const MCO_DB_FT_TIME: u8 = 17;
    pub const MCO_DB_FT_AUTOOID: u8 = 18;
    pub const MCO_DB_FT_UNICODE_CHARS: u8 = 19;
    pub const MCO_DB_FT_UNICODE_STRING: u8 = 20;
    pub const MCO_DB_FT_WIDE_CHARS: u8 = 21;
    pub const MCO_DB_FT_WCHAR_STRING: u8 = 22;
    pub const MCO_DB_FT_BOOL: u8 = 23;
    pub const MCO_DB_FT_DATETIME: u8 = 24;
    pub const MCO_DB_FT_BINARY: u8 = 25;
    pub const MCO_DB_FT_VARBINARY: u8 = 26;
    pub const MCO_DB_FT_SEQUENCE_UINT1: u8 = 30;
    pub const MCO_DB_FT_SEQUENCE_UINT2: u8 = 31;
    pub const MCO_DB_FT_SEQUENCE_UINT4: u8 = 32;
    pub const MCO_DB_FT_SEQUENCE_UINT8: u8 = 33;
    pub const MCO_DB_FT_SEQUENCE_INT1: u8 = 34;
    pub const MCO_DB_FT_SEQUENCE_INT2: u8 = 35;
    pub const MCO_DB_FT_SEQUENCE_INT4: u8 = 36;
    pub const MCO_DB_FT_SEQUENCE_INT8: u8 = 37;
    pub const MCO_DB_FT_SEQUENCE_FLOAT: u8 = 38;
    pub const MCO_DB_FT_SEQUENCE_DOUBLE: u8 = 39;
    pub const MCO_DB_FT_SEQUENCE_CHAR: u8 = 40;
    pub const MCO_DB_FT_SEQUENCE_DATETIME: u8 = 41;
    pub const MCO_DB_FT_STRUCT: u8 = 50;
    pub const MCO_DB_FT_BLOB: u8 = 51;

    pub const MCO_DICT_FLDF_VECTOR: u8 = 0x01;
    pub const MCO_DICT_FLDF_ARRAY: u8 = 0x02;
    pub const MCO_DICT_FLDF_OPTIONAL: u8 = 0x04;
    pub const MCO_DICT_FLDF_INDEXED: u8 = 0x08;
    pub const MCO_DICT_FLDF_HIDDEN: u8 = 0x10;
    pub const MCO_DICT_FLDF_NULLABLE: u8 = 0x20;
    pub const MCO_DICT_FLDF_NULL_INDICATOR: u8 = 0x40;
    pub const MCO_DICT_FLDF_NUMERIC: u8 = 0x80;

    pub const MCO_DICT_STF_IS_DYNAMIC: u16 = 1;
    pub const MCO_DICT_STF_HAS_BLOBS: u16 = 2;
    pub const MCO_DICT_STF_INIT: u16 = 4;
    pub const MCO_DICT_STF_IS_DIRECT: u16 = 8;
    pub const MCO_DICT_STF_IS_PACKED: u16 = 16;
    pub const MCO_DICT_STF_HAS_SEQUENCES: u16 = 32;

    pub const MCO_DB_TYPINFO_HAS_LIST: u16 = 0x0001;
    pub const MCO_DB_TYPINFO_HAS_OID: u16 = 0x0002;
    pub const MCO_DB_TYPINFO_HAS_BLOBS: u16 = 0x0004;
    pub const MCO_DB_TYPINFO_COMPACT: u16 = 0x0008;
    pub const MCO_DB_TYPINFO_FIXEDREC: u16 = 0x0010;
    pub const MCO_DB_TYPINFO_PERSISTENT: u16 = 0x0020;
    pub const MCO_DB_TYPINFO_HAS_AUTOID: u16 = 0x0040;
    pub const MCO_DB_TYPINFO_UPTABLE: u16 = 0x0080;
    pub const MCO_DB_TYPINFO_HAS_EVENTS: u16 = 0x0100;
    pub const MCO_DB_TYPINFO_HAS_SEQUENCES: u16 = 0x0200;
    pub const MCO_DB_TYPINFO_LOCAL: u16 = 0x0400;
    pub const MCO_DB_TYPINFO_DISTRIBUTED: u16 = 0x0800;
    pub const MCO_DB_TYPINFO_HIDDEN: u16 = 0x1000;
    pub const MCO_DB_TYPINFO_NONATOMIC: u16 = 0x2000;
    pub const MCO_DB_TYPINFO_DROPPED: u16 = 0x4000;
    pub const MCO_DB_TYPINFO_DOWNTABLE: u16 = 0x8000;

    #[repr(C)]
    pub enum IndexImplName {
        None = 0,       // MCO_INDEX_NONE
        BTreeInMem,     // MCO_INDEX_BTREE_INMEM
        BTreeDisk,      // MCO_INDEX_BTREE_DISK
        HashInMem,      // MCO_INDEX_HASH_INMEM
        KDTreeInMem,    // MCO_INDEX_KDTREE_INMEM
        KDTreeDisk,     // MCO_INDEX_KDTREE_DISK
        RTreeInMem,     // MCO_INDEX_RTREE_INMEM
        RTreeDisk,      // MCO_INDEX_RTREE_DISK
        PatriciaInMem,  // MCO_INDEX_PATRICIA_INMEM
        PatriciaDisk,   // MCO_INDEX_PATRICIA_DISK
        FixedRecList,   // MCO_INDEX_FIXEDREC_LIST
        Union,          // MCO_INDEX_UNION
        Intersect,      // MCO_INDEX_INTERSECT
        InclusiveBTree, // MCO_INDEX_INCLUSIVE_BTREE
        TrigramInMem,   // MCO_INDEX_TRIGRAM_INMEM
        TrigramDisk,    // MCO_INDEX_TRIGRAM_DISK
        NameMax,        // MCO_INDEX_IMPLEMENTATION_NAME_MAX
    }
}

#[repr(C)]
pub struct Dictionary {
    pub nested: McoDictionary,
}

unsafe impl Sync for Dictionary {}

#[repr(C)]
pub struct DictClassInfo {
    pub nested: McoDictClassInfo,
}

impl DictClassInfo {
    pub fn zero() -> Self {
        DictClassInfo {
            nested: McoDictClassInfo {
                first_index_num: 0,
                last_index_num: 0,
                list_index_num: 0,
                autoid_index_num: 0,
                fixedsize: 0,
                autoid_offset: 0,
                history_index_num: 0,
                history_length: 0,
                history_offset: 0,
                first_event_num: 0,
                last_event_num: 0,
                flags: 0,
                struct_ptr: ptr::null(),
                init_size: 0,
                auto_oid_offset: 0,
                reserved: 0,
            },
        }
    }
}

unsafe impl Sync for DictClassInfo {}

#[repr(C)]
pub struct DictStruct {
    pub nested: McoDictStruct,
}

impl DictStruct {
    pub fn new() -> Self {
        DictStruct {
            nested: McoDictStruct {
                name: ptr::null(),
                flags: 0,
                n_fields: 0,
                fields: ptr::null(),
                c_size: 0,
                c_align: 0,
                u_size: 0,
                u_align: 0,
            },
        }
    }
}

unsafe impl Sync for DictStruct {}

#[repr(C)]
pub struct DictField {
    pub nested: McoDictField,
}

impl DictField {
    pub fn new() -> Self {
        DictField {
            nested: McoDictField {
                name: ptr::null(),
                layout: DictLayout::new().nested,
                field_el_type: 0,
                flags: 0,
                array_size: 0,
                struct_num: -1,
                field_size: 0,
                refto_class: -1,
                init_index: 0,
                order_no: 0,
                no: 0,
                event_id: 0,
                indicator: 0,
                precision: 0, // -1 means "no precision", but mcocomp resets it to 0 in strdict.cpp:288
                seq_order: 0,
                seq_elem_size: 0,
            },
        }
    }
}

unsafe impl Sync for DictField {}

#[repr(C)]
pub struct DictIndex {
    pub nested: McoDictIndex,
}

impl DictIndex {
    pub fn new() -> Self {
        DictIndex {
            nested: McoDictIndex {
                class_code: 0,
                impl_no: 0,
                numof_fields: 0,
                vect_field_offset: -1,
                flags: 0,
                fields: ptr::null(),
                numof_keys_estimate: 0,
                userdef_id: 0,
                reserved: 0,
            },
        }
    }
}

unsafe impl Sync for DictIndex {}

#[repr(C)]
pub struct DictIndexField {
    pub nested: McoDictIndexField,
}

impl DictIndexField {
    pub fn new() -> Self {
        DictIndexField {
            nested: McoDictIndexField {
                field_offset: 0,
                vect_field_offset: -1,
                indicator_offset: u32::max_value(),
                field_size: 0,
                field_type: 0,
                fld_flags: 0,
                fld_no: 0,
                collation_id: -1,
            },
        }
    }

    pub fn zero() -> Self {
        DictIndexField {
            nested: McoDictIndexField {
                field_offset: 0,
                vect_field_offset: 0,
                indicator_offset: 0,
                field_size: 0,
                field_type: 0,
                fld_flags: 0,
                fld_no: 0,
                collation_id: 0,
            },
        }
    }
}

unsafe impl Sync for DictIndexField {}

#[repr(C)]
pub struct DictEvent {
    pub nested: McoDictEvent,
}

unsafe impl Sync for DictEvent {}

#[repr(C)]
pub struct DictCollation {
    pub nested: McoDictCollation,
}

unsafe impl Sync for DictCollation {}

#[repr(C)]
pub struct DictLayout {
    pub nested: McoDictLayout,
}

impl DictLayout {
    fn new() -> Self {
        DictLayout {
            nested: McoDictLayout {
                c_size: 0,
                c_align: 0,
                c_offset: 0,
                u_size: 0,
                u_align: 0,
                u_offset: 0,
            },
        }
    }
}

unsafe impl Sync for DictLayout {}

#[repr(C)]
pub struct DictInitDataI {
    pub nested: McoDictInitDataI,
}

unsafe impl Sync for DictInitDataI {}

#[repr(C)]
pub struct DictInitDataD {
    pub nested: McoDictInitDataD,
}

unsafe impl Sync for DictInitDataD {}
