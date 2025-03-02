use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ self, User, UserGender };
use crate::router::context::{ RouteContext, UserContext };
use crate::ui_primitives::alert::AlertTemplate;
use crate::util::format::make_content_links;

pub struct MemberProfileParams<'a, Ctx>
where &'a Ctx: RouteContext + UserContext {
    pub username: &'a str,
    pub context: &'a Ctx,
}

#[derive(Template)]
#[template(path = "ui_modules/member_profile.html")]
pub struct MemberProfileTemplate<'a, Ctx>
where &'a Ctx: RouteContext + UserContext {
    phantom: PhantomData<&'a Ctx>,
    pub user: User,
    email_alert: Option<AlertTemplate<'a>>,
}
impl<'a, Ctx> MemberProfileTemplate<'a, Ctx>
where &'a Ctx: RouteContext + UserContext {
    pub async fn new(
        params: MemberProfileParams<'a, Ctx>,
    ) -> Result<MemberProfileTemplate<'a, Ctx>, Box<dyn Error>> {
        let MemberProfileParams { username, context } = params;

        let user = database::get_user_by_username(username).await?;
        let mut email_alert = None;

        if context.username() == username && context.route_query().get("first-visit").is_some() {
            email_alert = Some(
                AlertTemplate {
                    variant: "info",
                    message_html: String::from(r#"
                        <p class="mb-2">It is optional to provide your email, but if you do not add one you will not be able to recover your password if you forget it.</p>
                        <a href="/editor/update/profile-info/"><span class="bi bi-pencil-square mr-1"></span>Edit your profile info to update your email.</a>
                    "#),
                }
            )
        }

        Ok(MemberProfileTemplate {
            phantom: PhantomData,
            user,
            email_alert,
        })
    }
}
