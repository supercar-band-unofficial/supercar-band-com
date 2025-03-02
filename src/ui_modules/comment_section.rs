use std::error::Error;
use askama::Template;
use urlencoding::encode;

use crate::database;
use crate::database::{ CommentWithReplies, CommentSectionName };
use crate::router::context::{ RouteContext, UserContext };
use crate::ui_primitives::comment_group::CommentGroupTemplate;
use crate::ui_primitives::comment::CommentTemplate;
use crate::ui_primitives::pagination::PaginationTemplate;
use crate::util::user::{ create_user_profile_href, is_guest_user };

const COMMENTS_PER_PAGE: u32 = 10;

pub struct CommentSectionParams<'a, Ctx>
where &'a Ctx: RouteContext + UserContext {
    pub context: &'a Ctx,
    pub section: &'a CommentSectionName,
    pub section_tag_id: Option<i32>,
    pub page_number: u32,
}

#[derive(Template)]
#[template(path = "ui_modules/comment_section.html")]
pub struct CommentSectionTemplate<'a, Ctx>
where &'a Ctx: RouteContext + UserContext {
    title: &'a str,
    username: String,
    comments_with_replies: Vec<CommentWithReplies>,
    pagination: PaginationTemplate<'a, Ctx>,
    profile_picture_filename: String,
    section: &'a CommentSectionName,
    section_tag_id: Option<i32>,
    redirect_url_encoded: String,
}
impl<'a, Ctx> CommentSectionTemplate<'a, Ctx>
where &'a Ctx: RouteContext + UserContext {
    pub async fn new(
        params: CommentSectionParams<'a, Ctx>,
    ) -> Result<CommentSectionTemplate<'a, Ctx>, Box<dyn Error>> {
        let CommentSectionParams { context, section, section_tag_id, page_number } = params;
        let title = get_title(section);

        let comment_count = database::get_comments_count(section, section_tag_id, -1).await?;
        let comments_with_replies = database::get_comments_in_range_with_replies(
            (page_number - 1) * COMMENTS_PER_PAGE, COMMENTS_PER_PAGE, section, section_tag_id, -1
        ).await?;

        let profile_picture_filename = if let Some(user) = context.user() {
            let user = database::get_user_by_username(&user.username).await?;
            user.profile_picture_filename
        } else {
            String::from("Guest.jpeg")
        };

        let redirect_url_encoded = encode(
            &context.route_original_uri().to_string()
                .split('#').collect::<Vec<&str>>().first().unwrap()
        ).to_string();

        Ok(CommentSectionTemplate {
            title,
            username: context.username().to_string(),
            profile_picture_filename,
            comments_with_replies,
            section,
            section_tag_id,
            redirect_url_encoded,
            pagination: PaginationTemplate::<Ctx> {
                context,
                current_page: page_number,
                page_count: (comment_count / COMMENTS_PER_PAGE) + (if comment_count % COMMENTS_PER_PAGE > 0 { 1 } else { 0 }),
                page_query_name: "comments-page",
                hx_target: "page-comments",
            },
        })
    }

    fn get_submit_action(&self) -> String {
        format!("/comment-section/{}/{}/?redirect-to={}%23page-comments",
            self.section,
            self.section_tag_id.unwrap_or_else(|| -1),
            self.redirect_url_encoded,
        )
    }

}

fn get_comment_group<'a>(
    comments_with_replies: &'a Vec<CommentWithReplies>,
    section: &'a CommentSectionName,
    section_tag_id: &'a Option<i32>,
    redirect_url_encoded: &'a str,
) -> CommentGroupTemplate<'a> {
    CommentGroupTemplate {
        comments: get_comments(
            comments_with_replies,
            section,
            section_tag_id,
            redirect_url_encoded,
        ),
    }
}

fn get_comments<'a>(
    comments_with_replies: &'a Vec<CommentWithReplies>,
    section: &'a CommentSectionName,
    section_tag_id: &'a Option<i32>,
    redirect_url_encoded: &'a str,
) -> Vec<CommentTemplate<'a>> {
    let mut comment_templates: Vec<CommentTemplate<'a>> = Vec::with_capacity(comments_with_replies.len());
    for comment_with_replies in comments_with_replies {
        comment_templates.push(
            CommentTemplate {
                username: &comment_with_replies.comment.username,
                profile_picture_filename: &comment_with_replies.comment.profile_picture_filename,
                comment: &comment_with_replies.comment.comment,
                post_time: &comment_with_replies.comment.post_time,
                replies: get_comments(&comment_with_replies.replies, section, section_tag_id, redirect_url_encoded),
                section: section,
                section_tag_id: section_tag_id.unwrap_or_else(|| -1),
                reply_id: comment_with_replies.comment.id,
                redirect_url_encoded: redirect_url_encoded,
            }
        );
    }
    comment_templates
}

fn get_title<'a>(section: &CommentSectionName) -> &'a str {
    match section {
        CommentSectionName::Home => "Site Comments",
        CommentSectionName::Lyrics => "Song Comments",
        CommentSectionName::Tabs => "Tabs Comments",
        CommentSectionName::Photos => "Photo Comments",
        CommentSectionName::Videos => "Video Comments",
        CommentSectionName::Members => "Profile Comments",
        _ => "",
    }
}
