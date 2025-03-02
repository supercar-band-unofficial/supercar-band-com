use std::error::Error;
use std::marker::PhantomData;
use askama::Template;
use crate::database;

#[derive(Template)]
#[template(path = "ui_modules/site_activity.html")]
pub struct SiteActivityTemplate<'a> {
    phantom: PhantomData<&'a ()>,
    site_events: Vec<crate::database::SiteEvent>,
}
impl<'a> SiteActivityTemplate<'a> {
    pub async fn new() -> Result<SiteActivityTemplate<'a>, Box<dyn Error>> {
        let site_events = database::get_recent_site_events().await?;
        Ok(SiteActivityTemplate {
            phantom: PhantomData,
            site_events: site_events
        })
    }
}
