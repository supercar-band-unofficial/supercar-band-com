use askama::Template;

#[derive(Template)]
#[template(path = "ui_primitives/captcha.html")]
pub struct CaptchaTemplate<'a> {
    pub captcha_id: String,
    pub form_id_prefix: &'a str, // e.g. "sign-up"
}
