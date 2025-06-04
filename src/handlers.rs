use crate::{
    database::Database,
    error::{AppError, Result},
    model::{BlocksQuery, BlocksResponse, HeaderStatus, HealthStatus, TransactionStatus},
};
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use std::sync::Arc;
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
pub async fn get_blocks(
    State(db): State<Arc<Database>>,
    Query(query): Query<BlocksQuery>,
) -> Result<Json<BlocksResponse>> {
    query
        .validate()
        .map_err(|e| AppError::InvalidQueryParameter(format!("Validation failed: {e}")))?;

    let limit = query.limit.unwrap_or(20);
    let response = db.get_blocks(limit, query.cursor).await?;

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
    State(db): State<Arc<Database>>,
    Path(identifier): Path<String>,
) -> Result<Json<crate::model::BlockDetail>> {
    let block = if let Ok(height) = identifier.parse::<u32>() {
        db.get_block_by_height(height).await?
    } else if identifier.len() == 64
        && identifier
            .chars()
            .all(|c| c.is_ascii_hexdigit() || c == '0')
    {
        db.get_block_by_hash(&identifier).await?
    } else {
        return Err(AppError::InvalidBlockIdentifier(identifier));
    };

    Ok(Json(block))
}

#[utoipa::path(
    get,
    path = "/v1/blocks/{height}/proof",
    responses(
        (status = 200, description = "STARK proof file"),
        (status = 404, description = "Block or proof not found"),
    )
)]
pub async fn get_block_proof(
    State(db): State<Arc<Database>>,
    Path(height): Path<u32>,
) -> Result<Response> {
    // Check if block exists
    if !db.block_exists_by_identifier(&height.to_string()).await? {
        return Err(AppError::BlockNotFound(height.to_string()));
    }

    // Check if proof file exists in database
    if !db.proof_file_exists(height).await? {
        return Err(AppError::ProofNotFound(height.to_string()));
    }

    // Load proof file from filesystem
    let proof_path = format!("data/proofs/{height}.json");
    let proof_data =
        std::fs::read(&proof_path).map_err(|_| AppError::ProofNotFound(height.to_string()))?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"block_{height}_proof.json\""),
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
pub async fn get_transaction_status(
    State(db): State<Arc<Database>>,
    Path(txid): Path<String>,
) -> Result<Json<TransactionStatus>> {
    if txid.len() != 64 {
        return Err(AppError::InvalidTransactionId(format!(
            "Invalid length: {}, expected 64",
            txid.len()
        )));
    }

    if !txid.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::InvalidTransactionId(format!(
            "Contains non-hex characters: {txid}"
        )));
    }

    let status = db.get_transaction_status(&txid).await?;

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
pub async fn get_header_status(
    State(db): State<Arc<Database>>,
    Path(hash): Path<String>,
) -> Result<Json<HeaderStatus>> {
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit() || c == '0') {
        return Err(AppError::InvalidHeaderHash(hash));
    }

    let status = db.get_header_status(&hash).await?;

    Ok(Json(status))
}

#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Service is healthy", body = HealthStatus),
    )
)]
pub async fn health_check(State(db): State<Arc<Database>>) -> Result<Json<HealthStatus>> {
    // Perform database health check
    db.health_check().await?;

    Ok(Json(HealthStatus {
        status: "up".to_string(),
        timestamp: Utc::now().timestamp(),
    }))
}

pub async fn metrics_handler() -> impl IntoResponse {
    "# Metrics will be implemented here\n"
}
