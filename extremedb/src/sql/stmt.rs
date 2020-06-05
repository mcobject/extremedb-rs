// stmt.rs
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr;

use crate::sql::allocator::{Owned, Ref};
use crate::sql::data_source::DataSource;
use crate::sql::engine::Engine;
use crate::sql::result_from_code;
use crate::sql::trans::Transaction;
use crate::sql::value::{ToValue, Value};
use crate::{exdb_sys, Result};

pub(crate) struct Statement {}

impl Statement {
    pub(crate) fn execute_statement(
        ctx: ExecutionContext,
        sql: &str,
        values: &[&dyn ToValue],
    ) -> Result<i64> {
        let alloc = Owned::new()?;
        let mut sql_values = Statement::create_values(Ref::new(&alloc), values)?;
        let mut n_records = MaybeUninit::uninit();

        result_from_code(unsafe {
            exdb_sys::mcosql_rs_statement_execute(
                ctx.engine,
                ctx.transaction,
                n_records.as_mut_ptr(),
                sql.as_ptr() as *const i8,
                sql_values.as_mut_ptr() as *mut exdb_sys::mcosql_rs_value,
                sql_values.len(),
            )
        })
        .and(Ok(unsafe { n_records.assume_init() }))
    }

    pub(crate) fn execute_query<'c>(
        ctx: ExecutionContext<'c>,
        sql: &str,
        values: &[&dyn ToValue],
    ) -> Result<Option<DataSource<'c>>> {
        let alloc = Owned::new()?;
        let mut sql_values = Statement::create_values(Ref::new(&alloc), values)?;
        let mut ds = MaybeUninit::uninit();

        result_from_code(unsafe {
            exdb_sys::mcosql_rs_query_execute(
                ctx.engine,
                ctx.transaction,
                ds.as_mut_ptr(),
                sql.as_ptr() as *const i8,
                sql_values.as_mut_ptr() as *mut exdb_sys::mcosql_rs_value,
                sql_values.len(),
            )
        })?;

        let ds = unsafe { ds.assume_init() };

        if ds.is_null() {
            Ok(None)
        } else {
            Ok(Some(DataSource::new(ds, &())))
        }
    }

    fn create_values<'a>(alloc: Ref<'a>, values: &[&dyn ToValue]) -> Result<Vec<Value<'a>>> {
        let mut ret = Vec::with_capacity(values.len());
        for val in values {
            ret.push(val.to_value(alloc)?);
        }
        Ok(ret)
    }
}

pub(crate) struct ExecutionContext<'a> {
    owner: PhantomData<&'a ()>,
    engine: exdb_sys::database_t,
    transaction: exdb_sys::transaction_t,
}

impl<'a> ExecutionContext<'a> {
    pub(crate) fn with_engine<E: Engine + ?Sized>(engine: &'a E) -> Self {
        ExecutionContext {
            owner: PhantomData,
            engine: engine.get_engine(),
            transaction: ptr::null_mut(),
        }
    }

    pub(crate) fn with_transaction(transaction: &'a Transaction) -> Self {
        ExecutionContext {
            owner: PhantomData,
            engine: transaction.engine.get_engine(),
            transaction: transaction.h,
        }
    }
}
