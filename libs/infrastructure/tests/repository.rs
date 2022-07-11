use std::collections::HashSet;

use chrono;
use domain::model;
use infrastructure::repositories::SqlxRepository;
use sqlx::{sqlite::SqlitePool, Row};

use futures_util::TryStreamExt;

#[tokio::test]
async fn repository_can_retrieve_a_batch_with_allocations() {
    let session = setup_db().await;
    let orderline_id = insert_order_line(&session).await;
    let batch1_id = insert_batch(&session, "batch1").await;
    insert_batch(&session, "batch2").await;
    insert_allocation(&session, orderline_id, batch1_id).await;

    let repo = SqlxRepository::new(session);
    let retrieved = repo.get("batch1").await;

    let expected = model::Batch::new("batch1".to_owned(), "GENERIC-SOFA".to_owned(), 100, None);

    assert_eq!(retrieved, expected);
    assert_eq!(retrieved.sku(), expected.sku());
    assert_eq!(
        retrieved.purchased_quantity(),
        expected.purchased_quantity()
    );
    let mut expected_allocations = HashSet::new();
    expected_allocations.insert(model::OrderLine::new(
        "order1".to_owned(),
        "GENERIC-SOFA".to_owned(),
        12,
    ));
    assert_eq!(retrieved.allocations(), &expected_allocations);
}

#[tokio::test]
async fn repository_can_save_a_batch() -> Result<(), Box<dyn std::error::Error>> {
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

async fn insert_order_line(session: &SqlitePool) -> u32 {
    sqlx::query(
        "INSERT INTO order_lines (orderid, sku, qty)
        VALUES ('order1', 'GENERIC-SOFA', 12)",
    )
    .execute(session)
    .await
    .expect("insert order line");

    let row = sqlx::query(
        "SELECT id FROM order_lines
            WHERE orderid=$1 AND sku=$2",
    )
    .bind("order1")
    .bind("GENERIC-SOFA")
    .fetch_one(session)
    .await
    .expect("fetching order_line id");
    let orderline_id: u32 = row.try_get("id").expect("get id");
    orderline_id
}

async fn insert_batch(session: &SqlitePool, batch_id: &str) -> u32 {
    const INSERT_BATCHES: &str = "
        INSERT INTO batches (reference, sku, _purchased_quantity, eta)
        VALUES ($1, 'GENERIC-SOFA', 100, null)
    ";
    const SELECT_BATCH: &str = "
        SELECT id 
        FROM batches 
        WHERE reference=$1 AND sku='GENERIC-SOFA'
    ";

    sqlx::query(INSERT_BATCHES)
        .bind(batch_id)
        .execute(session)
        .await
        .expect("insert batch");
    let row = sqlx::query(SELECT_BATCH)
        .bind(batch_id)
        .fetch_one(session)
        .await
        .expect("select batch");
    let batch_id: u32 = row.try_get("id").expect("get id");
    batch_id
}

async fn insert_allocation(session: &SqlitePool, orderline_id: u32, batch_id: u32) {
    const INSERT_ALLOCATIONS: &str = "
        INSERT INTO allocations (orderline_id, batch_id)
        VALUES ($1, $2)
    ";

    sqlx::query(INSERT_ALLOCATIONS)
        .bind(orderline_id)
        .bind(batch_id)
        .execute(session)
        .await
        .expect("insert allocation");
}

async fn setup_db() -> SqlitePool {
    let db = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to db");
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("running migrations");
    db
}
