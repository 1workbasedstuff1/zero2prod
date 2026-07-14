use crate::domain::NewSubscriber;
use actix_web::{HttpResponse, web};
use chrono::Utc;
use log::log_enabled;
use sqlx::PgPool;
use tracing::{self, Instrument, span::Entered};
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
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

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
pub async fn subscribe(
    form: web::Form<NewSubscriber>,
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
    form: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email.as_ref(),
        form.name.as_ref(), // we need the ampersand to perform the conversion into a &str
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
