use std::default;
use std::net::TcpListener;

use reqwest;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tracing::subscriber;
use uuid::Uuid;
use zero2prod::configuration::{DatabaseSettings, get_configuration};
use zero2prod::startup;
use zero2prod::telemtry::{get_subscriber, init_subscriber};
// use zero2prod::

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // create database
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: "password".to_string().into(),
        port: config.port,
        host: config.host.clone(),
    };

    // forms connectoin to PostGres
    let mut connection = PgConnection::connect(
        &maintenance_settings.connection_string().expose_secret(),
    )
    .await
    .expect("Failed to connect to Postgres");

    // EXPLAIN: we have to wrap string in AssertSqlSafe for security reasons
    // designed to prevent injection attacks
    // [-]
    // then we tell Postgres to make a new data base
    connection
        .execute(sqlx::AssertSqlSafe(format!(
            r#"CREATE DATABASE "{}";"#,
            config.database_name
        )))
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool =
        PgPool::connect(&config.connection_string().expose_secret())
            .await
            .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}

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

// NOTE: added telemetry to our tests
// BUG: testing will fail because Logger is a global variable,
// and attempting to set it multiple times will cause our code
// to panic with
// Failed to set logger: SetLoggerError(())
// so we need to wrap it in a global variable
async fn spawn_app() -> TestApp {
    // logging messages should appear as name
    // "debug" is our env filter
    // BUG:
    // let subscriber = get_subscriber("test".into(), "debug".into());
    // init_subscriber(subscriber);

    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");

    // get the final number of the random port
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration =
        get_configuration().expect("Failed to read config");
    // change the name to a random one
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool =
        configure_database(&configuration.database).await;

    let server = startup::run(listener, connection_pool.clone())
        .expect("Failed to bind address");

    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // ping our server
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscriber_returns_200_from_valid_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=henry%20figglebottom&email=figgysmalls%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "figgysmalls@gmail.com");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
