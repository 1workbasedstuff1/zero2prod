use secrecy::ExposeSecret;
use sqlx::{self, PgPool};
use zero2prod::configuration::get_configuration;

#[tokio::main]
async fn main() {
    let data_base_address = get_configuration()
        .expect("failed to find or parse config")
        .database
        .without_db();

    let pool = PgPool::connect_with(data_base_address)
        .await
        .expect("Failed to connect to Postgres");

    let rows = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch saved subcriber");

    for row in rows {
        println!("name: {}, email: {}", row.name, row.email);
    }
}
