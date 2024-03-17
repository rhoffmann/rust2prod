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
    use wiremock::{Mock, MockServer};
    use wiremock::matchers::any;
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // arrange
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender, Secret::new(Faker.fake()));

        Mock::given(any())
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
    }
}