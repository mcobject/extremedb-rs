// rssql.cpp
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

#include "rssql.h"

#include "mco.h"

#include "sql/mcoapiseq.h"
#include "sql/mcosql.h"
#include "sql/sqlcpp.h"


#define CATCH_AND_RETURN()                      \
    catch (const McoSql::McoSqlException &e) {  \
        return (status_t)e.code;                \
    } catch (const std::exception &) {          \
        return RUNTIME_ERROR;                   \
    }


status_t mcors_sql_session_create(database_t database, mcors_sql_session *session)
{
    McoSqlEngine *engine = (McoSqlEngine *)database;
    try {
        *session = (mcors_sql_session)(new McoSqlSession(engine));
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_session_destroy(mcors_sql_session session)
{
    McoSqlSession *sess = (McoSqlSession *)session;
    try {
        delete sess;
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_database_allocator(database_t database, mcors_sql_allocator *allocator)
{
    McoSql::SqlEngine *engine = (McoSql::SqlEngine *)database;
    try {
        *allocator = (mcors_sql_allocator)engine->getAllocator();
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_transaction_allocator(transaction_t transaction,
    mcors_sql_allocator *allocator)
{
    McoSql::Transaction *txn = (McoSql::Transaction *)transaction;
    *allocator = (mcors_sql_allocator)txn->allocator;
    return SQL_OK;
}

status_t mcors_sql_allocator_create(mcors_sql_allocator *allocator)
{
    try {
        McoSql::Allocator *alloc = new McoSql::Allocator();
        *allocator = (mcors_sql_allocator)alloc;
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_allocator_destroy(mcors_sql_allocator allocator)
{
    McoSql::Allocator *alloc = (McoSql::Allocator *)allocator;
    try {
        delete alloc;
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_value_create_null(mcors_sql_value *out)
{
    *out = (mcors_sql_value)&McoSql::Null;
    return SQL_OK;
}

status_t mcors_sql_value_create_bool(int val, mcors_sql_value *out)
{
    *out = (mcors_sql_value)McoSql::BoolValue::create(val != 0);
    return SQL_OK;
}

status_t mcors_sql_value_create_int(mcors_sql_allocator allocator, mco_int8 val,
    mcors_sql_value *out)
{
    McoSql::Allocator *alloc = (McoSql::Allocator *)allocator;
    try {
        *out = (mcors_sql_value)McoSql::IntValue::create(alloc, val);
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_value_create_real(mcors_sql_allocator allocator, double val,
    mcors_sql_value *out)
{
    McoSql::Allocator *alloc = (McoSql::Allocator *)allocator;
    try {
        *out = (mcors_sql_value)McoSql::RealValue::create(alloc, val);
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_value_create_datetime(mcors_sql_allocator allocator,
    mco_datetime val, mcors_sql_value *out)
{
    McoSql::Allocator *alloc = (McoSql::Allocator *)allocator;
    try {
        *out = (mcors_sql_value)McoSql::DateTime::create(alloc, val);
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_value_create_numeric(mcors_sql_allocator allocator,
    mco_int8 val, size_t prec, mcors_sql_value *out)
{
    McoSql::Allocator *alloc = (McoSql::Allocator *)allocator;
    try {
        *out = (mcors_sql_value)McoSql::NumericValue::create(alloc, val, prec);
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_value_create_string(mcors_sql_allocator allocator,
    const char *p, size_t len, mcors_sql_value *out)
{
    McoSql::Allocator *alloc = (McoSql::Allocator *)allocator;
    try {
        *out = (mcors_sql_value)McoSql::String::create(alloc, p, len);
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_value_create_binary(mcors_sql_allocator allocator,
    const void *p, size_t len, mcors_sql_value *out)
{
    McoSql::Allocator *alloc = (McoSql::Allocator *)allocator;
    try {
        *out = (mcors_sql_value)McoSql::Binary::create(alloc, p, len);
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_value_create_array(mcors_sql_allocator allocator,
    type_t elem_type, size_t size, mcors_sql_value *out)
{
    McoSql::Allocator *alloc = (McoSql::Allocator *)allocator;
    try {
        *out = (mcors_sql_value)McoSql::Array::create(
            alloc, (McoSql::Type)elem_type, 0, size);
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_value_type(mcors_sql_value val, type_t *out)
{
    McoSql::Value *v = (McoSql::Value *)val;
    try {
        *out = (type_t)v->type();
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_value_size(mcors_sql_value val, size_t *out)
{
    McoSql::Value *v = (McoSql::Value *)val;
    try {
        *out = v->size();
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

int mcors_sql_value_is_null(mcors_sql_value val)
{
    McoSql::Value *v = (McoSql::Value *)val;
    return (v->isNull() ? 1 : 0);
}

int mcors_sql_value_is_true(mcors_sql_value val)
{
    McoSql::Value *v = (McoSql::Value *)val;
    return (v->isTrue() ? 1 : 0);
}

status_t mcors_sql_value_int(mcors_sql_value val, mco_int8 *out)
{
    McoSql::Value *v = (McoSql::Value *)val;
    try {
        *out = v->intValue();
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_value_real(mcors_sql_value val, double *out)
{
    McoSql::Value *v = (McoSql::Value *)val;
    try {
        *out = v->realValue();
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_value_datetime(mcors_sql_value val, mco_datetime *out)
{
    McoSql::Value *v = (McoSql::Value *)val;
    try {
        *out = v->timeValue();
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_value_numeric(mcors_sql_value val, mco_int8 *out_value,
    size_t *out_precision)
{
    McoSql::NumericValue *v = (McoSql::NumericValue *)val;
    if (v->type() != McoSql::tpNumeric) {
        return INVALID_OPERATION;
    }
    try {
        *out_value = v->scale(0);
        *out_precision = (size_t)v->precision;
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_value_string_ref(mcors_sql_value val,
    mcors_sql_allocator allocator, mcors_sql_value_ref *out)
{
    McoSql::Allocator *alloc = (McoSql::Allocator *)allocator;
    McoSql::Value *v = (McoSql::Value *)val;
    try {
        McoSql::Ref<McoSql::String> ref(v->stringRef(alloc));
        out->allocator = (mcors_sql_allocator)ref.getAllocator();
        out->ref = (mcors_sql_value)ref.grab();
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_value_binary(mcors_sql_value val,
    mcors_sql_allocator allocator, mcors_sql_value_ref *out)
{
    McoSql::Allocator *alloc = (McoSql::Allocator *)allocator;
    McoSql::Value *v = (McoSql::Value *)val;
    try {
        McoSql::Ref<McoSql::Binary> ref(v->binaryValue(alloc));
        out->allocator = (mcors_sql_allocator)ref.getAllocator();
        out->ref = (mcors_sql_value)ref.grab();
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_value_ptr(mcors_sql_value val, void **out)
{
    McoSql::Value *v = (McoSql::Value *)val;
    try {
        *out = v->pointer();
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_array_is_plain(mcors_sql_value array, int *out)
{
    McoSql::Array *arr = (McoSql::Array *)array;
    if (arr->type() != McoSql::tpArray) {
        return INVALID_OPERATION;
    }
    try {
        *out = (arr->isPlain() ? 1 : 0);
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_array_allocator(mcors_sql_value array,
    mcors_sql_allocator *allocator)
{
    McoSql::Array *arr = (McoSql::Array *)array;
    if (arr->type() != McoSql::tpArray) {
        return INVALID_OPERATION;
    } else if (arr->allocator == NULL) {
        return RUNTIME_ERROR;
    }
    *allocator = (mcors_sql_allocator)arr->allocator;
    return SQL_OK;
}

status_t mcors_sql_array_elem_type(mcors_sql_value array, type_t *out)
{
    McoSql::Array *arr = (McoSql::Array *)array;
    if (arr->type() != McoSql::tpArray) {
        return INVALID_OPERATION;
    }
    try {
        *out = (type_t)arr->getElemType();
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_array_get_at(mcors_sql_value array, size_t at,
    mcors_sql_value_ref *out)
{
    McoSql::Array *arr = (McoSql::Array *)array;
    if (arr->type() != McoSql::tpArray) {
        return INVALID_OPERATION;
    }
    try {
        McoSql::ValueRef ref(arr->getAt(at));
        out->allocator = (mcors_sql_allocator)ref.getAllocator();
        out->ref = (mcors_sql_value)ref.grab();
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_array_set_at(mcors_sql_value array, size_t at,
    mcors_sql_value value)
{
    McoSql::Array *arr = (McoSql::Array *)array;
    McoSql::Value *val = (McoSql::Value *)value;
    if (arr->type() != McoSql::tpArray) {
        return INVALID_OPERATION;
    } else if (arr->getElemType() != val->type()) {
        return INVALID_TYPE_CAST;
    }
    try {
        arr->setAt(at, val);
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_array_set_body(mcors_sql_value array,
    const void *elems, size_t n_elems)
{
    McoSql::Array *arr = (McoSql::Array *)array;
    if (arr->type() != McoSql::tpArray) {
        return INVALID_OPERATION;
    }
    try {
        arr->setBody((void *)elems, 0, n_elems);
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_seq_allocator(mcors_sql_value sequence,
    mcors_sql_allocator *allocator)
{
    McoGenericSequence *seq = (McoGenericSequence *)sequence;
    if (seq->type() != McoSql::tpSequence) {
        return INVALID_OPERATION;
    }
    *allocator = (mcors_sql_allocator)seq->allocator;
    return SQL_OK;
}

status_t mcors_sql_seq_count(mcors_sql_value sequence, size_t *out)
{
    McoGenericSequence *seq = (McoGenericSequence *)sequence;
    if (seq->type() != McoSql::tpSequence) {
        return INVALID_OPERATION;
    }
    try {
        *out = seq->count();
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_seq_elem_type(mcors_sql_value sequence, type_t *out)
{
    McoGenericSequence *seq = (McoGenericSequence *)sequence;
    if (seq->type() != McoSql::tpSequence) {
        return INVALID_OPERATION;
    }
    *out = (type_t)seq->elemType;
    return SQL_OK;
}

status_t mcors_sql_seq_get_iterator(mcors_sql_value sequence)
{
    McoGenericSequence *seq = (McoGenericSequence *)sequence;
    if (seq->type() != McoSql::tpSequence) {
        return INVALID_OPERATION;
    }
    try {
        seq->getIterator();
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_seq_reset(mcors_sql_value sequence)
{
    McoGenericSequence *seq = (McoGenericSequence *)sequence;
    if (seq->type() != McoSql::tpSequence) {
        return INVALID_OPERATION;
    }
    try {
        seq->reset();
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_seq_next(mcors_sql_value sequence, mcors_sql_value *out)
{
    McoGenericSequence *seq = (McoGenericSequence *)sequence;
    if (seq->type() != McoSql::tpSequence) {
        return INVALID_OPERATION;
    }
    try {
        *out = (mcors_sql_value)seq->next();
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_blob_available(mcors_sql_value blob, size_t *out)
{
    McoSql::Blob *blo = (McoSql::Blob *)blob;
    if (blo->type() != McoSql::tpBlob) {
        return INVALID_OPERATION;
    }
    try {
        *out = blo->available();
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_blob_get(mcors_sql_value blob, void *buf, size_t size,
    size_t *out)
{
    McoSql::Blob *blo = (McoSql::Blob *)blob;
    if (blo->type() != McoSql::tpBlob) {
        return INVALID_OPERATION;
    }
    try {
        *out = blo->get(buf, size);
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_blob_reset(mcors_sql_value blob, size_t pos)
{
    McoSql::Blob *blo = (McoSql::Blob *)blob;
    if (blo->type() != McoSql::tpBlob) {
        return INVALID_OPERATION;
    }
    try {
        blo->reset(pos);
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_value_release(mcors_sql_allocator allocator,
    mcors_sql_value sql_value)
{
    McoSql::Allocator *alloc = (McoSql::Allocator *)allocator;
    McoSql::Value *val = (McoSql::Value *)sql_value;
    try {
        DELETE_OBJ(alloc, val);
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_statement_execute(database_t database, transaction_t transaction,
    mco_int8 *n_records, const char *sql, mcors_sql_value *values,
    size_t n_values)
{
    McoSql::SqlEngine *engine = (McoSql::SqlEngine *)database;
    McoSql::Transaction *txn = (McoSql::Transaction *)transaction;
    McoSql::Value **vals = (McoSql::Value **)values;
    try {
        int64_t n_recs = engine->vexecuteStatement(txn, sql, vals, n_values);
        if (n_records != NULL) {
            *n_records = n_recs;
        }
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_query_execute(database_t database, transaction_t transaction,
    data_source_t *data_source, const char *sql, mcors_sql_value *values,
    size_t n_values)
{
    McoSql::SqlEngine *engine = (McoSql::SqlEngine *)database;
    McoSql::Transaction *txn = (McoSql::Transaction *)transaction;
    McoSql::Value **vals = (McoSql::Value **)values;
    try {
        McoSql::DataSource *ds = engine->vexecuteQuery(txn, sql, vals, n_values);
        if (data_source != NULL) {
            *data_source = (data_source_t)ds;
        } else {
            ds->release();
        }
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}

status_t mcors_sql_record_allocator(record_t record, mcors_sql_allocator *allocator)
{
    McoSql::Record *rec = (McoSql::Record *)record;
    if (rec->type() != McoSql::tpStruct) {
        return INVALID_OPERATION;
    } else if (rec->allocator == NULL) {
        return RUNTIME_ERROR;
    }
    *allocator = (mcors_sql_allocator)rec->allocator;
    return SQL_OK;
}

status_t mcors_sql_record_get_column_value_ref(record_t record, size_t column_no,
    mcors_sql_value_ref *out)
{
    McoSql::Record *rec = (McoSql::Record *)record;
    try {
        McoSql::ValueRef ref(rec->get(column_no));
        out->allocator = (mcors_sql_allocator)ref.getAllocator();
        out->ref = (mcors_sql_value)ref.grab();
    }
    CATCH_AND_RETURN()
    return SQL_OK;
}
