// rssql.h
//
// This file is a part of the eXtremeDB source code
// Copyright (c) 2020 McObject LLC
// All Rights Reserved

#ifdef __cplusplus
extern "C" {
#endif

#include "mcospec.h"

#include "sql/sqlc.h"


typedef void *mcors_sql_allocator;

typedef void *mcors_sql_session;

// Pointer to McoSql::Value.
typedef void *mcors_sql_value;

typedef struct mcors_sql_value_ref_
{
    mcors_sql_allocator allocator;
    mcors_sql_value ref;
} mcors_sql_value_ref;

status_t mcors_sql_session_create(database_t database, mcors_sql_session *session);

status_t mcors_sql_session_destroy(mcors_sql_session session);

status_t mcors_sql_database_allocator(database_t database,
    mcors_sql_allocator *allocator);

status_t mcors_sql_transaction_allocator(transaction_t transaction,
    mcors_sql_allocator *allocator);

status_t mcors_sql_allocator_create(mcors_sql_allocator *allocator);

status_t mcors_sql_allocator_destroy(mcors_sql_allocator allocator);

status_t mcors_sql_value_create_null(mcors_sql_value *out);

status_t mcors_sql_value_create_bool(int val, mcors_sql_value *out);

status_t mcors_sql_value_create_int(mcors_sql_allocator allocator, mco_int8 val,
    mcors_sql_value *out);

status_t mcors_sql_value_create_real(mcors_sql_allocator allocator, double val,
    mcors_sql_value *out);

status_t mcors_sql_value_create_datetime(mcors_sql_allocator allocator,
    mco_datetime val, mcors_sql_value *out);

status_t mcors_sql_value_create_numeric(mcors_sql_allocator allocator,
    mco_int8 val, size_t prec, mcors_sql_value *out);

status_t mcors_sql_value_create_string(mcors_sql_allocator allocator,
    const char *p, size_t len, mcors_sql_value *out);

status_t mcors_sql_value_create_binary(mcors_sql_allocator allocator,
    const void *p, size_t len, mcors_sql_value *out);

status_t mcors_sql_value_create_array(mcors_sql_allocator allocator,
    type_t elem_type, size_t size, mcors_sql_value *out);

status_t mcors_sql_value_type(mcors_sql_value val, type_t *out);

status_t mcors_sql_value_size(mcors_sql_value val, size_t *out);

int mcors_sql_value_is_null(mcors_sql_value val);

int mcors_sql_value_is_true(mcors_sql_value val);

status_t mcors_sql_value_int(mcors_sql_value val, mco_int8 *out);

status_t mcors_sql_value_real(mcors_sql_value val, double *out);

status_t mcors_sql_value_datetime(mcors_sql_value val, mco_datetime *out);

status_t mcors_sql_value_numeric(mcors_sql_value val, mco_int8 *out_value,
    size_t *out_precision);

status_t mcors_sql_value_string_ref(mcors_sql_value val,
    mcors_sql_allocator allocator, mcors_sql_value_ref *out);

status_t mcors_sql_value_binary(mcors_sql_value val,
    mcors_sql_allocator allocator, mcors_sql_value_ref *out);

status_t mcors_sql_value_ptr(mcors_sql_value val, void **out);

status_t mcors_sql_array_is_plain(mcors_sql_value array, int *out);

status_t mcors_sql_array_allocator(mcors_sql_value array,
    mcors_sql_allocator *allocator);

status_t mcors_sql_array_elem_type(mcors_sql_value array, type_t *out);

status_t mcors_sql_array_get_at(mcors_sql_value array, size_t at,
    mcors_sql_value_ref *out);

status_t mcors_sql_array_set_at(mcors_sql_value array, size_t at,
    mcors_sql_value value);

status_t mcors_sql_array_set_body(mcors_sql_value array,
    const void *elems, size_t n_elems);

status_t mcors_sql_seq_allocator(mcors_sql_value sequence,
    mcors_sql_allocator *allocator);

status_t mcors_sql_seq_count(mcors_sql_value sequence, size_t *out);

status_t mcors_sql_seq_elem_type(mcors_sql_value sequence, type_t *out);

status_t mcors_sql_seq_get_iterator(mcors_sql_value sequence);

status_t mcors_sql_seq_reset(mcors_sql_value sequence);

status_t mcors_sql_seq_next(mcors_sql_value sequence, mcors_sql_value *out);

status_t mcors_sql_blob_available(mcors_sql_value blob, size_t *out);

status_t mcors_sql_blob_get(mcors_sql_value blob, void *buf, size_t size,
    size_t *out);

status_t mcors_sql_blob_reset(mcors_sql_value blob, size_t pos);

status_t mcors_sql_value_release(mcors_sql_allocator allocator,
    mcors_sql_value sql_value);

status_t mcors_sql_statement_execute(database_t database, transaction_t transaction,
    mco_int8 *n_records, const char *sql, mcors_sql_value *values,
    size_t n_values);

status_t mcors_sql_query_execute(database_t database, transaction_t transaction,
    data_source_t *data_source, const char *sql, mcors_sql_value *values,
    size_t n_values);

status_t mcors_sql_record_allocator(record_t record, mcors_sql_allocator *allocator);

status_t mcors_sql_record_get_column_value_ref(record_t record, size_t column_no,
    mcors_sql_value_ref *out);

#ifdef __cplusplus
}
#endif
