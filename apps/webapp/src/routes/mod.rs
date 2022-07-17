use axum::extract::{Extension, Json};
use axum::http::StatusCode;
use domain::model;
use infrastructure::repositories;
use sqlx::SqlitePool;

#[derive(serde::Deserialize)]
pub struct Allocate {
    pub orderid: String,
    pub sku: String,
    pub qty: u32,
}

pub async fn allocate(
    Json(data): Json<Allocate>,
    Extension(db_pool): Extension<SqlitePool>,
) -> (StatusCode, Json<serde_json::Value>) {
    let repo = repositories::SqlxRepository::new(db_pool);
    let mut batches = repo.list().await;

    let line = model::OrderLine::new(data.orderid, data.sku, data.qty);
    let batchref = model::allocate(line, &mut batches).expect("");
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "batchref": batchref })),
    )
}
