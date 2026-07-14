use crate::helpers::spawn_app;

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
async fn subscribe_returns_a_400_when_name_is_invalid() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let long_str = "o".repeat(257);
    let body = format!("name={long_str}&email=something");
    let test_cases = vec![
        ("name=le%20guin".to_string(), "missing the email"),
        (
            "name=\\&email=ursula_le_guin%40gmail.com".to_string(),
            "missing the name",
        ),
        (
            // exceeding lenght limit
            body.to_string(),
            "missing both name and email",
        ),
        (
            "name={}()&email=le%40gmail.com".to_string(),
            "invalid characters",
        ),
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

#[tokio::test]
async fn invalid_names_return_a_400() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=  &email=something", "name is only white space"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
        ("name={}()&email=le%40gmail.com", "invalid characters"),
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

#[tokio::test]
async fn invalid_email_addresses_returns_a_400() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=fine&email=something", "incorrect email format"),
        ("name=fine&email=ursula_le_guin.com", "missing the name"),
        // ("name=fine&email=some%40gmail", "incorrect email format"),
        // ("name=fine&email=some%40gmail.co", "incorrect email format"),
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
