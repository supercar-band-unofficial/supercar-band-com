use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ self, QueryOrder, UserSummary };
use crate::router::context::RouteContext;
use crate::ui_primitives::pagination::PaginationTemplate;
use crate::util::user::create_user_profile_href;

const MEMBERS_PER_PAGE: u32 = 42;

pub struct MemberListParams<'a, Ctx>
where &'a Ctx: RouteContext {
    pub page_number: u32,
    pub context: &'a Ctx,
}

#[derive(Template)]
#[template(path = "ui_modules/member_list.html")]
pub struct MemberListTemplate<'a, Ctx>
where &'a Ctx: RouteContext {
    phantom: PhantomData<&'a ()>,
    users: Vec<UserSummary>,
    pagination: Option<PaginationTemplate<'a, Ctx>>,
}
impl<'a, Ctx> MemberListTemplate<'a, Ctx>
where &'a Ctx: RouteContext {
    pub async fn new(
        params: MemberListParams<'a, Ctx>,
    ) -> Result<MemberListTemplate<'a, Ctx>, Box<dyn Error>> {
        let MemberListParams { page_number, context } = params;

        let users_count = database::get_users_count().await?;
        let users = database::get_users_in_range((page_number - 1) * MEMBERS_PER_PAGE, MEMBERS_PER_PAGE, &QueryOrder::Asc).await?;

        let pagination = if users_count > MEMBERS_PER_PAGE {
            Some(PaginationTemplate::<Ctx> {
                context,
                current_page: page_number,
                page_count: (users_count / MEMBERS_PER_PAGE) + (if users_count % MEMBERS_PER_PAGE > 0 { 1 } else { 0 }),
                page_query_name: "members-page",
                hx_target: "main-article",
            })
        } else {
            None
        };

        Ok(MemberListTemplate {
            phantom: PhantomData,
            users,
            pagination,
        })
    }
}
