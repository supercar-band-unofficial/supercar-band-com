use std::error::Error;
use std::marker::PhantomData;
use askama::Template;
use urlencoding::encode;

use crate::router::context::RouteContext;
use crate::ui_primitives::alert::AlertTemplate;

pub struct SignInParams<'a, Ctx>
where &'a Ctx: RouteContext {
    pub context: &'a Ctx,
    pub status: u16,
    pub entered_username: &'a str,
}

#[derive(Template)]
#[template(path = "ui_modules/sign_in.html")]
pub struct SignInTemplate<'a, Ctx: 'a>
where &'a Ctx: RouteContext {
    phantom: PhantomData<&'a Ctx>,
    alert: Option<AlertTemplate<'a>>,
    redirect_url_encoded: String,
    entered_username: &'a str,
}
impl<'a, Ctx> SignInTemplate<'a, Ctx>
where &'a Ctx: RouteContext {
    pub async fn new(
        params: SignInParams<'a, Ctx>,
    ) -> Result<SignInTemplate<'a, Ctx>, Box<dyn Error>> {
        let SignInParams { context, status, entered_username } = params;

        let alert = match status {
            401 => Some(AlertTemplate {
                variant: "danger",
                message_html: String::from("The username / password combination is invalid. Please try again."),
            }),
            500 => Some(AlertTemplate {
                variant: "danger",
                message_html: String::from("An internal server error occurred. Please try again later."),
            }),
            _ => None,
        };

        let current_url = context.route_original_uri().to_string();
        let redirect_url = context.route_query()
            .get("redirect-to")
            .unwrap_or(&current_url);

        Ok(SignInTemplate {
            phantom: PhantomData,
            alert,
            redirect_url_encoded: encode(redirect_url).to_string(),
            entered_username,
        })
    }
}
