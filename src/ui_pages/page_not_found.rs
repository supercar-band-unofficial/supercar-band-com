use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::router::routes::page_not_found::{ PageNotFoundContext };
use crate::ui_modules::sidebar::{ SidebarParams, SidebarTemplate };

#[derive(Template)]
#[template(path = "ui_pages/page_not_found.html")]
pub struct PageNotFoundTemplate<'a> {
    active_page: &'a str,
    sidebar: SidebarTemplate<'a, PageNotFoundContext>,
}
impl<'a> PageNotFoundTemplate<'a> {
    pub async fn new(context: &'a PageNotFoundContext) -> Result<PageNotFoundTemplate<'a>, Box<dyn Error>> {
        let active_page: &str = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;
        Ok(PageNotFoundTemplate { active_page, sidebar })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/page_not_found.html", block = "page_content")]
pub struct PageNotFoundContentTemplate<'a> {
    phantom: PhantomData<&'a ()>,
}
impl<'a> PageNotFoundContentTemplate<'a> {
    pub async fn new(_context: &'a PageNotFoundContext) -> Result<PageNotFoundContentTemplate<'a>, Box<dyn Error>> {
        Ok(PageNotFoundContentTemplate { phantom: PhantomData })
    }
}
