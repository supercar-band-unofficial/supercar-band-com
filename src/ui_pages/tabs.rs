use std::error::Error;
use askama::Template;

use crate::database::{ self, Band, CommentSectionName, SongTabType };
use crate::ui_modules::comment_section::{ CommentSectionParams, CommentSectionTemplate };
use crate::ui_modules::sidebar::{ SidebarParams, SidebarTemplate };
use crate::ui_modules::tabs_display::{ TabsDisplayTemplate, TabsDisplayParams };
use crate::ui_modules::tabs_edit_bar::{ TabsEditBarTemplate, TabsEditBarParams };
use crate::ui_modules::tabs_song_detail::{ TabsSongDetailTemplate, TabsSongDetailParams };
use crate::ui_modules::tabs_song_list::{ TabsSongListTemplate, TabsSongListParams };
use crate::router::routes::tabs::{ TabsPageContext };
use crate::util::format;

struct TabsTemplateCommon<'a> {
    bands: Vec<Band>,
    band_name: String,
    seo_title: String,
    comment_section: Option<CommentSectionTemplate<'a, TabsPageContext>>,
    tabs_display: Option<TabsDisplayTemplate<'a>>,
    tabs_edit_bar: TabsEditBarTemplate<'a, TabsPageContext>,
    tabs_song_detail: Option<TabsSongDetailTemplate<'a>>,
    tabs_song_list: Option<TabsSongListTemplate<'a>>,
}

#[derive(Template)]
#[template(path = "ui_pages/tabs.html")]
pub struct TabsTemplate<'a> {
    active_page: &'a str,
    content: TabsTemplateCommon<'a>,
    needs_title_update: bool,
    sidebar: SidebarTemplate<'a, TabsPageContext>,
}
impl<'a> TabsTemplate<'a> {
    pub async fn new(context: &'a TabsPageContext) -> Result<TabsTemplate<'a>, Box<dyn Error>> {
        let active_page = "tabs";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;
        let content = create_common_params(context).await?;
        Ok(TabsTemplate {
            active_page, content, sidebar, needs_title_update: false,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/tabs.html", block = "page_content")]
pub struct TabsContentTemplate<'a> {
    content: TabsTemplateCommon<'a>,
    needs_title_update: bool,
}
impl<'a> TabsContentTemplate<'a> {
    pub async fn new(context: &'a TabsPageContext) -> Result<TabsContentTemplate<'a>, Box<dyn Error>> {
        let content = create_common_params(context).await?;
        Ok(TabsContentTemplate {
            content, needs_title_update: true,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/tabs.html", block = "page_comments")]
pub struct TabsCommentsTemplate<'a> {
    content: TabsTemplateCommon<'a>,
}
impl<'a> TabsCommentsTemplate<'a> {
    pub async fn new(context: &'a TabsPageContext) -> Result<TabsCommentsTemplate<'a>, Box<dyn Error>> {
        let content = create_common_params(context).await?;
        Ok(TabsCommentsTemplate {
            content,
        })
    }
}

async fn create_common_params<'a>(context: &'a TabsPageContext) -> Result<TabsTemplateCommon<'a>, Box<dyn Error>> {
    let bands = database::get_all_bands().await?;

    let mut band_id: i32 = 0;
    let mut band_name = String::from("Unknown");
    let mut band_slug = String::from("");
    let seo_title: String;
    let mut comment_section = None;
    let mut tabs_display = None;
    let mut tabs_song_detail = None;
    let mut tabs_song_list = None;

    match bands.iter().find(|&band| band.band_slug == context.params.band) {
        Some(band) => {
            band_id = band.id;
            band_slug = band.band_slug.clone();
            band_name = band.band_name.clone();
        },
        _ => {},
    };

    if !context.params.contributor.is_empty() {
        tabs_display = Some(
            TabsDisplayTemplate::new(
                TabsDisplayParams {
                    band_id,
                    band_slug: band_slug.clone(),
                    band_name: band_name.clone(),
                    song_slug: context.params.song.clone(),
                    tab_type: context.params.tab_type.clone(),
                    contributor: context.params.contributor.clone(),
                }
            ).await?
        );

        comment_section = Some(
            CommentSectionTemplate::<TabsPageContext>::new(
                CommentSectionParams {
                    context,
                    section: &CommentSectionName::Tabs,
                    section_tag_id: Some(tabs_display.as_ref().unwrap().tab.id),
                    page_number: context.params.comments_page,
                }
            ).await?
        );

        let tab_type = format::to_snake_case(&context.params.tab_type).parse::<SongTabType>().unwrap_or_else(|_| SongTabType::Unknown);
        seo_title = format!("{} Tabs for 「{}」 by {}", tab_type.as_display(), &tabs_display.as_ref().unwrap().song_name, &band_name);
    } else if !context.params.song.is_empty() {
        tabs_song_detail = Some(
            TabsSongDetailTemplate::new(
                TabsSongDetailParams {
                    band_id,
                    band_slug: band_slug.clone(),
                    band_name: band_name.clone(),
                    song_slug: context.params.song.clone(),
                    is_signed_in: context.user.is_some(),
                }
            ).await?
        );
        seo_title = format!("Guitar Tabs for 「{}」 by {}", &tabs_song_detail.as_ref().unwrap().song_name, &band_name);
    } else {
        tabs_song_list = Some(
            TabsSongListTemplate::new(
                TabsSongListParams {
                    band_id,
                }
            ).await?
        );
        seo_title = format!("All Guitar Tabs for {}", &band_name);
    }

    let mut song_slug = None;
    let mut song_name = None;
    if let Some(tabs_song_detail) = &tabs_song_detail {
        song_slug = Some(tabs_song_detail.song_slug.clone());
        song_name = Some(tabs_song_detail.song_name.clone());
    } else if let Some(tabs_display) = &tabs_display {
        song_slug = Some(context.params.song.clone());
        song_name = Some(tabs_display.song_name.clone());
    }
    let tabs_edit_bar = TabsEditBarTemplate::new(
        TabsEditBarParams {
            context,
            band_slug: band_slug,
            contributor: if tabs_display.is_some() { Some(context.params.contributor.clone()) } else { None },
            song_name,
            song_slug,
            tab_type: if tabs_display.is_some() { Some(context.params.tab_type.clone()) } else { None },
        }
    ).await?;

    Ok(
        TabsTemplateCommon {
            bands,
            band_name,
            seo_title,
            comment_section,
            tabs_display,
            tabs_edit_bar,
            tabs_song_detail,
            tabs_song_list,
        }
    )
}
