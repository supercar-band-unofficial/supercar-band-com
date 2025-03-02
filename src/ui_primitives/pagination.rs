use std::cmp;
use askama::Template;
use crate::router::context::{ RouteContext, QueryHashMap };

const PAGE_RANGE_COUNT: u32 = 5;

#[derive(Template)]
#[template(path = "ui_primitives/pagination.html")]
pub struct PaginationTemplate<'a, Ctx>
where &'a Ctx: RouteContext {
    pub context: &'a Ctx,
    pub current_page: u32,
    pub page_count: u32,
    pub page_query_name: &'a str,
    pub hx_target: &'a str,
}
impl<'a, Ctx> PaginationTemplate<'a, Ctx>
where &'a Ctx: RouteContext {
    fn get_page_range(&self) -> Vec<u32> {
        let start = cmp::max(1, self.current_page.saturating_sub(PAGE_RANGE_COUNT / 2));
        let end = cmp::min(self.page_count + 0, start - 1 + PAGE_RANGE_COUNT);
        (start..=end).collect()
    }

    fn create_page_href(&self, page_number: &u32) -> String {
        let mut query = self.context.route_query().clone();
        query.insert(self.page_query_name.to_string(), page_number.to_string());
        format!("?{}#{}", &query.to_query_string(), self.hx_target)
    }

    fn create_first_page_href(&self) -> String {
        self.create_page_href(&(1 as u32))
    }
}
