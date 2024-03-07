use lettre::Transport;

pub struct EmailPayload {
    pub subject: Option<String>,
    pub content: String,
}

// https://support.microsoft.com/en-us/office/pop-imap-and-smtp-settings-for-outlook-com-d088b986-291d-42b8-9564-9c414e2aa040
struct Outlook {
    username: String,
    password: String,
}

impl Outlook {
    fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.to_owned(),
            password: password.to_owned(),
        }
    }
    fn get_mailer(&self) -> lettre::SmtpTransport {
        let credentials = lettre::transport::smtp::authentication::Credentials::new(
            self.username.to_owned(),
            self.password.to_owned(),
        );
        lettre::transport::smtp::SmtpTransport::starttls_relay("smtp-mail.outlook.com")
            .unwrap()
            .credentials(credentials)
            .build()
    }
}

pub async fn send_email_to_myself(
    payload: EmailPayload,
) -> Result<lettre::transport::smtp::response::Response, lettre::transport::smtp::Error> {
    let outlook_account = Outlook::new("<账号>", "<密码>");
    let email = lettre::Message::builder()
        .from(
            format!("RS Bot <{}>", outlook_account.username)
                .parse()
                .unwrap(),
        )
        .to("You <用来接收消息的邮件地址>".parse().unwrap())
        .subject(payload.subject.unwrap_or("新邮件".to_string()))
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(payload.content)
        .unwrap();
    outlook_account.get_mailer().send(&email)
}
