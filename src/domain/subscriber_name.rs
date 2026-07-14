use sqlx;
use std::{convert::TryFrom, fmt::Display, ops::Deref};
use unicode_segmentation::UnicodeSegmentation;

use crate::routes::FormData;

// EXPLAIN:
// this is to try and implement the same extractor pattern the actix uses
// for example when we can .subscribe() we actually pass a raw string
// to process it into Form<FormData> so ive use the same pattern but
// instead it will parse it into the NewSubscriber type

// needed to be called in sqlx functions
#[derive(serde::Deserialize, sqlx::Type, Debug)]
#[serde(try_from = "String")] // constructor from string type
#[sqlx(transparent)]
pub struct SubscriberName(String);

// needed for the tracing macros
use std::fmt;
impl fmt::Display for SubscriberName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for SubscriberName {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

pub fn is_valid_name(s: &str) -> bool {
    // trim removes all white space
    let is_empty_or_whitespace = s.trim().is_empty();

    // imported from graphemes
    let is_too_long = s.graphemes(true).count() > 256;

    let forbidden_characters =
        ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

    let contains_forbidden =
        s.chars().any(|g| forbidden_characters.contains(&g));

    !(is_empty_or_whitespace || is_too_long || contains_forbidden)
}

impl TryFrom<String> for SubscriberName {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match is_valid_name(&value) {
            true => Ok(SubscriberName(value)),
            false => Err("Subscirber name is invalid".to_string()),
        }
    }
}

impl TryInto<SubscriberName> for &str {
    type Error = String;
    fn try_into(self) -> Result<SubscriberName, Self::Error> {
        match is_valid_name(self) {
            true => Ok(SubscriberName(self.to_string())),
            false => Err("Subscriber name is invalid".to_string()),
        }
    }
}

// impl TryInto<SubscriberName> for String {
//     type Error = String;
//     fn try_into(self) -> Result<SubscriberName, Self::Error> {
//         match is_valid_name(&self) {
//             true => Ok(SubscriberName(self.to_string())),
//             false => Err("Subscriber name is invalid".to_string()),
//         }
//     }
// }

// run some unit tests
#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "a".repeat(256);
        // let sub_name: Result<SubscriberName, _> = name.try_into();
        // assert_ok!(name.try_into())
        claim::assert_ok!(SubscriberName::try_from(name));
    }

    #[test]
    fn name_longer_than_256_rejected() {
        let name = "a".repeat(257);
        // we use claim because it will show us the error message
        claim::assert_err!(SubscriberName::try_from(name));
    }

    #[test]
    fn whitespace_only_names() {
        let name = " ".to_string();
        claim::assert_err!(SubscriberName::try_from(name));
    }

    #[test]
    fn names_with_invalid_characters_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(SubscriberName::try_from(name));
        }
    }
}
