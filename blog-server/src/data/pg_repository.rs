use sqlx::PgPool;

#[derive(Clone)]
pub struct PgRepository {
    pub pool: PgPool,
}

impl PgRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}
