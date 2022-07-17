use std::collections::HashSet;

use chrono::{Datelike, NaiveDate, NaiveDateTime};
use domain::model;
use sqlx::{sqlite::SqlitePool, Row};

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
            .bind(encode_date(batch.eta()))
            .execute(&self.pool)
            .await
            .expect("repositories/sqlx_batches: inserting batch");
    }

    pub async fn get(&self, reference: &str) -> model::Batch {
        const QUERY: &str = "
            SELECT id, reference, sku, _purchased_quantity, eta
            FROM batches
            WHERE reference=$1
        ";
        const ALLOCATIONS_QUERY: &str = "
            SELECT order_lines.sku, order_lines.qty, order_lines.orderid
            FROM order_lines
            LEFT JOIN allocations
            ON order_lines.id = allocations.orderline_id
            AND allocations.batch_id = $1
        ";
        let row = sqlx::query(QUERY)
            .bind(reference)
            .fetch_one(&self.pool)
            .await
            .expect("repositories/sqlx_batches: get batch");

        let batch_id: u32 = row.try_get("id").unwrap();
        let reference: String = row.try_get("reference").expect("");
        let sku: String = row.try_get("sku").unwrap();
        let purchased_quantity: u32 = row.try_get("_purchased_quantity").unwrap();
        let eta: Option<chrono::Date<chrono::Utc>> = decode_date(row.try_get("eta").unwrap());

        let mut allocations = HashSet::new();
        let allocation_rows = sqlx::query(ALLOCATIONS_QUERY)
            .bind(batch_id)
            .fetch_all(&self.pool)
            .await
            .expect("");
        for allocation_row in allocation_rows {
            let allocation_sku: String = allocation_row.try_get("sku").unwrap();
            let orderid: String = allocation_row.try_get("orderid").unwrap();
            let qty: u32 = allocation_row.try_get("qty").unwrap();
            allocations.insert(model::OrderLine::new(orderid, allocation_sku, qty));
        }
        model::Batch::with_allocations(reference, sku, purchased_quantity, eta, allocations)
    }

    pub async fn list(&self) -> Vec<model::Batch> {
        const QUERY: &str = "
            SELECT reference, sku, _purchased_quantity, eta
            FROM batches
        ";

        sqlx::query(QUERY)
            .fetch_all(&self.pool)
            .await
            .expect("")
            .iter()
            .map(|row| {
                model::Batch::new(
                    row.get("reference"),
                    row.get("sku"),
                    row.get("_purchased_quantity"),
                    decode_date(row.get("eta")),
                )
            })
            .collect()
    }
}

fn encode_date(date: Option<&chrono::Date<chrono::Utc>>) -> Option<NaiveDate> {
    date.map(|d| NaiveDate::from_ymd(d.year(), d.month(), d.day()))
}

fn decode_date(date: Option<NaiveDate>) -> Option<chrono::Date<chrono::Utc>> {
    date.map(|d| chrono::Date::from_utc(d, chrono::Utc))
}
