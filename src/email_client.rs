use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize, Serializer};

// dont want clone, we only need shared access to
// the http_client, so cloning would effect performance
// we need to wrap our connection in a shareable reference pointer
// #[derive(Clone)]
pub struct EmailClient {
    sender: SubscriberEmail,
    http_client: Client,
    base_url: String,
    authorization_token: SecretString,
}

// adding time is breaking change
impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: SecretString,
        timeout: std::time::Duration,
    ) -> Self {
        // configure the http client server properties with timeout
        let http_client =
            Client::builder().timeout(timeout).build().unwrap();
        Self {
            http_client,
            base_url,
            sender,
            authorization_token,
        }
    }
}

// NOTE: we have to go the the PostMark API to understand
// the development
impl EmailClient {
    // async IO
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);

        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject: subject.as_ref(),
            html_body: html_content.as_ref(),
            text_body: text_content.as_ref(),
        };

        // NOTE: sending the reqwest fixed our test
        // let builder = self.http_client.post(&self.base_url);
        let builder = self
            .http_client
            .post(&url)
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
    // #[serde(serialize_with = "serialize_secret")]
    // pub authorization_token: SecretString,
}

// NOTE: API changed for Secret to no longer
// have blanket implementation of Serialize
fn serialize_secret<S>(
    secret: &SecretString,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(secret.expose_secret())
}

#[cfg(test)]
mod test {
    use crate::domain::SubscriberEmail;
    use crate::email_client::{self, EmailClient};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::SecretBox;
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use wiremock::Request;

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct SendEmailBodyMatcher;

    // this fails because of case requirements
    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            // parse value into json
            let result: Result<serde_json::Value, _> =
                serde_json::from_slice(&request.body);

            // now check successful parse matches postmark api
            if let Ok(body) = result {
                // check all mandatory fields are populated
                // without inspecting the individual fields
                dbg!(&body);
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    // create functions to help tests look cleaner
    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    // generate random subscriber email
    fn email() -> SubscriberEmail {
        SubscriberEmail::try_from(SafeEmail().fake::<String>()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            SecretBox::new(Box::from(Faker.fake::<String>())),
            std::time::Duration::from_secs(5),
        )
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // this is a full blown HTTP server
        // asks the OS for a random port, spins up server
        // on a background thread
        let mock_server = MockServer::start().await;
        let sender = email();

        // retreive email address of mock server
        // point our email client to the mock server
        let email_client = EmailClient::new(
            mock_server.uri(),
            sender,
            SecretBox::new(Box::from(Faker.fake::<String>())),
            std::time::Duration::from_secs(5),
        );

        // Mock usually just returns 404 Not Found error to everything
        // Instruct it to behave differently by mounting a Mock
        // Mock::given(any())
        //     .respond_with(ResponseTemplate::new(200))
        //     .expect(1) // this is the hidden assertion of our test
        //     // expect to see at least one request
        //     .mount(&mock_server)
        //     .await;

        // EXPLAIN: check that a given header exsists
        // Mock::given(header_exists("X-Postmark-Server-Token"))
        //     .respond_with(ResponseTemplate::new(200))
        //     .expect(1)
        //     .mount(&mock_server)
        //     .await;

        // EXPLAIN: this is actually example of a builder pattern
        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            // use custom matcher
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email =
            SubscriberEmail::try_from(SafeEmail().fake::<String>())
                .unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Sentence(1..10).fake();

        // ACT
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // ASSERT
        // Mock expectations are checked on drop
    }

    // new happy path test
    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_a_200() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        let subscriber_email = email();
        let subject = subject();
        let content = content();
        // We do not copy in all the matchers we have in the other test.
        // The purpose of this test is not to assert on the request we
        // are sending out!
        // We add the bare minimum needed to trigger the path we want
        // to test in `send_email`.
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;
        // Act
        let outcome = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;
        // Assert
        claim::assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let subscriber_email =
            SubscriberEmail::try_from(SafeEmail().fake::<String>())
                .unwrap();

        let subject: String = Sentence(1..10).fake();
        let content: String = Paragraph(1..10).fake();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        claim::assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // Arrange
        let mock_server = MockServer::start().await;
        let sender =
            SubscriberEmail::try_from(SafeEmail().fake::<String>())
                .unwrap();

        let email_client = EmailClient::new(
            mock_server.uri(),
            sender,
            SecretBox::new(Box::from(Faker.fake::<String>())),
            std::time::Duration::from_secs(5),
        );

        // set up a random email
        let subscriber_email =
            SubscriberEmail::try_from(SafeEmail().fake::<String>())
                .unwrap();

        // construct the sections of our response
        let subject: String = Sentence(1..10).fake();
        let content: String = Paragraph(1..10).fake();

        let response = ResponseTemplate::new(200)
            // NOTE set timer of 3 minutes to trigger time out
            .set_delay(std::time::Duration::from_secs(190));

        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        claim::assert_err!(outcome);
    }
}
