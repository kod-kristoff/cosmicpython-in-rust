use sqlx::sqlite::SqlitePool;

pub struct SqlxRepository {}

impl SqlxRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self {}
    }
}
