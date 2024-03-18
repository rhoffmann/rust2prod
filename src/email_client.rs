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
struct SendEmailRequest {
    from: String,
    to: String,
    subject: String,
    text: String,
    html: String,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail, authorization_token: Secret<String>) -> Self {
        let base_url = reqwest::Url::parse(&base_url).expect("Invalid base URL");
        Self {
            sender,
            http_client: reqwest::Client::new(),
            base_url,
            authorization_token,
        }
    }
    pub async fn send_email(&self, to: SubscriberEmail, subject: &str, html_content: &str, text_content: &str) -> Result<(), reqwest::Error> {
        let url = self.base_url.join("email").unwrap();

        let request_body = SendEmailRequest {
            from: self.sender.as_ref().to_owned(),
            to: to.as_ref().to_owned(),
            subject: subject.to_owned(),
            text: text_content.to_owned(),
            html: html_content.to_owned(),
        };

        self.http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.authorization_token.expose_secret()))
            .json(&request_body)
            .send()
            .await?;

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Sentence, Paragraph};
    use secrecy::Secret;
    use wiremock::{Mock, MockServer, Request};
    use wiremock::matchers::{header, header_exists, method, path};
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;

    #[tokio::test]
    async fn send_email_sends_expected_request() {
        // arrange
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender, Secret::new(Faker.fake()));

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

        Mock::given(header_exists("Authorization"))
            .and(method("POST"))
            .and(path("/email"))
            .and(header("Content-Type", "application/json"))
            .and(SendEmailBodyMatcher)
            .respond_with(wiremock::ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        // act

        let _ = email_client.send_email(subscriber_email, &subject, &content, &content).await;

        // assert
        // mock exceptions are checked when dropped
    }
}