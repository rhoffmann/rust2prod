use std::env;
use actix_web::{HttpRequest, HttpResponse, Responder};
use resend_email::client::ResendClient;
use resend_email::mail::MailText;

pub async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    let subject = format!("Greetings {}!", &name);
    let body = format!("Hello {}!", &name);

    let resend_token = match env::var("RESEND_API_TOKEN") {
        Ok(token) => token,
        Err(_) => return HttpResponse::Ok().body(body)
    };

    let mail = MailText {
        from: "richard@richnet.guru",
        to: vec!["rjh.hoffmann@googlemail.com"],
        subject: subject.as_str(),
        text: body.as_str(),
        attachments: None
    };

    let client = ResendClient::new(resend_token.as_str());

    let response = match client.send(&mail).await {
        Ok(_) =>  HttpResponse::Ok().body(body),
        Err(_) => HttpResponse::InternalServerError().finish()
    };

    response
}