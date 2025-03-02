use std::error::Error;
use askama::Template;
use crate::database::CommentSectionName;
use crate::ui_modules::sidebar::{ SidebarParams, SidebarTemplate };
use crate::ui_modules::site_activity::SiteActivityTemplate;
use crate::ui_modules::comment_section::{ CommentSectionParams, CommentSectionTemplate };
use crate::router::routes::home::HomePageContext;

#[derive(Template)]
#[template(path = "ui_pages/home.html")]
pub struct HomeTemplate<'a> {
    active_page: &'a str,
    site_activity: SiteActivityTemplate<'a>,
    comment_section: CommentSectionTemplate<'a, HomePageContext>,
    sidebar: SidebarTemplate<'a, HomePageContext>,
}
impl<'a> HomeTemplate<'a> {
    pub async fn new(context: &'a HomePageContext) -> Result<HomeTemplate<'a>, Box<dyn Error>> {
        let active_page: &str = "home";
        let comment_section = CommentSectionTemplate::<HomePageContext>::new(
            CommentSectionParams {
                context,
                section: &CommentSectionName::Home,
                section_tag_id: None,
                page_number: context.params.comments_page,
            },
        ).await?;
        let site_activity = SiteActivityTemplate::new().await?;
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;
        Ok(HomeTemplate { active_page, comment_section, site_activity, sidebar })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/home.html", block = "page_content")]
pub struct HomeContentTemplate<'a> {
    site_activity: SiteActivityTemplate<'a>,
    comment_section: CommentSectionTemplate<'a, HomePageContext>,
}
impl<'a> HomeContentTemplate<'a> {
    pub async fn new(context: &'a HomePageContext) -> Result<HomeContentTemplate<'a>, Box<dyn Error>> {
        let site_activity = SiteActivityTemplate::new().await?;
        let comment_section = CommentSectionTemplate::<HomePageContext>::new(
            CommentSectionParams {
                context,
                section: &CommentSectionName::Home,
                section_tag_id: None,
                page_number: 1,
            }
        ).await?;
        Ok(HomeContentTemplate { site_activity, comment_section })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/home.html", block = "page_comments")]
pub struct HomeCommentsTemplate<'a> {
    comment_section: CommentSectionTemplate<'a, HomePageContext>,
}
impl<'a> HomeCommentsTemplate<'a> {
    pub async fn new(context: &'a HomePageContext) -> Result<HomeCommentsTemplate<'a>, Box<dyn Error>> {
        let comment_section = CommentSectionTemplate::<HomePageContext>::new(
            CommentSectionParams {
                context,
                section: &CommentSectionName::Home,
                section_tag_id: None,
                page_number: context.params.comments_page,
            }
        ).await?;
        Ok(HomeCommentsTemplate { comment_section })
    }
}
