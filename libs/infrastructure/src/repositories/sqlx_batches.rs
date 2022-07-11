use domain::model;
use sqlx::sqlite::SqlitePool;

pub struct SqlxRepository {
    pool: SqlitePool,
}

impl SqlxRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn add(&self, batch: model::Batch) {
        const QUERY: &str = "INSERT INTO batches 
            (reference, sku, _purchased_quantity, eta)
            VALUES ($1, $2, $3, $4)";
        sqlx::query(QUERY)
            .bind(batch.reference())
            .bind(batch.sku())
            .bind(batch.purchased_quantity())
            .bind(batch.eta())
            .execute(&self.pool)
            .await
            .expect("repositories/sqlx_batches: inserting batch");
    }
}
