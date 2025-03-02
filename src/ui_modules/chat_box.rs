use std::error::Error;
use std::marker::PhantomData;
use askama::Template;
use urlencoding::encode;

use crate::database::{ self, Comment, CommentSectionName, QueryOrder };
use crate::router::context::{ RouteContext, UserContext };
use crate::util::format::make_content_links;
use crate::util::user::{ create_user_profile_href, is_guest_user };

const COMMENTS_PER_PAGE: u32 = 30;

pub struct ChatBoxParams<'a, Ctx: 'a>
where &'a Ctx: RouteContext + UserContext {
    pub context: &'a Ctx,
    pub page_number: u32,
}

#[derive(Template)]
#[template(path = "ui_modules/chat_box.html")]
pub struct ChatBoxTemplate<'a, Ctx: 'a>
where &'a Ctx: RouteContext + UserContext {
    phantom: PhantomData<&'a Ctx>,
    comments: Vec<Comment>,
    redirect_url_encoded: String,
}
impl<'a, Ctx> ChatBoxTemplate<'a, Ctx>
where &'a Ctx: RouteContext + UserContext {
    pub async fn new(
        params: ChatBoxParams<'a, Ctx>,
    ) -> Result<ChatBoxTemplate<'a, Ctx>, Box<dyn Error>> {
        let ChatBoxParams { context, page_number } = params;

        let comments = database::get_comments_in_range(
            (page_number - 1) * COMMENTS_PER_PAGE,
            COMMENTS_PER_PAGE, &CommentSectionName::Chatbox, None,
            -1, &QueryOrder::Desc,
        ).await?;

        let redirect_url_encoded = encode(
            &context.route_original_uri().to_string()
                .split('#').collect::<Vec<&str>>().first().unwrap()
        ).to_string();

        Ok(ChatBoxTemplate {
            phantom: PhantomData,
            comments,
            redirect_url_encoded,
        })
    }
}
