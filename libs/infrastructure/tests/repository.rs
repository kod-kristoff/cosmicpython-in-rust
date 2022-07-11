use chrono;
use domain::model;
use infrastructure::repositories::SqlxRepository;
use sqlx::{sqlite::SqlitePool, Row};

use futures_util::TryStreamExt;

#[tokio::test]
async fn test_orm() {
    let db = setup_db().await;
}

#[tokio::test]
async fn test_repository_can_save_a_batch() -> Result<(), Box<dyn std::error::Error>> {
    let session = setup_db().await;
    let batch = model::Batch::new("batch1".to_owned(), "RUSTY-SOAPDISH".to_owned(), 100, None);

    let repo = SqlxRepository::new(session.clone());
    repo.add(batch).await;
    // session.commit();

    let mut rows =
        sqlx::query("SELECT reference, sku, _purchased_quantity, eta FROM batches").fetch(&session);
    // assert_eq!(rows.len(), 1);
    if let Some(row) = rows.try_next().await? {
        let reference: &str = row.try_get("reference")?;
        let sku: &str = row.try_get("sku")?;
        let qty: u32 = row.try_get("_purchased_quantity")?;
        let eta: Option<chrono::NaiveDateTime> = row.try_get("eta")?;
        assert_eq!(reference, "batch1");
        assert_eq!(sku, "RUSTY-SOAPDISH");
        assert_eq!(qty, 100);
        assert_eq!(eta, None);
    }
    Ok(())
}

async fn setup_db() -> SqlitePool {
    const CREATE_BATCHES: &str = "
        CREATE TABLE IF NOT EXISTS batches
        (
            id                    INTEGER PRIMARY KEY NOT NULL,
            reference             TEXT                NOT NULL,
            sku                   STRING(128)         NOT NULL,
            _purchased_quantity   INTEGER             NOT NULL,
            eta                   DATETIME
    )
    ";
    let db = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to db");
    sqlx::query(CREATE_BATCHES)
        .execute(&db)
        .await
        .expect("failed to create batches");
    db
}
