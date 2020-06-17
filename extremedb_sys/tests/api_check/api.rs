use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::{Error as IoError, Read};
use std::mem;

use proc_macro2::{LexError, Span};
use syn;

#[derive(Debug)]
pub enum ApiError {
    InvalidSyntax(String),
    UnexpectedItem(String),
    InvalidEnumMod(String),
    InvalidStruct(String),
    InvalidFunction(String),
    Redefined(String),
}

#[derive(Debug)]
pub enum MismatchError {
    TypedefMismatch(String),
    EnumMissing(String),
    EnumTypeMismatch(String),
    EnumVariantMissing(String, String),
    EnumVariantUnknown(String, String),
    EnumVariantMismatch(String, String),
    StructMissing(String),
    StructFieldCountMismatch(String),
    StructFieldNameMismatch(String, String, String),
    StructFieldTypeMismatch(String, String, String),
    UnionMissing(String),
    UnionFieldCountMismatch(String),
    UnionFieldNameMismatch(String, String, String),
    UnionFieldTypeMismatch(String, String, String),
    FunctionMissing(String),
    FunctionAbiMismatch(String),
    FunctionArgsMismatch(String),
    FunctionRetMismatch(String),
}

#[derive(Debug)]
pub enum Error {
    InOut(IoError),
    Api(ApiError),
    Mismatch(MismatchError),
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::InOut(e)
    }
}

impl From<syn::Error> for Error {
    fn from(e: syn::Error) -> Self {
        Error::Api(ApiError::InvalidSyntax(e.to_string()))
    }
}

impl From<LexError> for Error {
    fn from(e: LexError) -> Self {
        Error::Api(ApiError::InvalidSyntax(format!("{:?}", e)))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

// An enum parsed from a constified enum module.
#[derive(Debug)]
struct ConstEnum {
    ty: syn::Type,
    variants: HashMap<syn::Ident, i64>,
}

impl ConstEnum {
    fn parse_const_value_int(ident: &syn::Ident, e: &syn::Expr) -> Result<i64> {
        use syn::{Expr, ExprLit, Lit};

        if let Expr::Lit(ExprLit {
            lit: Lit::Int(i), ..
        }) = e
        {
            Ok(i.base10_parse::<i64>()?)
        } else {
            Err(Error::Api(ApiError::InvalidEnumMod(ident.to_string())))
        }
    }

    fn parse_const_value(ident: &syn::Ident, e: &syn::Expr) -> Result<i64> {
        use syn::{Expr, ExprUnary, UnOp};

        if let Expr::Unary(ExprUnary {
            op: UnOp::Neg(_),
            expr,
            ..
        }) = e
        {
            ConstEnum::parse_const_value_int(ident, expr)
        } else {
            ConstEnum::parse_const_value_int(ident, e)
        }
    }
}

impl TryFrom<syn::ItemMod> for ConstEnum {
    type Error = Error;

    // Tries to parse a constified enum module (see bindgen).
    fn try_from(value: syn::ItemMod) -> Result<Self> {
        let mut ty = None;
        let mut variants = Vec::new();

        let (_, items) = value.content.ok_or(Error::Api(ApiError::InvalidEnumMod(
            value.ident.to_string(),
        )))?;

        for item in items {
            match item {
                syn::Item::Type(t) => {
                    if ty.is_none() && t.ident == "Type" {
                        ty = Some(*t.ty);
                    } else {
                        return Err(Error::Api(ApiError::InvalidEnumMod(t.ident.to_string())));
                    }
                }
                syn::Item::Const(c) => {
                    let val = ConstEnum::parse_const_value(&c.ident, &c.expr)?;
                    variants.push((c.ident, val));
                }
                _ => {
                    return Err(Error::Api(ApiError::InvalidEnumMod(
                        value.ident.to_string(),
                    )))
                }
            }
        }

        if ty.is_none() || variants.is_empty() {
            Err(Error::Api(ApiError::InvalidEnumMod(
                value.ident.to_string(),
            )))
        } else {
            Ok(ConstEnum {
                ty: ty.unwrap(),
                variants: variants.into_iter().collect(),
            })
        }
    }
}

#[derive(Debug)]
struct Struct {
    fields: Vec<(syn::Ident, syn::Type)>,
}

impl Struct {
    fn new() -> Self {
        Struct { fields: Vec::new() }
    }
}

impl TryFrom<syn::ItemStruct> for Struct {
    type Error = Error;

    fn try_from(value: syn::ItemStruct) -> Result<Self> {
        let ident = value.ident.clone();
        if let syn::Fields::Named(fields) = value.fields {
            let mut s = Struct::new();

            for field in fields.named {
                s.fields.push((field.ident.unwrap(), field.ty));
            }

            Ok(s)
        } else {
            Err(Error::Api(ApiError::InvalidStruct(ident.to_string())))
        }
    }
}

#[derive(Debug)]
struct Union {
    fields: Vec<(syn::Ident, syn::Type)>,
}

impl Union {
    fn new() -> Self {
        Union { fields: Vec::new() }
    }
}

impl TryFrom<syn::ItemUnion> for Union {
    type Error = Error;

    fn try_from(value: syn::ItemUnion) -> Result<Self> {
        let mut u = Union::new();

        for field in value.fields.named {
            u.fields.push((field.ident.unwrap(), field.ty));
        }

        Ok(u)
    }
}

#[derive(Debug)]
struct Function {
    abi: Option<String>,
    args: Vec<(syn::Ident, syn::Type)>,
    vararg: bool,
    ret: Option<syn::Type>,
}

impl Function {
    fn new() -> Self {
        Function {
            abi: None,
            args: Vec::new(),
            vararg: false,
            ret: None,
        }
    }

    fn add_fn_arg(&mut self, ident: &syn::Ident, arg: syn::FnArg) -> Result<()> {
        if let syn::FnArg::Typed(syn::PatType { pat, ty, .. }) = arg {
            if let syn::Pat::Ident(id) = *pat {
                self.args.push((id.ident, *ty));
                Ok(())
            } else {
                Err(Error::Api(ApiError::InvalidFunction(ident.to_string())))
            }
        } else {
            Err(Error::Api(ApiError::InvalidFunction(ident.to_string())))
        }
    }
}

impl TryFrom<syn::Signature> for Function {
    type Error = Error;

    fn try_from(value: syn::Signature) -> Result<Self> {
        let ident = value.ident.clone();
        let mut f = Function::new();

        if let Some(syn::Abi {
            name: Some(lit_abi),
            ..
        }) = value.abi
        {
            f.abi = Some(lit_abi.value());
        }

        if let syn::Generics {
            lt_token: Some(_), ..
        } = value.generics
        {
            return Err(Error::Api(ApiError::InvalidFunction(ident.to_string())));
        }

        for arg in value.inputs {
            f.add_fn_arg(&ident, arg)?;
        }

        if let syn::ReturnType::Type(_, ty) = value.output {
            f.ret = Some(*ty);
        }

        Ok(f)
    }
}

// Contains API information.
#[derive(Debug)]
pub struct Api {
    typedefs: HashMap<syn::Ident, syn::Type>,
    enums: HashMap<syn::Ident, ConstEnum>,
    structs: HashMap<syn::Ident, Struct>,
    unions: HashMap<syn::Ident, Union>,
    functions: HashMap<syn::Ident, Function>,
}

impl Api {
    pub fn new() -> Self {
        Api {
            typedefs: HashMap::new(),
            enums: HashMap::new(),
            structs: HashMap::new(),
            unions: HashMap::new(),
            functions: HashMap::new(),
        }
    }
}

// Builds an Api structure from input file(s).
pub struct Builder {
    api: Api,
    strict: bool,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            api: Api::new(),
            strict: true,
        }
    }

    // Disables strict checks.
    pub fn no_strict(&mut self) -> &mut Self {
        self.strict = false;
        self
    }

    // Reads API from a reader.
    pub fn read<R: Read>(&mut self, mut r: R) -> Result<()> {
        let mut s = String::new();
        r.read_to_string(&mut s)?;
        self.add_file(syn::parse_file(&s)?)
    }

    // Reads API from a string.
    pub fn read_file_str(&mut self, s: &str) -> Result<()> {
        self.add_file(syn::parse_file(s)?)
    }

    // Produces the Api struct.
    pub fn finish(self) -> Result<Api> {
        Ok(self.api)
    }

    fn add_file(&mut self, f: syn::File) -> Result<()> {
        for item in f.items {
            self.add_item(item)?;
        }

        Ok(())
    }

    fn add_item(&mut self, i: syn::Item) -> Result<()> {
        if Builder::test_only(&i) {
            return Ok(());
        }

        match i {
            syn::Item::Mod(syn::ItemMod { content: None, .. }) => Ok(()),
            syn::Item::Mod(i) => self.add_mod(i),
            syn::Item::Use(_) => Ok(()),
            syn::Item::Type(i) => self.add_type(i),
            syn::Item::Struct(i) => self.add_struct(i),
            syn::Item::Union(i) => self.add_union(i),
            syn::Item::ForeignMod(i) => self.add_foreign_mod(i),
            _ => self.strict_err(Error::Api(ApiError::UnexpectedItem(format!("{:?}", i)))),
        }
    }

    fn add_foreign_mod(&mut self, i: syn::ItemForeignMod) -> Result<()> {
        let abi = i.abi.name.map(|lit| lit.value());
        for item in i.items {
            self.add_foreign_item(item, abi.as_ref())?;
        }

        Ok(())
    }

    fn add_foreign_item(&mut self, i: syn::ForeignItem, abi: Option<&String>) -> Result<()> {
        match i {
            syn::ForeignItem::Fn(i) => self.add_foreign_fn(i, abi),
            _ => self.strict_err(Error::Api(ApiError::UnexpectedItem(format!("{:?}", i)))),
        }
    }

    fn add_mod(&mut self, i: syn::ItemMod) -> Result<()> {
        // Only "constified enum modules" (see bindgen) are supported, which are
        // expected to contain a Type definition, and a number of integer
        // variants.
        // Everything else is reported as an error in strict mode.

        let ident = i.ident.clone();

        match ConstEnum::try_from(i) {
            Ok(en) => {
                if let None = self.api.enums.insert(ident.clone(), en) {
                    Ok(())
                } else {
                    Err(Error::Api(ApiError::Redefined(ident.to_string())))
                }
            }
            Err(e) => self.strict_err(e),
        }
    }

    fn add_type(&mut self, i: syn::ItemType) -> Result<()> {
        let ident = i.ident.to_string();
        if let None = self.api.typedefs.insert(i.ident, *i.ty) {
            Ok(())
        } else {
            Err(Error::Api(ApiError::Redefined(ident)))
        }
    }

    fn add_struct(&mut self, i: syn::ItemStruct) -> Result<()> {
        let ident = i.ident.clone();
        match Struct::try_from(i) {
            Ok(s) => {
                if let None = self.api.structs.insert(ident.clone(), s) {
                    Ok(())
                } else {
                    Err(Error::Api(ApiError::Redefined(ident.to_string())))
                }
            }
            Err(e) => self.strict_err(e),
        }
    }

    fn add_union(&mut self, i: syn::ItemUnion) -> Result<()> {
        let ident = i.ident.clone();
        match Union::try_from(i) {
            Ok(u) => {
                if let None = self.api.unions.insert(ident.clone(), u) {
                    Ok(())
                } else {
                    Err(Error::Api(ApiError::Redefined(ident.to_string())))
                }
            }
            Err(e) => self.strict_err(e),
        }
    }

    fn add_foreign_fn(&mut self, i: syn::ForeignItemFn, abi: Option<&String>) -> Result<()> {
        let ident = i.sig.ident.clone();
        let f = Function::try_from(i.sig);
        match f {
            Ok(mut f) => {
                if let Some(abi_str) = abi {
                    f.abi = Some(abi_str.clone());
                }

                if let None = self.api.functions.insert(ident.clone(), f) {
                    Ok(())
                } else {
                    Err(Error::Api(ApiError::Redefined(ident.to_string())))
                }
            }
            Err(e) => self.strict_err(e),
        }
    }

    fn strict_err(&self, e: Error) -> Result<()> {
        if self.strict {
            Err(e)
        } else {
            Ok(())
        }
    }

    // Returns true if the item has a "test" annotation.
    fn test_only(item: &syn::Item) -> bool {
        let attrs = match item {
            syn::Item::Fn(item) => Some(&item.attrs),
            _ => None,
        };
        attrs.map_or(false, |attrs| {
            attrs.iter().any(|a| Builder::is_test_attr(a))
        })
    }

    // Returns true if the attribute is a "test" attribute.
    fn is_test_attr(attr: &syn::Attribute) -> bool {
        let meta = attr.parse_meta();
        match meta {
            Ok(meta) => match meta {
                syn::Meta::Path(path) => path.is_ident(&syn::Ident::new("test", Span::call_site())),
                _ => false,
            },
            _ => false,
        }
    }
}

// Matches an "inner" API against an "outer" one.
//
// The "inner" API is supposed to be the static FFI bindings, and the "outer"
// API is expected to be generated by bindgen. As such, the "outer" API will
// contain all of the functions and types defined in the C headers, and
// the "inner" API will normally be a subset of the "outer". The "inner" API
// must not contain any items that are not present in the "outer" API.
// The items declared by both APIs must have compatible definitions.
pub struct Matcher {}

impl Matcher {
    pub fn new() -> Self {
        Matcher {}
    }

    pub fn match_apis(&self, inner: &Api, outer: &Api) -> Result<()> {
        self.match_typedefs(inner, outer)?;
        self.match_enums(inner, outer)?;
        self.match_structs(inner, outer)?;
        self.match_unions(inner, outer)?;
        self.match_fns(inner, outer)?;

        Ok(())
    }

    fn match_typedefs(&self, inner: &Api, outer: &Api) -> Result<()> {
        let mut ty_match = TypeMatcher::new();
        ty_match.use_typedefs(&inner.typedefs, &outer.typedefs);
        ty_match.allow_compatible_ints();

        for (ident, inner_type) in &inner.typedefs {
            let outer_type = outer.typedefs.get(ident);
            // It is ok to have typedefs missing in the outer API, as long as
            // the types are used consistently.
            if outer_type.is_some() {
                let outer_type = outer_type.unwrap();
                if !ty_match.match_types(inner_type, outer_type) {
                    return Err(Error::Mismatch(MismatchError::TypedefMismatch(
                        ident.to_string(),
                    )));
                }
            }
        }

        Ok(())
    }

    fn match_enums(&self, inner: &Api, outer: &Api) -> Result<()> {
        for (ident, inner_enum) in &inner.enums {
            let outer_enum =
                outer
                    .enums
                    .get(ident)
                    .ok_or(Error::Mismatch(MismatchError::EnumMissing(
                        ident.to_string(),
                    )))?;

            self.match_enum(ident, inner_enum, outer_enum)?;
        }

        Ok(())
    }

    fn match_enum(&self, ident: &syn::Ident, inner: &ConstEnum, outer: &ConstEnum) -> Result<()> {
        let mut ty_match = TypeMatcher::new();
        // Clang produces enum types with different signedness on different
        // platforms.
        ty_match.ignore_int_sign();

        if !ty_match.match_types(&inner.ty, &outer.ty) {
            return Err(Error::Mismatch(MismatchError::EnumTypeMismatch(
                ident.to_string(),
            )));
        }

        for (item_ident, inner_item) in &inner.variants {
            let outer_item = outer.variants.get(item_ident).ok_or(Error::Mismatch(
                MismatchError::EnumVariantUnknown(ident.to_string(), item_ident.to_string()),
            ))?;

            if *outer_item != *inner_item {
                return Err(Error::Mismatch(MismatchError::EnumVariantMismatch(
                    ident.to_string(),
                    item_ident.to_string(),
                )));
            }
        }

        for (item_ident, _) in &outer.variants {
            inner.variants.get(item_ident).ok_or(Error::Mismatch(
                MismatchError::EnumVariantMissing(ident.to_string(), item_ident.to_string()),
            ))?;
        }

        Ok(())
    }

    fn match_structs(&self, inner: &Api, outer: &Api) -> Result<()> {
        for (ident, inner_struct) in &inner.structs {
            let outer_struct =
                outer
                    .structs
                    .get(ident)
                    .ok_or(Error::Mismatch(MismatchError::StructMissing(
                        ident.to_string(),
                    )))?;

            self.match_struct(ident, inner_struct, outer_struct)?;
        }

        Ok(())
    }

    fn match_struct(&self, ident: &syn::Ident, inner: &Struct, outer: &Struct) -> Result<()> {
        if inner.fields.len() != outer.fields.len() {
            Err(Error::Mismatch(MismatchError::StructFieldCountMismatch(
                ident.to_string(),
            )))
        } else if inner.fields != outer.fields {
            // Check fields one by one
            self.match_struct_fields(ident, inner, outer)
        } else {
            Ok(())
        }
    }

    fn match_struct_fields(
        &self,
        ident: &syn::Ident,
        inner: &Struct,
        outer: &Struct,
    ) -> Result<()> {
        assert_eq!(inner.fields.len(), outer.fields.len());

        let mut ty_matcher = TypeMatcher::new();
        ty_matcher.ignore_ptr_constness();

        for ((inner_id, inner_ty), (outer_id, outer_ty)) in
            inner.fields.iter().zip(outer.fields.iter())
        {
            if inner_id != outer_id {
                return Err(Error::Mismatch(MismatchError::StructFieldNameMismatch(
                    ident.to_string(),
                    inner_id.to_string(),
                    outer_id.to_string(),
                )));
            }

            if !ty_matcher.match_types(inner_ty, outer_ty) {
                return Err(Error::Mismatch(MismatchError::StructFieldTypeMismatch(
                    ident.to_string(),
                    inner_id.to_string(),
                    outer_id.to_string(),
                )));
            }
        }

        Ok(())
    }

    fn match_unions(&self, inner: &Api, outer: &Api) -> Result<()> {
        for (ident, inner_union) in &inner.unions {
            let outer_union =
                outer
                    .unions
                    .get(ident)
                    .ok_or(Error::Mismatch(MismatchError::UnionMissing(
                        ident.to_string(),
                    )))?;

            self.match_union(ident, inner_union, outer_union)?;
        }

        Ok(())
    }

    fn match_union(&self, ident: &syn::Ident, inner: &Union, outer: &Union) -> Result<()> {
        if inner.fields.len() != outer.fields.len() {
            Err(Error::Mismatch(MismatchError::UnionFieldCountMismatch(
                ident.to_string(),
            )))
        } else if inner.fields != outer.fields {
            // Check fields one by one
            self.match_union_fields(ident, inner, outer)
        } else {
            Ok(())
        }
    }

    fn match_union_fields(&self, ident: &syn::Ident, inner: &Union, outer: &Union) -> Result<()> {
        assert_eq!(inner.fields.len(), outer.fields.len());

        let mut ty_matcher = TypeMatcher::new();
        ty_matcher.ignore_ptr_constness();

        for ((inner_id, inner_ty), (outer_id, outer_ty)) in
            inner.fields.iter().zip(outer.fields.iter())
        {
            if inner_id != outer_id {
                return Err(Error::Mismatch(MismatchError::UnionFieldNameMismatch(
                    ident.to_string(),
                    inner_id.to_string(),
                    outer_id.to_string(),
                )));
            }

            if !ty_matcher.match_types(inner_ty, outer_ty) {
                return Err(Error::Mismatch(MismatchError::UnionFieldTypeMismatch(
                    ident.to_string(),
                    inner_id.to_string(),
                    outer_id.to_string(),
                )));
            }
        }

        Ok(())
    }

    fn match_fns(&self, inner: &Api, outer: &Api) -> Result<()> {
        for (ident, inner_fn) in &inner.functions {
            let outer_fn = outer.functions.get(ident).ok_or(Error::Mismatch(
                MismatchError::FunctionMissing(ident.to_string()),
            ))?;

            self.match_fn(ident, inner_fn, outer_fn)?;
        }

        Ok(())
    }

    fn match_fn(&self, ident: &syn::Ident, inner: &Function, outer: &Function) -> Result<()> {
        if inner.abi != outer.abi {
            Err(Error::Mismatch(MismatchError::FunctionAbiMismatch(
                ident.to_string(),
            )))
        } else if inner.args.len() != outer.args.len() {
            Err(Error::Mismatch(MismatchError::FunctionArgsMismatch(
                ident.to_string(),
            )))
        } else if inner.args != outer.args {
            Err(Error::Mismatch(MismatchError::FunctionArgsMismatch(
                ident.to_string(),
            )))
        } else if inner.vararg != outer.vararg {
            Err(Error::Mismatch(MismatchError::FunctionArgsMismatch(
                ident.to_string(),
            )))
        } else if inner.ret != outer.ret {
            Err(Error::Mismatch(MismatchError::FunctionRetMismatch(
                ident.to_string(),
            )))
        } else {
            Ok(())
        }
    }
}

// A reference or a pointer.
struct RefType {
    mutable: bool,
    is_ptr: bool,
    target: syn::Type,
}

impl RefType {
    fn from_syn_type(ty: &syn::Type) -> Option<Self> {
        match ty {
            syn::Type::Reference(ty) => Some(RefType {
                mutable: ty.mutability.is_some(),
                is_ptr: false,
                target: *ty.elem.clone(),
            }),
            syn::Type::Ptr(ty) => Some(RefType {
                mutable: ty.mutability.is_some(),
                is_ptr: true,
                target: *ty.elem.clone(),
            }),
            _ => None,
        }
    }
}

// An integer type.
struct IntType {
    size: usize,                 // Size of a value of this type.
    unsigned: Option<syn::Type>, // Unsigned variant of this type, if any.
}

impl IntType {
    fn new(size: usize, unsigned: Option<syn::Type>) -> Self {
        IntType { size, unsigned }
    }
}

// Returns syn::Type for the given Rust type.
macro_rules! syn_type {
    ($ty:ty) => {
        syn::parse_str::<syn::Type>(stringify!($ty)).unwrap()
    };
}

// Returns syn::Ident for the given identifier.
macro_rules! syn_ident {
    ($id:ident) => {
        syn::parse_str::<syn::Ident>(stringify!($id)).unwrap()
    };
}

// Returns a tuple of (syn::Type, IntType) for the given signed integer type.
macro_rules! int_info {
    ($ty:ty, $unsigned:ty) => {
        (
            syn_type!($ty),
            IntType::new(mem::size_of::<$ty>(), Some(syn_type!($unsigned))),
        )
    };
}

// Returns a tuple of (syn::Type, IntType) for the given unsigned integer type.
macro_rules! uint_info {
    ($ty:ty) => {
        (syn_type!($ty), IntType::new(mem::size_of::<$ty>(), None))
    };
}

struct TypeMatcher {
    ign_int_sign: bool,      // Ignore sign when comparing integer types.
    ign_ptr_constness: bool, // Ignore ptr/ref constness introduced by the inner API.
    allow_compat_ints: bool, // Accept compatible integer types (e.g. c_uint and u32).

    // Inner and outer API type definitions for type resolution.
    inner_defs: HashMap<syn::Type, syn::Type>,
    outer_defs: HashMap<syn::Type, syn::Type>,

    // Integer types' metadata for compatibility checks.
    int_info_map: HashMap<syn::Type, IntType>,
}

impl TypeMatcher {
    fn new() -> Self {
        TypeMatcher {
            ign_int_sign: false,
            ign_ptr_constness: false,
            allow_compat_ints: false,

            inner_defs: HashMap::new(),
            outer_defs: HashMap::new(),

            int_info_map: Self::build_int_info(),
        }
    }

    fn ignore_int_sign(&mut self) -> &mut Self {
        self.ign_int_sign = true;
        self
    }

    fn ignore_ptr_constness(&mut self) -> &mut Self {
        self.ign_ptr_constness = true;
        self
    }

    fn use_typedefs(
        &mut self,
        inner: &HashMap<syn::Ident, syn::Type>,
        outer: &HashMap<syn::Ident, syn::Type>,
    ) -> &mut Self {
        for (id, ty) in inner {
            let id_ty: syn::Type = syn::parse_str(&id.to_string()).unwrap();
            self.inner_defs.insert(id_ty, (*ty).clone());
        }

        for (id, ty) in outer {
            let id_ty: syn::Type = syn::parse_str(&id.to_string()).unwrap();
            self.outer_defs.insert(id_ty, (*ty).clone());
        }

        self
    }

    fn allow_compatible_ints(&mut self) -> &mut Self {
        self.allow_compat_ints = true;
        self
    }

    fn match_types(&self, inner: &syn::Type, outer: &syn::Type) -> bool {
        // Resolve typedefs and unsign if signs are to be ignored.
        let inner = self.unsign_if_allowed(self.resolve_inner_typedef(inner));
        let outer = self.unsign_if_allowed(self.resolve_outer_typedef(outer));

        if inner == outer {
            // The simplest case: types are equal.
            true
        } else if let (Some(inner_i), Some(outer_i)) =
            (self.int_info_map.get(&inner), self.int_info_map.get(&outer))
        {
            // Check integer types for compatibility, if allowed.
            self.allow_compat_ints
                && (inner_i.size == outer_i.size)
                && (inner_i.unsigned.is_some() == outer_i.unsigned.is_some())
        } else if let (Some(inner_p), Some(outer_p)) = (
            RefType::from_syn_type(&inner),
            RefType::from_syn_type(&outer),
        ) {
            // Reference types are compatible if they refer to the same type,
            // and mutability is not introduced by the inner API type.
            if (inner_p.is_ptr == outer_p.is_ptr)
                && ((inner_p.mutable == outer_p.mutable) || !inner_p.mutable)
            {
                self.ign_ptr_constness && (inner_p.target == outer_p.target)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn unsign_if_allowed(&self, t: syn::Type) -> syn::Type {
        if self.ign_int_sign {
            self.unsign(&t)
        } else {
            t
        }
    }

    fn unsign(&self, t: &syn::Type) -> syn::Type {
        self.int_info_map
            .get(t)
            .and_then(|u| u.unsigned.clone())
            .unwrap_or((*t).clone())
    }

    fn build_int_info() -> HashMap<syn::Type, IntType> {
        vec![
            uint_info!(u8),
            uint_info!(u16),
            uint_info!(u32),
            uint_info!(u64),
            int_info!(i8, u8),
            int_info!(i16, u16),
            int_info!(i32, u32),
            int_info!(i64, u64),
            uint_info!(::std::os::raw::c_uchar),
            uint_info!(::std::os::raw::c_ushort),
            uint_info!(::std::os::raw::c_uint),
            uint_info!(::std::os::raw::c_ulong),
            uint_info!(::std::os::raw::c_ulonglong),
            int_info!(::std::os::raw::c_schar, ::std::os::raw::c_uchar),
            int_info!(::std::os::raw::c_short, ::std::os::raw::c_ushort),
            int_info!(::std::os::raw::c_int, ::std::os::raw::c_uint),
            int_info!(::std::os::raw::c_long, ::std::os::raw::c_ulong),
            int_info!(::std::os::raw::c_longlong, ::std::os::raw::c_ulonglong),
            uint_info!(usize),
        ]
        .into_iter()
        .collect()
    }

    fn resolve_inner_typedef(&self, ty: &syn::Type) -> syn::Type {
        Self::resolve_typedef(ty, &self.inner_defs)
    }

    fn resolve_outer_typedef(&self, ty: &syn::Type) -> syn::Type {
        Self::resolve_typedef(ty, &self.outer_defs)
    }

    fn resolve_typedef(ty: &syn::Type, ty_map: &HashMap<syn::Type, syn::Type>) -> syn::Type {
        match ty_map.get(ty) {
            Some(ty_def) => Self::resolve_typedef(ty_def, ty_map),
            None => (*ty).clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn parse_const_enum() -> Result<()> {
        let tokens = quote! {
            pub mod ConstifiedEnum {
                pub type Type = u32;
                pub const VARIANT_1: Type = 1;
                pub const VARIANT_2: Type = 2;
                pub const VARIANT_3: Type = 3;
            }
        };

        let item: syn::ItemMod = syn::parse2(tokens).unwrap();
        let e = ConstEnum::try_from(item)?;

        assert_eq!(&e.ty, &syn_type!(u32));
        assert_eq!(e.variants.len(), 3);
        assert_eq!(e.variants.get(&syn_ident!(VARIANT_1)).unwrap(), &1);
        assert_eq!(e.variants.get(&syn_ident!(VARIANT_2)).unwrap(), &2);
        assert_eq!(e.variants.get(&syn_ident!(VARIANT_3)).unwrap(), &3);

        Ok(())
    }

    #[test]
    fn parse_const_enum_no_type_fails() {
        let tokens = quote! {
            pub mod ConstifiedEnum {
                pub const VARIANT_1: Type = 1;
                pub const VARIANT_2: Type = 2;
                pub const VARIANT_3: Type = 3;
            }
        };

        let item: syn::ItemMod = syn::parse2(tokens).unwrap();
        let e = ConstEnum::try_from(item);

        if let Err(Error::Api(ApiError::InvalidEnumMod(ident))) = e {
            assert_eq!(ident, "ConstifiedEnum".to_string())
        } else {
            panic!("Unexpected result {:?}", e)
        }
    }

    #[test]
    fn parse_const_enum_no_items_fails() {
        let tokens = quote! {
            pub mod ConstifiedEnum {
                pub type Type = u32;
            }
        };

        let item: syn::ItemMod = syn::parse2(tokens).unwrap();
        let e = ConstEnum::try_from(item);

        if let Err(Error::Api(ApiError::InvalidEnumMod(ident))) = e {
            assert_eq!(ident, "ConstifiedEnum".to_string())
        } else {
            panic!("Unexpected result {:?}", e)
        }
    }

    #[test]
    fn parse_const_enum_unexpected_item_fails() {
        let tokens = quote! {
            pub mod ConstifiedEnum {
                pub type Type = u32;
                pub const VARIANT_1: Type = 1;
                pub const VARIANT_2: Type = 2;
                pub const VARIANT_3: Type = 3;
                pub fn some_fn() -> u32 { 42 }
            }
        };

        let item: syn::ItemMod = syn::parse2(tokens).unwrap();
        let e = ConstEnum::try_from(item);

        if let Err(Error::Api(ApiError::InvalidEnumMod(ident))) = e {
            assert_eq!(ident, "ConstifiedEnum".to_string())
        } else {
            panic!("Unexpected result {:?}", e)
        }
    }

    #[test]
    fn parse_struct() -> Result<()> {
        let tokens = quote! {
            pub struct Structure {
                pub field1: u32,
                pub field2: u64,
            }
        };

        let item: syn::ItemStruct = syn::parse2(tokens).unwrap();
        let s = Struct::try_from(item)?;

        assert_eq!(s.fields.len(), 2);
        assert_eq!(
            s.fields.get(0).unwrap(),
            &(syn_ident!(field1), syn_type!(u32))
        );
        assert_eq!(
            s.fields.get(1).unwrap(),
            &(syn_ident!(field2), syn_type!(u64))
        );

        Ok(())
    }

    #[test]
    fn parse_tuple_struct_fails() {
        let tokens = quote! {
            pub struct Structure(u32, u64);
        };

        let item: syn::ItemStruct = syn::parse2(tokens).unwrap();
        let s = Struct::try_from(item);

        if let Err(Error::Api(ApiError::InvalidStruct(ident))) = s {
            assert_eq!(ident, "Structure".to_string())
        } else {
            panic!("Unexpected result {:?}", s)
        }
    }

    #[test]
    fn parse_union() -> Result<()> {
        let tokens = quote! {
            pub union Union {
                pub field1: u32,
                pub field2: u64,
            }
        };

        let item: syn::ItemUnion = syn::parse2(tokens).unwrap();
        let u = Union::try_from(item)?;

        assert_eq!(u.fields.len(), 2);
        assert_eq!(
            u.fields.get(0).unwrap(),
            &(syn_ident!(field1), syn_type!(u32))
        );
        assert_eq!(
            u.fields.get(1).unwrap(),
            &(syn_ident!(field2), syn_type!(u64))
        );

        Ok(())
    }

    #[test]
    fn parse_function() -> Result<()> {
        let tokens = quote! {
            pub fn some_function(integer: i32, pointer: *const u64) -> u8;
        };

        let item: syn::ForeignItemFn = syn::parse2(tokens).unwrap();
        let f = Function::try_from(item.sig)?;

        assert!(f.abi.is_none());
        assert_eq!(f.args.len(), 2);
        assert!(!f.vararg);
        assert!(f.ret.is_some());

        assert_eq!(
            f.args.get(0).unwrap(),
            &(syn_ident!(integer), syn_type!(i32))
        );
        assert_eq!(
            f.args.get(1).unwrap(),
            &(syn_ident!(pointer), syn_type!(*const u64))
        );
        assert_eq!(&f.ret.unwrap(), &syn_type!(u8));

        Ok(())
    }

    #[test]
    fn api_parse_file() -> Result<()> {
        let tokens = quote! {
            mod child_module;  // Expected to be ignored.

            pub mod ConstifiedEnum {
                pub type Type = u32;
                pub const VARIANT_1: Type = 1;
                pub const VARIANT_2: Type = 2;
                pub const VARIANT_3: Type = 3;
            }

            pub type TypeAlias = u32;

            pub struct Structure {
                pub struct_field1: u32,
                pub struct_field2: u64,
            }

            pub union Union {
                pub union_field1: u32,
                pub union_field2: u64,
            }

            extern "C" {
                pub fn some_function(arg_integer: i32, arg_pointer: *const u64) -> u8;
            }
        };

        let file: syn::File = syn::parse2(tokens).unwrap();
        let mut builder = Builder::new();
        builder.add_file(file)?;
        let api = builder.finish()?;

        assert_eq!(api.enums.len(), 1);
        assert_eq!(api.typedefs.len(), 1);
        assert_eq!(api.structs.len(), 1);
        assert_eq!(api.unions.len(), 1);
        assert_eq!(api.functions.len(), 1);

        let e = api.enums.get(&syn_ident!(ConstifiedEnum)).unwrap();
        assert_eq!(&e.ty, &syn_type!(u32));
        assert_eq!(e.variants.len(), 3);
        assert_eq!(e.variants.get(&syn_ident!(VARIANT_1)).unwrap(), &1);
        assert_eq!(e.variants.get(&syn_ident!(VARIANT_2)).unwrap(), &2);
        assert_eq!(e.variants.get(&syn_ident!(VARIANT_3)).unwrap(), &3);

        let t = api.typedefs.get(&syn_ident!(TypeAlias)).unwrap();
        assert_eq!(t, &syn_type!(u32));

        let s = api.structs.get(&syn_ident!(Structure)).unwrap();
        assert_eq!(s.fields.len(), 2);
        assert_eq!(
            s.fields.get(0).unwrap(),
            &(syn_ident!(struct_field1), syn_type!(u32))
        );
        assert_eq!(
            s.fields.get(1).unwrap(),
            &(syn_ident!(struct_field2), syn_type!(u64))
        );

        let u = api.unions.get(&syn_ident!(Union)).unwrap();
        assert_eq!(u.fields.len(), 2);
        assert_eq!(
            u.fields.get(0).unwrap(),
            &(syn_ident!(union_field1), syn_type!(u32))
        );
        assert_eq!(
            u.fields.get(1).unwrap(),
            &(syn_ident!(union_field2), syn_type!(u64))
        );

        let f = api.functions.get(&syn_ident!(some_function)).unwrap();
        assert!(f.abi.is_some());
        assert_eq!(f.abi.as_ref().unwrap(), "C");
        assert_eq!(f.args.len(), 2);
        assert!(!f.vararg);
        assert!(f.ret.is_some());

        assert_eq!(
            f.args.get(0).unwrap(),
            &(syn_ident!(arg_integer), syn_type!(i32))
        );
        assert_eq!(
            f.args.get(1).unwrap(),
            &(syn_ident!(arg_pointer), syn_type!(*const u64))
        );
        assert_eq!(f.ret.as_ref().unwrap(), &syn_type!(u8));

        Ok(())
    }

    #[test]
    fn api_add_existing_enum_fails() {
        let tokens = quote! {
            pub mod ConstifiedEnum {
                pub type Type = u32;
                pub const VARIANT_1: Type = 1;
                pub const VARIANT_2: Type = 2;
                pub const VARIANT_3: Type = 3;
            }
        };

        let item: syn::ItemMod = syn::parse2(tokens).unwrap();
        let mut builder = Builder::new();
        builder.add_mod(item.clone()).unwrap();

        let ret = builder.add_mod(item);

        if let Err(Error::Api(ApiError::Redefined(ident))) = ret {
            assert_eq!(ident, "ConstifiedEnum")
        } else if let Err(e) = ret {
            panic!("Unexpected error {:?}", e)
        } else {
            panic!("Error expected")
        }
    }

    #[test]
    fn api_add_existing_type_fails() {
        let tokens = quote! {
            pub type TypeAlias = u32;
        };

        let item: syn::ItemType = syn::parse2(tokens).unwrap();
        let mut builder = Builder::new();
        builder.add_type(item.clone()).unwrap();

        let ret = builder.add_type(item);

        if let Err(Error::Api(ApiError::Redefined(ident))) = ret {
            assert_eq!(ident, "TypeAlias")
        } else if let Err(e) = ret {
            panic!("Unexpected error {:?}", e)
        } else {
            panic!("Error expected")
        }
    }

    #[test]
    fn api_add_existing_struct_fails() {
        let tokens = quote! {
            pub struct Structure {
                pub struct_field1: u32,
                pub struct_field2: u64,
            }
        };

        let item: syn::ItemStruct = syn::parse2(tokens).unwrap();
        let mut builder = Builder::new();
        builder.add_struct(item.clone()).unwrap();

        let ret = builder.add_struct(item);

        if let Err(Error::Api(ApiError::Redefined(ident))) = ret {
            assert_eq!(ident, "Structure")
        } else if let Err(e) = ret {
            panic!("Unexpected error {:?}", e)
        } else {
            panic!("Error expected")
        }
    }

    #[test]
    fn api_add_existing_union_fails() {
        let tokens = quote! {
            pub union Union {
                pub union_field1: u32,
                pub union_field2: u64,
            }
        };

        let item: syn::ItemUnion = syn::parse2(tokens).unwrap();
        let mut builder = Builder::new();
        builder.add_union(item.clone()).unwrap();

        let ret = builder.add_union(item);

        if let Err(Error::Api(ApiError::Redefined(ident))) = ret {
            assert_eq!(ident, "Union")
        } else if let Err(e) = ret {
            panic!("Unexpected error {:?}", e)
        } else {
            panic!("Error expected")
        }
    }

    #[test]
    fn api_add_existing_foreign_fn_fails() {
        let tokens = quote! {
            extern "C" {
                pub fn some_function(arg_integer: i32, arg_pointer: *const u64) -> u8;
            }
        };

        let item: syn::ItemForeignMod = syn::parse2(tokens).unwrap();
        let mut builder = Builder::new();
        builder.add_foreign_mod(item.clone()).unwrap();

        let ret = builder.add_foreign_mod(item);

        if let Err(Error::Api(ApiError::Redefined(ident))) = ret {
            assert_eq!(ident, "some_function")
        } else if let Err(e) = ret {
            panic!("Unexpected error {:?}", e)
        } else {
            panic!("Error expected")
        }
    }

    #[test]
    fn match_apis() -> Result<()> {
        let tokens_inner = quote! {
            mod child_module;  // Expected to be ignored.

            pub mod ConstifiedEnum {
                pub type Type = u32;
                pub const VARIANT_1: Type = 1;
                pub const VARIANT_2: Type = 2;
                pub const VARIANT_3: Type = 3;
            }

            pub type TypeAlias = u32;

            pub struct Structure {
                pub field1: u32,
                pub field2: u64,
            }

            pub union Union {
                pub field1: u32,
                pub field2: u64,
            }

            extern "C" {
                pub fn some_function(integer: i32, pointer: *const u64) -> u8;
            }
        };

        let tokens_outer = quote! {
            extern "C" {
                pub fn some_function(integer: i32, pointer: *const u64) -> u8;
                pub fn another_function() -> u32;
            }

            pub type TypeAlias = u32;
            pub type AnotherAlias = bool;

            pub struct Structure {
                pub field1: u32,
                pub field2: u64,
            }

            pub struct AnotherStructure {
                pub field1: f64,
            }

            pub mod ConstifiedEnum {
                pub type Type = u32;
                pub const VARIANT_1: Type = 1;
                pub const VARIANT_2: Type = 2;
                pub const VARIANT_3: Type = 3;
            }

            pub mod AnotherEnum {
                pub type Type = u8;
                pub const VARIANT_1: Type = 100;
                pub const VARIANT_2: Type = 200;
            }

            pub union Union {
                pub field1: u32,
                pub field2: u64,
            }

            pub union AnotherUnion {
                pub field1: Structure,
                pub field2: AnotherStructure,
            }
        };

        let mut bld_inner = Builder::new();
        let mut bld_outer = Builder::new();

        bld_inner.add_file(syn::parse2::<syn::File>(tokens_inner)?)?;
        bld_outer.add_file(syn::parse2::<syn::File>(tokens_outer)?)?;

        let api_inner = bld_inner.finish()?;
        let api_outer = bld_outer.finish()?;

        let m = Matcher::new();

        // Matching with inner and outer APIs swapped must fail
        let mismatch = m.match_apis(&api_outer, &api_inner);
        assert!(mismatch.is_err());

        m.match_apis(&api_inner, &api_outer)
    }

    #[test]
    fn match_enum_types_ignore_sign() -> Result<()> {
        let mut variants = HashMap::new();

        variants.insert(syn_ident!(VAR1), 1);
        variants.insert(syn_ident!(VAR2), 2);
        variants.insert(syn_ident!(VAR3), 3);

        let en1 = ConstEnum {
            ty: syn_type!(u32),
            variants: variants.clone(),
        };

        let en2 = ConstEnum {
            ty: syn_type!(i32),
            variants,
        };

        let mut api_inner = Api::new();
        let mut api_outer = Api::new();

        api_inner.enums.insert(syn_ident!(TEST_ENUM), en1);
        api_outer.enums.insert(syn_ident!(TEST_ENUM), en2);

        let m = Matcher::new();
        m.match_apis(&api_inner, &api_outer)
    }

    #[test]
    fn match_types_equal() {
        let types: Vec<syn::Type> = vec![
            "i64",
            "u64",
            "::std::os::raw::c_uint",
            "*mut ::std::os::raw::c_void",
            "[::std::os::raw::c_char; 64usize]",
        ]
        .into_iter()
        .map(|s| syn::parse_str::<syn::Type>(s).unwrap())
        .collect();

        let ty_matcher = TypeMatcher::new();

        for ty in types {
            assert!(ty_matcher.match_types(&ty, &ty));
        }
    }

    #[test]
    fn match_types_sign_fails() {
        let ty_matcher = TypeMatcher::new();
        assert!(!ty_matcher.match_types(&syn_type!(i8), &syn_type!(u8)));
        assert!(!ty_matcher.match_types(&syn_type!(i16), &syn_type!(u16)));
        assert!(!ty_matcher.match_types(&syn_type!(i32), &syn_type!(u32)));
        assert!(!ty_matcher.match_types(&syn_type!(i64), &syn_type!(u64)));
    }

    #[test]
    fn match_types_ignore_sign() {
        let mut ty_matcher = TypeMatcher::new();
        ty_matcher.ignore_int_sign();
        assert!(ty_matcher.match_types(&syn_type!(i8), &syn_type!(u8)));
        assert!(ty_matcher.match_types(&syn_type!(i16), &syn_type!(u16)));
        assert!(ty_matcher.match_types(&syn_type!(i32), &syn_type!(u32)));
        assert!(ty_matcher.match_types(&syn_type!(i64), &syn_type!(u64)));
    }

    #[test]
    fn match_types_incompatible_ignore_sign_fails() {
        let mut ty_matcher = TypeMatcher::new();
        ty_matcher.ignore_int_sign();
        assert!(!ty_matcher.match_types(&syn_type!(i8), &syn_type!(u32)));
        assert!(!ty_matcher.match_types(&syn_type!(u16), &syn_type!(i64)));
    }

    #[test]
    fn match_types_ptrs_const_fails() {
        let ty_matcher = TypeMatcher::new();
        assert!(!ty_matcher.match_types(&syn_type!(*const i8), &syn_type!(*mut i8)));
    }

    #[test]
    fn match_types_ptrs_ignore_const() {
        let mut ty_matcher = TypeMatcher::new();
        ty_matcher.ignore_ptr_constness();
        // Inner type can introduce constness
        assert!(ty_matcher.match_types(&syn_type!(*const i8), &syn_type!(*mut i8)));
        // Inner type cannot introduce mutability
        assert!(!ty_matcher.match_types(&syn_type!(*mut i8), &syn_type!(*const i8)));

        // Identical types must match
        assert!(ty_matcher.match_types(&syn_type!(*const i8), &syn_type!(*const i8)));
        assert!(ty_matcher.match_types(&syn_type!(*mut i8), &syn_type!(*mut i8)));
    }

    #[test]
    fn match_types_allow_compat_ints() {
        let mut ty_match = TypeMatcher::new();
        ty_match.allow_compatible_ints();
        assert!(ty_match.match_types(&syn_type!(i8), &syn_type!(::std::os::raw::c_schar)));
        assert!(ty_match.match_types(&syn_type!(i16), &syn_type!(::std::os::raw::c_short)));
        assert!(ty_match.match_types(&syn_type!(i32), &syn_type!(::std::os::raw::c_int)));
        assert!(ty_match.match_types(&syn_type!(i64), &syn_type!(::std::os::raw::c_longlong)));
        assert!(ty_match.match_types(&syn_type!(u8), &syn_type!(::std::os::raw::c_uchar)));
        assert!(ty_match.match_types(&syn_type!(u16), &syn_type!(::std::os::raw::c_ushort)));
        assert!(ty_match.match_types(&syn_type!(u32), &syn_type!(::std::os::raw::c_uint)));
        assert!(ty_match.match_types(&syn_type!(u64), &syn_type!(::std::os::raw::c_ulonglong)));

        #[cfg(all(target_pointer_width = "64", not(target_env = "msvc")))]
        {
            assert!(ty_match.match_types(&syn_type!(u64), &syn_type!(::std::os::raw::c_ulong)));
            assert!(ty_match.match_types(&syn_type!(i64), &syn_type!(::std::os::raw::c_long)));
        }

        #[cfg(any(target_pointer_width = "32", target_env = "msvc"))]
        {
            assert!(ty_match.match_types(&syn_type!(u32), &syn_type!(::std::os::raw::c_ulong)));
            assert!(ty_match.match_types(&syn_type!(i32), &syn_type!(::std::os::raw::c_long)));
        }
    }

    #[test]
    fn match_types_allow_compat_incompatible_ints_fail() {
        let mut ty_match = TypeMatcher::new();
        ty_match.allow_compatible_ints();
        assert!(!ty_match.match_types(&syn_type!(i8), &syn_type!(::std::os::raw::c_uchar)));
        assert!(!ty_match.match_types(&syn_type!(i8), &syn_type!(u8)));
        assert!(!ty_match.match_types(&syn_type!(::std::os::raw::c_uchar), &syn_type!(i8)));
        assert!(!ty_match.match_types(&syn_type!(u8), &syn_type!(i8)));
        assert!(!ty_match.match_types(&syn_type!(i8), &syn_type!(i16)));
        assert!(!ty_match.match_types(&syn_type!(i16), &syn_type!(i8)));
        assert!(!ty_match.match_types(&syn_type!(i8), &syn_type!(u16)));
        assert!(!ty_match.match_types(&syn_type!(i16), &syn_type!(u8)));
    }

    #[test]
    fn match_types_allow_compat_ints_ignore_sign() {
        let mut ty_match = TypeMatcher::new();
        ty_match.allow_compatible_ints();
        ty_match.ignore_int_sign();
        assert!(ty_match.match_types(&syn_type!(i8), &syn_type!(::std::os::raw::c_uchar)));
        assert!(ty_match.match_types(&syn_type!(i16), &syn_type!(::std::os::raw::c_ushort)));
        assert!(ty_match.match_types(&syn_type!(i32), &syn_type!(::std::os::raw::c_uint)));
        assert!(ty_match.match_types(&syn_type!(i64), &syn_type!(::std::os::raw::c_ulonglong)));
        assert!(ty_match.match_types(&syn_type!(u8), &syn_type!(::std::os::raw::c_schar)));
        assert!(ty_match.match_types(&syn_type!(u16), &syn_type!(::std::os::raw::c_short)));
        assert!(ty_match.match_types(&syn_type!(u32), &syn_type!(::std::os::raw::c_int)));
        assert!(ty_match.match_types(&syn_type!(u64), &syn_type!(::std::os::raw::c_longlong)));

        #[cfg(all(target_pointer_width = "64", not(target_env = "msvc")))]
        {
            assert!(ty_match.match_types(&syn_type!(u64), &syn_type!(::std::os::raw::c_long)));
            assert!(ty_match.match_types(&syn_type!(i64), &syn_type!(::std::os::raw::c_ulong)));
        }

        #[cfg(any(target_pointer_width = "32", target_env = "msvc"))]
        {
            assert!(ty_match.match_types(&syn_type!(u32), &syn_type!(::std::os::raw::c_long)));
            assert!(ty_match.match_types(&syn_type!(i32), &syn_type!(::std::os::raw::c_ulong)));
        }
    }

    #[test]
    fn match_types_follow_typedefs() {
        let inner_defs: HashMap<syn::Ident, syn::Type> = vec![
            (syn_ident!(InnerFinalDef), syn_type!(InnerMidDef)),
            (syn_ident!(InnerMidDef), syn_type!(u32)),
        ]
        .into_iter()
        .collect();

        let outer_defs: HashMap<syn::Ident, syn::Type> = vec![
            (syn_ident!(OuterFinalDef), syn_type!(OuterMidDef)),
            (syn_ident!(OuterMidDef), syn_type!(u32)),
        ]
        .into_iter()
        .collect();

        let mut ty_match = TypeMatcher::new();
        ty_match.use_typedefs(&inner_defs, &outer_defs);

        assert!(ty_match.match_types(&syn_type!(InnerFinalDef), &syn_type!(OuterFinalDef)));
        assert!(ty_match.match_types(&syn_type!(InnerFinalDef), &syn_type!(OuterMidDef)));
        assert!(ty_match.match_types(&syn_type!(InnerMidDef), &syn_type!(OuterFinalDef)));
        assert!(ty_match.match_types(&syn_type!(InnerFinalDef), &syn_type!(OuterMidDef)));
        assert!(ty_match.match_types(&syn_type!(u32), &syn_type!(OuterFinalDef)));
        assert!(ty_match.match_types(&syn_type!(u32), &syn_type!(OuterMidDef)));
        assert!(ty_match.match_types(&syn_type!(InnerFinalDef), &syn_type!(u32)));
        assert!(ty_match.match_types(&syn_type!(InnerMidDef), &syn_type!(u32)));

        assert!(!ty_match.match_types(&syn_type!(InnerFinalDef), &syn_type!(InnerFinalDef)));
        assert!(!ty_match.match_types(&syn_type!(InnerFinalDef), &syn_type!(InnerMidDef)));
        assert!(!ty_match.match_types(&syn_type!(InnerMidDef), &syn_type!(InnerFinalDef)));
    }
}
