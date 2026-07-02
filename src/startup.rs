use crate::routes::{health_check, subcribe, subcribe_one_span};
use actix_web::{
    App, HttpServer,
    dev::Server,
    middleware::Logger,
    web::{self, Data},
};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
) -> Result<Server, std::io::Error> {
    // wrap in the the Data ptr type so that it
    // can be cloned across applications
    let db_pool = Data::new(db_pool);

    let server = HttpServer::new(move || {
        App::new()
            // Middlewares added using wrap method
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subcribe_one_span))
            // registers the pool to actix app
            // this handles the shared state
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
