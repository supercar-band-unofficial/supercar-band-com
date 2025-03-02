use std::error::Error;
use std::marker::PhantomData;
use askama::Template;
use crate::ui_modules::sidebar::{ SidebarParams, SidebarTemplate };
use crate::router::routes::terms_of_service::TermsOfServicePageContext;

#[derive(Template)]
#[template(path = "ui_pages/terms_of_service.html")]
pub struct TermsOfServiceTemplate<'a> {
    active_page: &'a str,
    sidebar: SidebarTemplate<'a, TermsOfServicePageContext>,
}
impl<'a> TermsOfServiceTemplate<'a> {
    pub async fn new(context: &'a TermsOfServicePageContext) -> Result<TermsOfServiceTemplate<'a>, Box<dyn Error>> {
        let active_page: &str = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;
        Ok(TermsOfServiceTemplate { active_page, sidebar })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/terms_of_service.html", block = "page_content")]
pub struct TermsOfServiceContentTemplate<'a> {
    phantom: PhantomData<&'a ()>,
}
impl<'a> TermsOfServiceContentTemplate<'a> {
    pub async fn new(_context: &'a TermsOfServicePageContext) -> Result<TermsOfServiceContentTemplate<'a>, Box<dyn Error>> {
        Ok(TermsOfServiceContentTemplate { phantom: PhantomData })
    }
}
