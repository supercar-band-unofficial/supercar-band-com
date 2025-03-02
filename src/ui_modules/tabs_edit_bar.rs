use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ UserPermission };
use crate::router::routes::tabs::{ TabsPageContext };

pub struct TabsEditBarParams<'a, TabsPageContext> {
    pub context: &'a TabsPageContext,
    pub band_slug: String,
    pub contributor: Option<String>,
    pub song_name: Option<String>,
    pub song_slug: Option<String>,
    pub tab_type: Option<String>,
}

#[derive(Template)]
#[template(path = "ui_modules/tabs_edit_bar.html")]
pub struct TabsEditBarTemplate<'a, TabsPageContext> {
    phantom: PhantomData<&'a TabsPageContext>,
    band_slug: String,
    contributor: String,
    can_create_tabs: bool,
    can_edit_tabs: bool,
    can_delete_tabs: bool,
    song_name: String,
    song_slug: String,
    tab_type: String,
}
impl<'a> TabsEditBarTemplate<'a, TabsPageContext> {
    pub async fn new(
        params: TabsEditBarParams<'a, TabsPageContext>,
    ) -> Result<TabsEditBarTemplate<'a, TabsPageContext>, Box<dyn Error>> {
        let TabsEditBarParams {
            context, band_slug, contributor, song_name, song_slug, tab_type,
        } = params;

        let mut can_create_tabs = false;
        let mut can_edit_tabs = false;
        let mut can_delete_tabs = false;

        if let Some(user) = &context.user {
            can_create_tabs = user.permissions.contains(&UserPermission::CreateOwnTabs);
            if song_slug.is_some() && tab_type.is_some() {
                if let Some(contributor) = &contributor {
                    can_edit_tabs =
                        user.permissions.contains(&UserPermission::EditTabs) || (
                            user.permissions.contains(&UserPermission::EditOwnTabs) &&
                            contributor == &user.username
                        );
                    can_delete_tabs =
                        user.permissions.contains(&UserPermission::DeleteTabs) || (
                            user.permissions.contains(&UserPermission::DeleteOwnTabs) &&
                            contributor == &user.username
                        );
                }
            }
        }

        Ok(TabsEditBarTemplate {
            phantom: PhantomData,
            band_slug,
            contributor: contributor.unwrap_or_else(|| String::from("")),
            can_create_tabs,
            can_edit_tabs,
            can_delete_tabs,
            song_name: song_name.unwrap_or_else(|| String::from("")),
            song_slug: song_slug.unwrap_or_else(|| String::from("")),
            tab_type: tab_type.unwrap_or_else(|| String::from("")),
        })
    }
}
