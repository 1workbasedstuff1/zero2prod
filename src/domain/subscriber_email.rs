use validator::ValidateEmail;

#[derive(Debug, serde::Deserialize, sqlx::Type, Clone)]
#[serde(try_from = "String")] // constructor from string type
#[sqlx(transparent)]
pub struct SubscriberEmail(String);

use std::fmt;

impl fmt::Display for SubscriberEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

use std::ops::Deref;
impl Deref for SubscriberEmail {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// WARN: added extra pattern match to find the .com or . after a
// @gmail
impl TryFrom<String> for SubscriberEmail {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value.validate_email() {
            return Err("value is not a valid email".to_string());
        }
        // Extra check: require a dot in the domain part
        match value.split('@').nth(1) {
            Some(domain) if domain.contains('.') => {
                Ok(SubscriberEmail(value))
            }
            _ => Err("value is not a valid email".to_string()),
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claim::{assert_err, assert_ok};

    use fake::faker::internet::en::SafeEmail;
    use fake::rand::SeedableRng;
    use fake::rand::rngs::StdRng;
    use fake::{Fake, rand};
    use rand::RngExt;

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::try_from(email));
    }
    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_err!(SubscriberEmail::try_from(email));
    }
    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::try_from(email));
    }

    #[test]
    fn is_valid_email() {
        let email = "someuser@gmail.com".to_string();
        claim::assert_ok!(SubscriberEmail::try_from(email));
    }

    // EXPLAIN: why is this allowed?
    // because this is a valid email on a private network
    #[test]
    fn missing_dot_com() {
        let email = "someuser@gmail".to_string();
        claim::assert_err!(SubscriberEmail::try_from(email));
    }

    #[test]
    fn valid_emails_are_passed_successfully() {
        let email: String = SafeEmail().fake();
        claim::assert_ok!(SubscriberEmail::try_from(email));
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let seed = u64::arbitrary(g);
            let mut rng = StdRng::seed_from_u64(seed);
            let email = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }
    // NOTE fails because we haven't provided any information about
    // how to create a an email to the quickcheck TRAIT
    // fn valid_email_parsed(valid_email: String) -> bool
    #[quickcheck_macros::quickcheck]
    fn valid_email_parsed(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::try_from(valid_email.0).is_ok()
    }
}
