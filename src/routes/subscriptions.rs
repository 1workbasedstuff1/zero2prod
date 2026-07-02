use actix_web::{HttpResponse, web};
use chrono::Utc;
use log::log_enabled;
use sqlx::PgPool;
use tracing::{self, Instrument, span::Entered};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

// parsing happens before hand in web::Form
pub async fn subcribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "request id  - Adding  as a new subscriber",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name
    );

    let _request_span_guard = request_span.enter();
    let query_span = tracing::info_span!("Saving new subscriber",);

    let query_response = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(pool.as_ref())
    .instrument(query_span)
    .await;

    // WARN: SPAN should not live over an await point
    // thats why we add instrument

    match query_response {
        Ok(_) => {
            tracing::info!(
                "request_id: {} - New subscriber details have been saved",
                request_id
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// impl<'a> Drop for Entered<'a> {
//     #[inline]
//     fn drop(&mut self) {
//         // Dropping the guard exits the span.
//         //
//         // Running this behaviour on drop rather than with an explicit function
//         // call means that spans may still be exited when unwinding.
//         if let Some(inner) = self.span.inner.as_ref() {
//             inner.subscriber.exit(&inner.id);
//         }
//         if_log_enabled! {{
//         if let Some(ref meta) = self.span.meta {
//         self.span.log(
//         ACTIVITY_LOG_TARGET,
//         log::Level::Trace,
//         format_args!("<- {}", meta.name())
//         );
//         }
//         }}
//     }
// }
//

// designed to clean up the previous code
// also refactored database query into its own function
// NOTE: we remove request_id do that actix_logger gives us a uniques
// ID for each span
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
pub async fn subcribe_one_span(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    match insert_subscriber(&pool, &form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(
    name = "Saving subscriber details in database",
    skip(form, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    form: &FormData,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    });
    Ok(())
}
