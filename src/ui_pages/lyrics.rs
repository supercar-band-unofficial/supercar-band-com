use std::error::Error;
use std::io;
use askama::Template;

use crate::database::{ self, Band, CommentSectionName, Lyrics };
use crate::ui_modules::album_detail::{ AlbumDetailParams, AlbumDetailTemplate };
use crate::ui_modules::album_list::{ AlbumListParams, AlbumListTemplate };
use crate::ui_modules::comment_section::{ CommentSectionParams, CommentSectionTemplate };
use crate::ui_modules::lyrics_edit_bar::{ LyricsEditBarTemplate, LyricsEditBarParams };
use crate::ui_modules::lyrics_search_results::{ LyricsSearchResultsTemplate, LyricsSearchResultsParams };
use crate::ui_modules::song_list::{ SongListParams, SongListTemplate };
use crate::ui_modules::recent_translations::{ RecentTranslationsParams, RecentTranslationsTemplate };
use crate::ui_modules::sidebar::{ SidebarParams, SidebarTemplate };
use crate::ui_modules::song_lyrics::{ SongLyricsParams, SongLyricsTemplate };
use crate::router::routes::lyrics::{ LyricsPageContext };

struct LyricsTemplateCommon<'a> {
    album_detail: Option<AlbumDetailTemplate<'a>>,
    band_id: i32,
    band_slug: String,
    band_name: String,
    bands: Vec<Band>,
    comment_section: Option<CommentSectionTemplate<'a, LyricsPageContext>>,
    lyrics_edit_bar: LyricsEditBarTemplate<'a, LyricsPageContext>,
    recent_translations: Option<RecentTranslationsTemplate<'a>>,
    search_results: Option<LyricsSearchResultsTemplate<'a>>,
    seo_title: String,
    song_lyrics: Option<SongLyricsTemplate<'a>>,
}

#[derive(Template)]
#[template(path = "ui_pages/lyrics.html")]
pub struct LyricsTemplate<'a> {
    active_page: &'a str,
    albums: AlbumListTemplate<'a>,
    album_detail: Option<AlbumDetailTemplate<'a>>,
    band_name: String,
    band_slug: String,
    bands: Vec<Band>,
    comment_section: Option<CommentSectionTemplate<'a, LyricsPageContext>>,
    lyrics_edit_bar: LyricsEditBarTemplate<'a, LyricsPageContext>,
    needs_title_update: bool,
    recent_translations: Option<RecentTranslationsTemplate<'a>>,
    search_results: Option<LyricsSearchResultsTemplate<'a>>,
    seo_title: String,
    sidebar: SidebarTemplate<'a, LyricsPageContext>,
    song_list: SongListTemplate<'a>,
    song_lyrics: Option<SongLyricsTemplate<'a>>,
}

impl<'a> LyricsTemplate<'a> {
    pub async fn new(context: &'a LyricsPageContext) -> Result<LyricsTemplate<'a>, Box<dyn Error>> {
        let LyricsTemplateCommon {
            album_detail, band_id, band_name, band_slug, bands, comment_section, lyrics_edit_bar,
            recent_translations, search_results, seo_title, song_lyrics, ..
        } = create_common_params(context).await?;
        let active_page = "lyrics";
        let albums = AlbumListTemplate::new(AlbumListParams { band_id, band_slug: band_slug.clone() }).await?;
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;
        let song_list = SongListTemplate::new(SongListParams { band_id }).await?;
        Ok(LyricsTemplate {
            albums, album_detail, active_page, band_name, band_slug, bands,
            comment_section, lyrics_edit_bar, needs_title_update: false, recent_translations,
            search_results, seo_title, sidebar, song_list, song_lyrics
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/lyrics.html", block = "page_content")]
pub struct LyricsContentTemplate<'a> {
    albums: AlbumListTemplate<'a>,
    album_detail: Option<AlbumDetailTemplate<'a>>,
    band_name: String,
    band_slug: String,
    bands: Vec<Band>,
    comment_section: Option<CommentSectionTemplate<'a, LyricsPageContext>>,
    lyrics_edit_bar: LyricsEditBarTemplate<'a, LyricsPageContext>,
    needs_title_update: bool,
    recent_translations: Option<RecentTranslationsTemplate<'a>>,
    search_results: Option<LyricsSearchResultsTemplate<'a>>,
    seo_title: String,
    song_list: SongListTemplate<'a>,
    song_lyrics: Option<SongLyricsTemplate<'a>>,
}
impl <'a>LyricsContentTemplate<'a> {
    pub async fn new(context: &'a LyricsPageContext) -> Result<LyricsContentTemplate<'a>, Box<dyn Error>> {
        let LyricsTemplateCommon {
            album_detail, band_id, band_name, band_slug, bands, comment_section, lyrics_edit_bar,
            recent_translations, search_results, seo_title, song_lyrics, ..
        } = create_common_params(context).await?;
        let albums = AlbumListTemplate::new(AlbumListParams { band_id, band_slug: band_slug.clone() }).await?;
        let song_list = SongListTemplate::new(SongListParams { band_id }).await?;
        Ok(LyricsContentTemplate {
            albums, album_detail, band_name, band_slug, bands,
            comment_section, lyrics_edit_bar, needs_title_update: true,
            recent_translations, search_results, seo_title, song_list, song_lyrics
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/lyrics.html", block = "page_comments")]
pub struct LyricsCommentsTemplate<'a> {
    comment_section: Option<CommentSectionTemplate<'a, LyricsPageContext>>,
}
impl <'a>LyricsCommentsTemplate<'a> {
    pub async fn new(context: &'a LyricsPageContext) -> Result<LyricsCommentsTemplate<'a>, Box<dyn Error>> {
        let LyricsTemplateCommon {
            comment_section, ..
        } = create_common_params(context).await?;
        Ok(LyricsCommentsTemplate { comment_section })
    }
}

async fn create_common_params<'a>(context: &'a LyricsPageContext) -> Result<LyricsTemplateCommon<'a>, Box<dyn Error>> {
    let bands = database::get_all_bands().await?;

    let mut band_id: i32 = 0;
    let mut band_name = String::from("Unknown");
    let mut band_slug = String::from("");
    let mut seo_title = String::from("");
    let mut album_detail = None;
    let mut comment_section = None;
    let mut contributor = String::from("");
    let mut recent_translations = None;
    let mut search_results = None;
    let mut song_lyrics = None;

    match bands.iter().find(|&band| band.band_slug == context.params.band) {
        Some(band) => {
            band_id = band.id;
            band_slug = band.band_slug.clone();
            band_name = band.band_name.clone();
        },
        _ => {},
    };

    if !context.params.search.is_empty() {
        search_results = Some(
            LyricsSearchResultsTemplate::new(
                LyricsSearchResultsParams {
                    search: context.params.search.clone(),
                }
            ).await?
        );
    } else if !context.params.song.is_empty() {
        let song = database::get_song_by_slug_and_band_id(&context.params.song, band_id).await?;
        let album = database::get_album_by_song_slug(&context.params.song, &band_slug).await?;
        seo_title = format!(r#" for 「{}」by {}"#, &song.song_name, &band_name);

        comment_section = Some(
            CommentSectionTemplate::<LyricsPageContext>::new(
                CommentSectionParams {
                    context,
                    section: &CommentSectionName::Lyrics,
                    section_tag_id: Some(song.id),
                    page_number: context.params.comments_page,
                }
            ).await?
        );

        let mut lyrics: Option<Lyrics> = None;
        let mut translations = database::get_lyrics_by_song_id(song.id).await?;
        if translations.len() > 0 {
            if let Some(index) = translations.iter().position(|translation| translation.username == context.params.contributor) {
                lyrics = Some(translations.remove(index));
            } else {
                lyrics = translations.drain(..1).next();
            }
        }
        if let Some(lyrics) = &lyrics {
            contributor = lyrics.username.clone();
        }

        song_lyrics = Some(
            SongLyricsTemplate::new(SongLyricsParams {
                song, album_slug: album.album_slug, album_name: album.album_name,
                band_slug: band_slug.clone(), band_name: band_name.clone(),
                lyrics,
            }).await?
        );
    } else if !context.params.album.is_empty() {
        let album = database::get_album_by_slug_and_band_id(&context.params.album, band_id).await?;
        seo_title = format!(r#" for Album "{}" by {}"#, &album.album_name, &band_name);
        album_detail = Some(
            AlbumDetailTemplate::new(AlbumDetailParams { album, band_slug: band_slug.clone() }).await?
        );
    } else {
        recent_translations = Some(
            RecentTranslationsTemplate::new(RecentTranslationsParams { band_id }).await?
        );
    }

    if band_id == 0 {
        return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Band not found.")));
    }

    let mut album_name = None;
    let mut album_slug = None;
    let mut song_name = None;
    let mut song_slug = None;
    if let Some(album_detail) = &album_detail {
        album_name = Some(album_detail.album.album_name.clone());
        album_slug = Some(album_detail.album.album_slug.clone());
    } else if let Some(song_lyrics) = &song_lyrics {
        album_name = Some(song_lyrics.album_name.clone());
        album_slug = Some(song_lyrics.album_slug.clone());
        song_name = Some(song_lyrics.song.song_name.clone());
        song_slug = Some(song_lyrics.song.song_slug.clone());
    }
    let lyrics_edit_bar = LyricsEditBarTemplate::new(
        LyricsEditBarParams {
            context,
            album_name,
            album_slug,
            band_name: band_name.clone(),
            band_slug: band_slug.clone(),
            song_name,
            song_slug,
            contributor,
        }
    ).await?;

    Ok(
        LyricsTemplateCommon {
            album_detail,
            band_id,
            band_slug,
            band_name,
            bands,
            comment_section,
            lyrics_edit_bar,
            recent_translations,
            search_results,
            seo_title,
            song_lyrics,
        }
    )
}
