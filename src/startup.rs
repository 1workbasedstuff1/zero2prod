use crate::{
    configuration::{DatabaseSettings, Settings},
    email_client::{self, EmailClient},
    routes::{health_check, subscribe},
};
use actix_web::{
    App, HttpServer,
    dev::Server,
    web::{self, Data},
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{net::TcpListener, time::Duration};
use tracing_actix_web::TracingLogger;

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    // wrap in the the Data ptr type so that it
    // can be cloned across applications
    let db_pool = Data::new(db_pool);
    let email_client = Data::new(email_client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            // registers the pool to actix app
            // this handles the shared state
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(
        configuration: Settings,
    ) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        // parse a valid email
        let sender_email = configuration
            .email_client
            .sender()
            .expect("invalid email address");

        // this holds the details to perform communication with a private email server
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            std::time::Duration::from_secs(2),
        );

        // NOTE: changes this for docker local and production flags
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, connection_pool, email_client)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn read_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
