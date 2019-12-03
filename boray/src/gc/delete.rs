// Copyright 2019 Joyent, Inc.

use serde_derive::{Deserialize, Serialize};
use serde_json::Error as SerdeError;
use serde_json::{json, Value};
use slog::{debug, error, Logger};
use uuid::Uuid;

use cueball_postgres_connection::PostgresConnection;
use rust_fast::protocol::{FastMessage, FastMessageData};
use tokio_postgres::Error as PGError;

use crate::sql;
use crate::types::{HandlerResponse, HasRequestId};
use crate::util::array_wrap;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DeleteGarbagePayload {
    pub request_id: Uuid,
    pub batch_id: Uuid,
}

impl HasRequestId for DeleteGarbagePayload {
    fn request_id(&self) -> Uuid {
        self.request_id
    }
}

pub(crate) fn decode_msg(value: &Value) -> Result<Vec<DeleteGarbagePayload>, SerdeError> {
    serde_json::from_value::<Vec<DeleteGarbagePayload>>(value.clone())
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn action(
    msg_id: u32,
    method: &str,
    log: &Logger,
    payload: DeleteGarbagePayload,
    conn: &mut PostgresConnection,
) -> Result<HandlerResponse, String> {
    // Make database request
    do_delete(&payload, conn, log)
        .and_then(|_affected_rows| {
            // Handle the successful database response
            debug!(log, "{} operation was successful", &method);

            let value = json!("ok");

            let msg_data = FastMessageData::new(method.into(), array_wrap(value));
            let msg: HandlerResponse = FastMessage::data(msg_id, msg_data).into();
            Ok(msg)
        })
        .or_else(|e| {
            // Handle database error response
            error!(log, "{} operation failed: {}", &method, &e);

            // Database errors are returned to as regular Fast messages
            // to be handled by the calling application
            let value = array_wrap(json!({
                "name": "PostgresError",
                "message": e
            }));

            let msg_data = FastMessageData::new(method.into(), value);
            let msg: HandlerResponse = FastMessage::data(msg_id, msg_data).into();
            Ok(msg)
        })
}

fn do_delete(
    _payload: &DeleteGarbagePayload,
    conn: &mut PostgresConnection,
    log: &Logger,
) -> Result<(), String> {
    let mut txn = (*conn).transaction().map_err(|e| e.to_string())?;

    sql::txn_query(
        sql::Method::GarbageGet,
        &mut txn,
        get_garbage_records_sql(),
        &[],
        log,
    )
    .and_then(|garbage_rows| {
        // Delete the records in the materialized view
        if garbage_rows.is_empty() {
            Ok(0)
        } else {
            let mut last_result: Result<u64, PGError> = Ok(0);

            for row in garbage_rows {
                let schema: String = row.get("schma");
                let id: Uuid = row.get("id");
                let owner: Uuid = row.get("owner");
                let bucket_id: Uuid = row.get("bucket_id");
                let name: String = row.get("name");

                let delete_stmt = delete_garbage_sql(schema);

                last_result = sql::txn_execute(
                    sql::Method::GarbageRecordDelete,
                    &mut txn,
                    delete_stmt.as_str(),
                    &[&owner, &bucket_id, &name, &id],
                    log,
                )
            }

            last_result
        }
    })
    .and_then(|_| {
        // Refresh the view
        sql::txn_execute(
            sql::Method::GarbageRefresh,
            &mut txn,
            refresh_garbage_view_sql(),
            &[],
            log,
        )
    })
    .and_then(|_| {
        // Update the batch id
        let batch_id = Uuid::new_v4();
        sql::txn_query(
            sql::Method::GarbageBatchIdUpdate,
            &mut txn,
            update_garbage_batch_id_sql(),
            &[&batch_id],
            log,
        )
    })
    .and_then(|_| {
        // All steps completed without error so commit the transaction
        txn.commit()
    })
    .map_err(|e| e.to_string())
}

fn get_garbage_records_sql() -> &'static str {
    "SELECT schma, id, name, owner, bucket_id FROM GARBAGE_BATCH"
}

fn delete_garbage_sql(schema: String) -> String {
    [
        "DELETE FROM ",
        &schema,
        ".manta_bucket_deleted_object WHERE owner = $1 AND bucket_id = $2 AND \
         name = $3 AND id = $4",
    ]
    .concat()
}

fn refresh_garbage_view_sql() -> &'static str {
    "REFRESH MATERIALIZED VIEW GARBAGE_BATCH"
}

fn update_garbage_batch_id_sql() -> &'static str {
    "UPDATE garbage_batch_id SET batch_id = $1 WHERE id = 1"
}
