use askama::Template;
use chrono::NaiveDateTime;

use crate::database::{ CommentSectionName };
use crate::util::format::make_content_links;
use crate::util::user::{ create_user_profile_href, is_guest_user };

#[derive(Template)]
#[template(path = "ui_primitives/comment.html")]
pub struct CommentTemplate<'a> {
    pub username: &'a str,
    pub profile_picture_filename: &'a str,
    pub comment: &'a str,
    pub post_time: &'a NaiveDateTime,
    pub replies: Vec<CommentTemplate<'a>>,
    pub section: &'a CommentSectionName,
    pub section_tag_id: i32,
    pub reply_id: i32,
    pub redirect_url_encoded: &'a str,
}
impl<'a> CommentTemplate<'a> {
    fn get_reply_href(&self) -> String {
        format!("/comment-section/{}/{}/{}/?redirect-to={}%23page-comments",
            self.section,
            self.section_tag_id,
            self.reply_id,
            self.redirect_url_encoded,
        )
    }   
}
