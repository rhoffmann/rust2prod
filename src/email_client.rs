use secrecy::{Secret, ExposeSecret};
use serde::Serialize;
use crate::domain::SubscriberEmail;

pub struct EmailClient {
    sender: SubscriberEmail,
    http_client: reqwest::Client,
    base_url: reqwest::Url,
    authorization_token: Secret<String>,
}

#[derive(Serialize)]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    text: &'a str,
    html: &'a str,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail, authorization_token: Secret<String>) -> Self {
        let base_url = reqwest::Url::parse(&base_url).expect("Invalid base URL");
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap();

        Self {
            sender,
            http_client,
            base_url,
            authorization_token,
        }
    }

    pub async fn send_email(&self, to: SubscriberEmail, subject: &str, html_content: &str, text_content: &str) -> Result<(), reqwest::Error> {
        let url = self.base_url.join("email").unwrap();

        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: to.as_ref(),
            subject,
            text: text_content,
            html: html_content,
        };

        self.http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.authorization_token.expose_secret()))
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};
    use fake::{Fake, Faker};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Sentence, Paragraph};
    use secrecy::Secret;
    use wiremock::{Mock, MockServer, Request};
    use wiremock::matchers::{any, header, header_exists, method, path};
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

            if let Ok(body) = result {
                body.get("from").is_some()
                    && body.get("to").is_some()
                    && body.get("subject").is_some()
                    && body.get("text").is_some()
                    && body.get("html").is_some()
            } else {
                false
            }
        }
    }

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(base_url, email(), Secret::new(Faker.fake()))
    }

    #[tokio::test]
    async fn send_email_sends_expected_request() {
        // arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("Authorization"))
            .and(method("POST"))
            .and(path("/email"))
            .and(header("Content-Type", "application/json"))
            .and(SendEmailBodyMatcher)
            .respond_with(wiremock::ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // act
        let result = email_client.send_email(email(), &subject(), &content(), &content()).await;

        // assert
        assert_ok!(result);
    }

    #[tokio::test]
    async fn send_email_fails_when_server_returns_500() {
        // arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(wiremock::ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // act
        let outcome = email_client.send_email(email(), &subject(), &content(), &content()).await;

        // assert
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_server_takes_too_long() {
        // arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let response = wiremock::ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));

        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // act
        let outcome = email_client.send_email(email(), &subject(), &content(), &content()).await;

        // assert
        assert_err!(outcome);
    }
}