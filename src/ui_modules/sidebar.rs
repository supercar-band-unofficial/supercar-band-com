use std::error::Error;
use askama::Template;

use crate::router::context::{ RouteContext, UserContext };
use crate::ui_modules::account_summary::{ AccountSummaryParams, AccountSummaryTemplate };
use crate::ui_modules::chat_box::{ ChatBoxParams, ChatBoxTemplate };
use crate::ui_modules::sign_in::{ SignInParams, SignInTemplate };

pub struct SidebarParams<'a, Ctx>
where &'a Ctx: RouteContext + UserContext {
    pub context: &'a Ctx,
}

#[derive(Template)]
#[template(path = "ui_modules/sidebar.html")]
pub struct SidebarTemplate<'a, Ctx>
where &'a Ctx: RouteContext + UserContext {
    account_summary: Option<AccountSummaryTemplate<'a, Ctx>>,
    sign_in: Option<SignInTemplate<'a, Ctx>>,
    chat_box: ChatBoxTemplate<'a, Ctx>,
}
impl<'a, Ctx> SidebarTemplate<'a, Ctx>
where &'a Ctx: RouteContext + UserContext {
    pub async fn new(
        params: SidebarParams<'a, Ctx>,
    ) -> Result<SidebarTemplate<'a, Ctx>, Box<dyn Error>> {
        let SidebarParams { context } = params;

        let mut account_summary = None;
        let mut sign_in = None;
        if context.is_signed_in() {
            account_summary = Some(AccountSummaryTemplate::new(AccountSummaryParams { context }).await?);
        } else {
            sign_in = Some(SignInTemplate::new(SignInParams { context, status: 0, entered_username: "" }).await?);
        }
        let chat_box = ChatBoxTemplate::new(ChatBoxParams { context, page_number: 1 }).await?;

        Ok(SidebarTemplate {
            account_summary,
            sign_in,
            chat_box,
        })
    }
}
