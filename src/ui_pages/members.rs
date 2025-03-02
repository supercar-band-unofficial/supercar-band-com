use std::error::Error;
use askama::Template;

use crate::database::{ CommentSectionName, UserPreference };
use crate::ui_modules::comment_section::{ CommentSectionParams, CommentSectionTemplate };
use crate::ui_modules::members_edit_bar::{ MembersEditBarTemplate };
use crate::ui_modules::member_list::{ MemberListTemplate, MemberListParams };
use crate::ui_modules::member_profile::{ MemberProfileTemplate, MemberProfileParams };
use crate::ui_modules::sidebar::{ SidebarParams, SidebarTemplate };
use crate::router::routes::members::{ MembersPageContext };

struct MembersTemplateCommon<'a> {
    seo_title: String,
    comment_section: Option<CommentSectionTemplate<'a, MembersPageContext>>,
    members_edit_bar: Option<MembersEditBarTemplate<'a>>,
    member_list: Option<MemberListTemplate<'a, MembersPageContext>>,
    member_profile: Option<MemberProfileTemplate<'a, MembersPageContext>>,
    username: &'a str,
}

#[derive(Template)]
#[template(path = "ui_pages/members.html")]
pub struct MembersTemplate<'a> {
    active_page: &'a str,
    content: MembersTemplateCommon<'a>,
    needs_title_update: bool,
    sidebar: SidebarTemplate<'a, MembersPageContext>,
}
impl<'a> MembersTemplate<'a> {
    pub async fn new(context: &'a MembersPageContext) -> Result<MembersTemplate<'a>, Box<dyn Error>> {
        let active_page = "members";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;
        let content = create_common_params(context).await?;
        Ok(MembersTemplate {
            active_page, content, sidebar, needs_title_update: false,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/members.html", block = "page_content")]
pub struct MembersContentTemplate<'a> {
    content: MembersTemplateCommon<'a>,
    needs_title_update: bool,
}
impl<'a> MembersContentTemplate<'a> {
    pub async fn new(context: &'a MembersPageContext) -> Result<MembersContentTemplate<'a>, Box<dyn Error>> {
        let content = create_common_params(context).await?;
        Ok(MembersContentTemplate {
            content, needs_title_update: true,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/members.html", block = "page_comments")]
pub struct MembersCommentsTemplate<'a> {
    content: MembersTemplateCommon<'a>,
}
impl<'a> MembersCommentsTemplate<'a> {
    pub async fn new(context: &'a MembersPageContext) -> Result<MembersCommentsTemplate<'a>, Box<dyn Error>> {
        let content = create_common_params(context).await?;
        Ok(MembersCommentsTemplate {
            content,
        })
    }
}

async fn create_common_params<'a>(context: &'a MembersPageContext) -> Result<MembersTemplateCommon<'a>, Box<dyn Error>> {
    let seo_title: String;
    let mut comment_section = None;
    let mut member_list = None;
    let mut member_profile = None;
    let mut members_edit_bar = None;

    if !context.params.username.is_empty() {
        
        member_profile = Some(
            MemberProfileTemplate::new(
                MemberProfileParams {
                    username: &context.params.username,
                    context,
                }
            ).await?
        );
        let user = &member_profile.as_ref().unwrap().user;

        if user.preferences.contains(&UserPreference::AllowProfileComments) {
            comment_section = Some(
                CommentSectionTemplate::<MembersPageContext>::new(
                    CommentSectionParams {
                        context,
                        section: &CommentSectionName::Members,
                        section_tag_id: Some(user.id),
                        page_number: context.params.comments_page,
                    }
                ).await?
            );
        }

        if let Some(logged_in_user) = &context.user {
            if user.username == logged_in_user.username {
                members_edit_bar = Some(
                    MembersEditBarTemplate::new().await?
                );
            }
        }

        seo_title = format!("{}'s Profile", context.params.username);
    } else {
        member_list = Some(
            MemberListTemplate::<MembersPageContext>::new(
                MemberListParams {
                    page_number: context.params.members_page,
                    context,
                }
            ).await?
        );
        
        seo_title = format!("Members");
    }

    Ok(
        MembersTemplateCommon {
            seo_title,
            comment_section,
            members_edit_bar,
            member_list,
            member_profile,
            username: &context.params.username,
        }
    )
}
