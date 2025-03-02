use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

#[derive(Template)]
#[template(path = "ui_modules/members_edit_bar.html")]
pub struct MembersEditBarTemplate<'a> {
    phantom: PhantomData<&'a str>,
}
impl<'a> MembersEditBarTemplate<'a> {
    pub async fn new() -> Result<MembersEditBarTemplate<'a>, Box<dyn Error>> {
        
        Ok(MembersEditBarTemplate {
            phantom: PhantomData,
        })
    }
}
