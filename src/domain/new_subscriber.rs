use crate::domain::SubscriberEmail;
use crate::domain::SubscriberName;
use crate::routes::FormData;

#[derive(serde::Deserialize)]
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = value.name.try_into()?;
        let email = value.email.try_into()?;
        Ok(NewSubscriber { email, name })
    }
}
