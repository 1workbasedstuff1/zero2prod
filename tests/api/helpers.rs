use std::net::TcpListener;

use quickcheck::Testable;
use reqwest;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::configuration::{DatabaseSettings, get_configuration};
use zero2prod::email_client::EmailClient;
use zero2prod::startup::{self, Application, get_connection_pool};
use zero2prod::telemtry::{get_subscriber, init_subscriber};

use once_cell::sync::Lazy;

// we define whether we want the logging messages or not
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::stdout,
        );
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::sink,
        );
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// NOTE: added telemetry to our tests
// BUG: testing will fail because Logger is a global variable,
// and attempting to set it multiple times will cause our code
// to panic with
// Failed to set logger: SetLoggerError(())
// so we need to wrap it in a global variable
pub async fn spawn_app() -> TestApp {
    // logging messages should appear as name
    // "debug" is our env filter
    // BUG:
    // let subscriber = get_subscriber("test".into(), "debug".into());
    // init_subscriber(subscriber);

    Lazy::force(&TRACING);

    // randomise configuration to ensure isolation
    let configuration = {
        let mut c =
            get_configuration().expect("Failed to read configuration");
        // use a different data base for each test
        c.database.database_name = Uuid::new_v4().to_string();
        // use random OS port
        c.application.port = 0;
        c
    };

    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");

    let application_port = application.port();
    let _ = tokio::spawn(application.read_until_stopped());

    TestApp {
        address: format!("http://127.0.0.1:{}", application_port),
        db_pool: get_connection_pool(&configuration.database),
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // forms connectoin to PostGres
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(sqlx::AssertSqlSafe(format!(
            r#"CREATE DATABASE "{}";"#,
            config.database_name
        )))
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}
