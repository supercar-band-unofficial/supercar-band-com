use std::error::Error;
use askama::Template;
use crate::ui_modules::sidebar::{ SidebarParams, SidebarTemplate };
use crate::ui_primitives::tabs::{ TabsTemplate, TabLink };
use crate::router::routes::bio::BioPageContext;

#[derive(Template)]
#[template(path = "ui_pages/bio.html")]
pub struct BioTemplate<'a> {
    active_page: &'a str,
    topic: &'a str,
    tabs: TabsTemplate<'a, BioPageContext>,
    sidebar: SidebarTemplate<'a, BioPageContext>,
}
impl<'a> BioTemplate<'a> {
    pub async fn new(context: &'a BioPageContext) -> Result<BioTemplate<'a>, Box<dyn Error>> {
        let active_page: &str = "bio";
        let tabs = create_article_tabs(context);
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;
        Ok(BioTemplate { active_page, tabs, topic: &context.params.topic, sidebar })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/bio.html", block = "page_content")]
pub struct BioContentTemplate<'a> {
    tabs: TabsTemplate<'a, BioPageContext>,
    topic: &'a str,
}
impl<'a> BioContentTemplate<'a> {
    pub async fn new(context: &'a BioPageContext) -> Result<BioContentTemplate<'a>, Box<dyn Error>> {
        let tabs = create_article_tabs(context);
        Ok(BioContentTemplate { tabs, topic: &context.params.topic })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/bio.html", block = "tab_container")]
pub struct BioTabContainerTemplate<'a> {
    topic: &'a str,
}
impl<'a> BioTabContainerTemplate<'a> {
    pub async fn new(context: &'a BioPageContext) -> Result<BioTabContainerTemplate<'a>, Box<dyn Error>> {
        Ok(BioTabContainerTemplate { topic: &context.params.topic })
    }
}

fn create_article_tabs<'a>(context: &'a BioPageContext) -> TabsTemplate<'a, BioPageContext> {
    TabsTemplate {
        context,
        aria_label: "Select a Topic",
        links: vec![
            TabLink { label: String::from("Band Bio"), section_name: String::from("band") },
            TabLink { label: String::from("Nakamura Kōji"), section_name: String::from("nakamura-koji") },
            TabLink { label: String::from("Furukawa Miki"), section_name: String::from("furukawa-miki") },
            TabLink { label: String::from("Ishiwatari Junji"), section_name: String::from("ishiwatari-junji") },
            TabLink { label: String::from("Tazawa Kōdai"), section_name: String::from("tazawa-kodai") },
        ],
        current_section_name: &context.params.topic,
        section_query_name: "topic",
        hx_target: "#main-article",
    }
}
