use sqlx::sqlite::SqlitePool;

pub mod repositories;

pub async fn run_migrations(db: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::migrate!("./migrations").run(db).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
