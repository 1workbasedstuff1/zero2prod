use secrecy::ExposeSecret;
use sqlx::postgres::PgPool;
use std::net::TcpListener;
use std::time::Duration; // used to get random ip
use zero2prod::configuration::get_configuration;
// weve moved all tracing functionality to these create
use sqlx::postgres::PgPoolOptions;
use zero2prod::startup::run;
use zero2prod::telemtry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("started");
    // NOTE: subscribe to the error messages
    let subscriber =
        get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration =
        get_configuration().expect("Failed to read config");

    // need connection url
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy(
            &configuration.database.connection_string().expose_secret(),
        )
        // .await
        .expect("Failed to connect to Postgres");

    // NOTE: changes this for docker local and production flags
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );

    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await?;
    Ok(())
}
