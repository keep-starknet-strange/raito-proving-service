use crate::{
    error::{AppError, Result},
    model::{BlocksQuery, BlocksResponse, HeaderStatus, HealthStatus, TransactionStatus},
    store::MockStore,
};
use axum::{
    extract::{Path, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use utoipa::OpenApi;
use validator::Validate;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_blocks,
        get_block_by_identifier,
        get_block_proof,
        get_transaction_status,
        get_header_status,
        health_check,
    ),
    components(
        schemas(
            crate::model::BlockSummary,
            crate::model::BlockDetail,
            crate::model::BlocksResponse,
            crate::model::TransactionStatus,
            crate::model::HeaderStatus,
            crate::model::HealthStatus,
            crate::model::BlocksQuery,
        )
    ),
    tags(
        (name = "blocks", description = "Block operations"),
        (name = "proofs", description = "STARK proof operations"),
        (name = "transactions", description = "Transaction verification"),
        (name = "headers", description = "Block header verification"),
        (name = "health", description = "Service health checks"),
    )
)]
pub struct ApiDoc;

#[utoipa::path(
    get,
    path = "/v1/blocks",
    responses(
        (status = 200, description = "List of blocks", body = BlocksResponse),
        (status = 400, description = "Invalid query parameters"),
    )
)]
pub async fn get_blocks(Query(query): Query<BlocksQuery>) -> Result<Json<BlocksResponse>> {
    query
        .validate()
        .map_err(|e| AppError::InvalidQueryParameter(format!("Validation failed: {}", e)))?;

    let limit = query.limit.unwrap_or(20);
    let store = MockStore::global();
    let response = store.get_blocks(limit, query.cursor);

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/v1/blocks/{identifier}",
    responses(
        (status = 200, description = "Block details", body = crate::model::BlockDetail),
        (status = 400, description = "Invalid block identifier"),
        (status = 404, description = "Block not found"),
    )
)]
pub async fn get_block_by_identifier(
    Path(identifier): Path<String>,
) -> Result<Json<crate::model::BlockDetail>> {
    let store = MockStore::global();

    let block = if let Ok(height) = identifier.parse::<u32>() {
        store.get_block_by_height(height)?
    } else if identifier.len() == 64
        && identifier
            .chars()
            .all(|c| c.is_ascii_hexdigit() || c == '0')
    {
        store.get_block_by_hash(&identifier)?
    } else {
        return Err(AppError::InvalidBlockIdentifier(identifier));
    };

    Ok(Json(block.clone()))
}

#[utoipa::path(
    get,
    path = "/v1/blocks/{height}/proof",
    responses(
        (status = 200, description = "STARK proof file"),
        (status = 404, description = "Block or proof not found"),
    )
)]
pub async fn get_block_proof(Path(height): Path<u32>) -> Result<Response> {
    let store = MockStore::global();
    let proof_data = store.get_proof_file(height)?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"block_{}_proof.json\"", height),
        )
        .header(header::CONTENT_LENGTH, proof_data.len())
        .body(proof_data.into())
        .map_err(|_| AppError::Internal)?;

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/v1/tx/{txid}",
    responses(
        (status = 200, description = "Transaction status", body = TransactionStatus),
        (status = 400, description = "Invalid transaction ID"),
    )
)]
pub async fn get_transaction_status(Path(txid): Path<String>) -> Result<Json<TransactionStatus>> {
    if txid.len() != 64 {
        return Err(AppError::InvalidTransactionId(format!(
            "Invalid length: {}, expected 64",
            txid.len()
        )));
    }

    if !txid.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::InvalidTransactionId(format!(
            "Contains non-hex characters: {}",
            txid
        )));
    }

    let store = MockStore::global();
    let status = store.get_transaction_status(&txid)?;

    Ok(Json(status))
}

#[utoipa::path(
    get,
    path = "/v1/header/{hash}",
    responses(
        (status = 200, description = "Header status", body = HeaderStatus),
        (status = 400, description = "Invalid header hash"),
    )
)]
pub async fn get_header_status(Path(hash): Path<String>) -> Result<Json<HeaderStatus>> {
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit() || c == '0') {
        return Err(AppError::InvalidHeaderHash(hash));
    }

    let store = MockStore::global();
    let status = store.get_header_status(&hash)?;

    Ok(Json(status))
}

#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Service is healthy", body = HealthStatus),
    )
)]
pub async fn health_check() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "up".to_string(),
        timestamp: Utc::now().timestamp(),
    })
}

pub async fn metrics_handler() -> impl IntoResponse {
    "# Metrics will be implemented here\n"
}
