use askama::Template;
use super::comment::CommentTemplate;

#[derive(Template)]
#[template(path = "ui_primitives/comment_group.html")]
pub struct CommentGroupTemplate<'a> {
    pub comments: Vec<CommentTemplate<'a>>,
}