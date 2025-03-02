use std::error::Error;
use std::marker::PhantomData;
use askama::Template;
use urlencoding::encode;

use crate::database::{ self };
use crate::router::context::{ RouteContext, UserContext };
use crate::util::user::create_user_profile_href;

pub struct AccountSummaryParams<'a, Ctx: 'a>
where &'a Ctx: RouteContext + UserContext {
    pub context: &'a Ctx,
}

#[derive(Template)]
#[template(path = "ui_modules/account_summary.html")]
pub struct AccountSummaryTemplate<'a, Ctx: 'a>
where &'a Ctx: RouteContext + UserContext {
    phantom: PhantomData<&'a Ctx>,
    first_name: String,
    username: String,
    profile_picture_filename: String,
    redirect_url_encoded: String,
}
impl<'a, Ctx> AccountSummaryTemplate<'a, Ctx>
where &'a Ctx: RouteContext + UserContext {
    pub async fn new(
        params: AccountSummaryParams<'a, Ctx>,
    ) -> Result<AccountSummaryTemplate<'a, Ctx>, Box<dyn Error>> {
        let AccountSummaryParams { context } = params;

        let username = context.username().to_string();
        let first_name = if let Some(user) = context.user() {
            user.first_name.to_string()
        } else {
            String::from("Guest")
        };

        let current_url = context.route_original_uri().to_string();
        let redirect_url = context.route_query()
            .get("redirect-to")
            .unwrap_or(&current_url);
        
        let user = database::get_user_by_username(&username).await?;

        Ok(AccountSummaryTemplate {
            phantom: PhantomData,
            first_name,
            username,
            profile_picture_filename: user.profile_picture_filename,
            redirect_url_encoded: encode(redirect_url).to_string(),
        })
    }
}
