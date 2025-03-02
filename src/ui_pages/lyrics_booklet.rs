use std::error::Error;
use askama::Template;

use crate::database::{ self };
use crate::ui_modules::sidebar::{ SidebarParams, SidebarTemplate };
use crate::router::routes::lyrics_booklet::{ LyricsBookletPageContext };

struct LyricsBookletTemplateCommon {
    band_slug: String,
    band_name: String,
    album_slug: String,
    album_name: String,
    seo_title: String,
    images: Vec<String>,
}

#[derive(Template)]
#[template(path = "ui_pages/lyrics_booklet.html")]
pub struct LyricsBookletTemplate<'a> {
    active_page: &'a str,
    content: LyricsBookletTemplateCommon,
    needs_title_update: bool,
    sidebar: SidebarTemplate<'a, LyricsBookletPageContext>,
}
impl<'a> LyricsBookletTemplate<'a> {
    pub async fn new(context: &'a LyricsBookletPageContext) -> Result<LyricsBookletTemplate<'a>, Box<dyn Error>> {
        let active_page = "lyrics";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;
        let content = create_common_params(context).await?;
        Ok(LyricsBookletTemplate {
            active_page, content, sidebar, needs_title_update: false,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/lyrics_booklet.html", block = "page_content")]
pub struct LyricsBookletContentTemplate {
    content: LyricsBookletTemplateCommon,
    needs_title_update: bool,
}
impl<'a> LyricsBookletContentTemplate {
    pub async fn new(context: &'a LyricsBookletPageContext) -> Result<LyricsBookletContentTemplate, Box<dyn Error>> {
        let content = create_common_params(context).await?;
        Ok(LyricsBookletContentTemplate {
            content, needs_title_update: true,
        })
    }
}

async fn create_common_params<'a>(context: &'a LyricsBookletPageContext) -> Result<LyricsBookletTemplateCommon, Box<dyn Error>> {
    let band = database::get_band_by_slug(context.params.band.as_str()).await?;
    let album = database::get_album_by_slug_and_band_id(context.params.album.as_str(), band.id).await?;

    let band_name = band.band_name;
    let album_name = album.album_name;

    let images = database::get_lyrics_booklet_images(&context.params.band, &context.params.album).await;

    let seo_title = format!(" for album {} by {}", &band_name, &album_name);

    Ok(
        LyricsBookletTemplateCommon {
            band_slug: context.params.band.clone(),
            album_slug: context.params.album.clone(),
            band_name,
            album_name,
            seo_title,
            images,
        }
    )
}
