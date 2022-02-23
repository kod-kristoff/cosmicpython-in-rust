use sea_orm::{Database, DatabaseConnection};

#[tokio::test]
async fn test_orm() {
    let db = setup_db().await;
}

async fn setup_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to db");
    db
}
