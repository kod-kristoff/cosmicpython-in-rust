use sqlx::{sqlite::SqlitePool, Row};
use std::{
    collections::{HashMap, HashSet},
    net::TcpListener,
};
use webapp::startup;

fn random_suffix() -> String {
    use rand::{distributions::Alphanumeric, Rng};

    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(6)
        .map(char::from)
        .collect()
}

fn random_sku(name: &str) -> String {
    format!("sku-{}-{}", name, random_suffix())
}

fn random_batchref(name: &str) -> String {
    format!("batch-{}-{}", name, random_suffix())
}

fn random_orderid(name: &str) -> String {
    format!("order-{}-{}", name, random_suffix())
}

async fn add_stock(
    session: &SqlitePool,
    lines: &[(String, String, u32, Option<&str>)],
) -> (HashSet<u32>, HashSet<String>) {
    const INSERT_BATCHES: &str = "
        INSERT INTO batches (reference, sku, _purchased_quantity, eta)
        VALUES ($1, $2, $3, $4)
    ";
    const SELECT_BATCH: &str = "
        SELECT id 
        FROM batches 
        WHERE reference=$1 AND sku=$2
    ";
    let mut batches_added = HashSet::new();
    let mut sku_added = HashSet::new();
    for (reference, sku, qty, eta) in lines {
        sqlx::query(INSERT_BATCHES)
            .bind(&reference)
            .bind(&sku)
            .bind(qty)
            .bind(eta)
            .execute(session)
            .await
            .expect("insert batch");
        let row = sqlx::query(SELECT_BATCH)
            .bind(&reference)
            .bind(&sku)
            .fetch_one(session)
            .await
            .expect("select batch");
        let batch_id: u32 = row.try_get("id").expect("get id");
        batches_added.insert(batch_id);
        sku_added.insert(sku.clone());
    }
    (batches_added, sku_added)
}

#[tokio::test]
async fn api_returns_allocation() {
    // Arrange
    let sku = random_sku("");
    let othersku = random_sku("other");
    let earlybatch = random_batchref("1");
    let laterbatch = random_batchref("2");
    let otherbatch = random_batchref("3");
    let app = spawn_app().await;
    let (batches_added, sku_added) = add_stock(
        &app.db_pool,
        &[
            (laterbatch, sku.clone(), 100, Some("2011-01-02")),
            (earlybatch.clone(), sku.clone(), 100, Some("2011-01-01")),
            (otherbatch, othersku, 100, None),
        ],
    )
    .await;
    let client = reqwest::Client::new();

    // Act
    let data = serde_json::json!({
        "orderid": random_orderid(""),
        "sku": sku.clone(),
        "qty": 3,
    });
    let response = client
        .post(format!("{}/allocate", &app.address))
        .json(&data)
        .send()
        .await
        .expect("Failed to execute request");
    let response_status = response.status().as_u16();
    let response_json = response
        .json::<HashMap<String, String>>()
        .await
        .expect("Failed to parse json");
    // Assert
    assert_eq!(response_status, 201);
    assert_eq!(response_json["batchref"], earlybatch);
}

pub struct TestApp {
    pub address: String,
    pub db_pool: SqlitePool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let db_pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to db");

    infrastructure::run_migrations(&db_pool)
        .await
        .expect("failed to run migrations");
    let db_pool_clone = db_pool.clone();
    tokio::spawn(async move { startup::run(listener, db_pool_clone).await });

    TestApp { address, db_pool }
}
