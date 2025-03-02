use std::error::Error;
use std::marker::PhantomData;
use askama::Template;
use crate::ui_modules::sidebar::{ SidebarParams, SidebarTemplate };
use crate::router::routes::privacy_policy::PrivacyPolicyPageContext;

#[derive(Template)]
#[template(path = "ui_pages/privacy_policy.html")]
pub struct PrivacyPolicyTemplate<'a> {
    active_page: &'a str,
    sidebar: SidebarTemplate<'a, PrivacyPolicyPageContext>,
}
impl<'a> PrivacyPolicyTemplate<'a> {
    pub async fn new(context: &'a PrivacyPolicyPageContext) -> Result<PrivacyPolicyTemplate<'a>, Box<dyn Error>> {
        let active_page: &str = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;
        Ok(PrivacyPolicyTemplate { active_page, sidebar })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/privacy_policy.html", block = "page_content")]
pub struct PrivacyPolicyContentTemplate<'a> {
    phantom: PhantomData<&'a ()>,
}
impl<'a> PrivacyPolicyContentTemplate<'a> {
    pub async fn new(_context: &'a PrivacyPolicyPageContext) -> Result<PrivacyPolicyContentTemplate<'a>, Box<dyn Error>> {
        Ok(PrivacyPolicyContentTemplate { phantom: PhantomData })
    }
}
