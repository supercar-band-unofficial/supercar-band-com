use std::error::Error;
use std::marker::PhantomData;
use askama::Template;
use crate::ui_modules::sidebar::{ SidebarParams, SidebarTemplate };
use crate::router::routes::community_guidelines::CommunityGuidelinesPageContext;

#[derive(Template)]
#[template(path = "ui_pages/community_guidelines.html")]
pub struct CommunityGuidelinesTemplate<'a> {
    active_page: &'a str,
    sidebar: SidebarTemplate<'a, CommunityGuidelinesPageContext>,
}
impl<'a> CommunityGuidelinesTemplate<'a> {
    pub async fn new(context: &'a CommunityGuidelinesPageContext) -> Result<CommunityGuidelinesTemplate<'a>, Box<dyn Error>> {
        let active_page: &str = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;
        Ok(CommunityGuidelinesTemplate { active_page, sidebar })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/community_guidelines.html", block = "page_content")]
pub struct CommunityGuidelinesContentTemplate<'a> {
    phantom: PhantomData<&'a ()>,
}
impl<'a> CommunityGuidelinesContentTemplate<'a> {
    pub async fn new(_context: &'a CommunityGuidelinesPageContext) -> Result<CommunityGuidelinesContentTemplate<'a>, Box<dyn Error>> {
        Ok(CommunityGuidelinesContentTemplate { phantom: PhantomData })
    }
}
