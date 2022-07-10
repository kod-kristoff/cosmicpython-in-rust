use crate::repository::SqlXRepository;
use chrono;
use domain::model;
use sqlx::sqlite::SqlitePool;

#[tokio::test]
async fn test_orm() {
    let db = setup_db().await;
}

#[tokio::test]
async fn test_repository_can_save_a_batch() {
    let session = setup_db().await;
    let batch = model::Batch::new(
        "batch1".to_owned(),
        "RUSTY-SOAPDISH".to_owned(),
        100,
        chrono::Utc::today(),
    );

    let repo = SqlXRepository::new(session);
    repo.add(batch).await;
    // session.commit();

    let rows = sqlx::query("SELECT reference, sku, _purchased_quantity, eta FROM batches")
        .fetch_all(&session)
        .await
        .expect("Failed to read from DB");
    assert_eq!(rows, [("batch1", "RUSTY-SOAPDISH", 100, None)]);
}

async fn setup_db() -> SqlitePool {
    let db = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to db");
    db
}
