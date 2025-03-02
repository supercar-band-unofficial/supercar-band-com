use std::error::Error;
use askama::Template;
use crate::ui_modules::sign_in::{ SignInParams, SignInTemplate as SignInModuleTemplate };
use crate::router::routes::sign_in::SignInPageContext;

#[derive(Template)]
#[template(path = "ui_pages/sign_in.html")]
pub struct SignInTemplate<'a> {
    active_page: &'a str,
    sign_in: SignInModuleTemplate<'a, SignInPageContext>,
}
impl<'a> SignInTemplate<'a> {
    pub async fn new(context: &'a SignInPageContext) -> Result<SignInTemplate<'a>, Box<dyn Error>> {
        let active_page: &str = "";
        let sign_in = SignInModuleTemplate::new(SignInParams {
            context,
            status: context.params.status,
            entered_username: &context.params.entered_username,
        }).await?;
        Ok(SignInTemplate { active_page, sign_in })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/sign_in.html", block = "sign_in")]
pub struct SignInContentTemplate<'a> {
    sign_in: SignInModuleTemplate<'a, SignInPageContext>,
}
impl<'a> SignInContentTemplate<'a> {
    pub async fn new(context: &'a SignInPageContext) -> Result<SignInContentTemplate<'a>, Box<dyn Error>> {
        let sign_in = SignInModuleTemplate::new(SignInParams {
            context,
            status: context.params.status,
            entered_username: &context.params.entered_username,
        }).await?;
        Ok(SignInContentTemplate { sign_in })
    }
}
