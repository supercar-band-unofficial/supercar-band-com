use askama::Template;
use crate::router::context::{ RouteContext, QueryHashMap };

pub struct TabLink {
    pub label: String,
    pub section_name: String,
}

#[derive(Template)]
#[template(path = "ui_primitives/tabs.html")]
pub struct TabsTemplate<'a, Ctx>
where &'a Ctx: RouteContext {
    pub context: &'a Ctx,
    pub links: Vec<TabLink>,
    pub current_section_name: &'a str,
    pub section_query_name: &'a str,
    pub hx_target: &'a str,
    pub aria_label: &'a str,
}
impl<'a, Ctx> TabsTemplate<'a, Ctx>
where &'a Ctx: RouteContext {
    fn create_tab_id(&self, section_name: &str) -> String {
        format!("tab-{}-{}", self.section_query_name, section_name)
    }

    fn create_tag_href(&self, section_name: &str) -> String {
        let mut query = self.context.route_query().clone();
        query.insert(self.section_query_name.to_string(), section_name.to_string());
        format!("?{}", &query.to_query_string())
    }
}
