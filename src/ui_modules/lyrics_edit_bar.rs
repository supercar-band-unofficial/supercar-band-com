use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ UserPermission };
use crate::router::routes::lyrics::{ LyricsPageContext };

pub struct LyricsEditBarParams<'a, LyricsPageContext> {
    pub context: &'a LyricsPageContext,
    pub album_name: Option<String>,
    pub album_slug: Option<String>,
    pub band_name: String,
    pub band_slug: String,
    pub contributor: String,
    pub song_name: Option<String>,
    pub song_slug: Option<String>,
}

#[derive(Template)]
#[template(path = "ui_modules/lyrics_edit_bar.html")]
pub struct LyricsEditBarTemplate<'a, LyricsPageContext> {
    phantom: PhantomData<&'a LyricsPageContext>,
    album_name: String,
    album_slug: String,
    band_name: String,
    band_slug: String,
    can_create_band: bool,
    can_create_album: bool,
    can_create_lyrics: bool,
    can_edit_band: bool,
    can_edit_album: bool,
    can_edit_lyrics: bool,
    can_delete_band: bool,
    can_delete_album: bool,
    can_delete_lyrics: bool,
    song_name: String,
    song_slug: String,
}
impl<'a> LyricsEditBarTemplate<'a, LyricsPageContext> {
    pub async fn new(
        params: LyricsEditBarParams<'a, LyricsPageContext>,
    ) -> Result<LyricsEditBarTemplate<'a, LyricsPageContext>, Box<dyn Error>> {
        let LyricsEditBarParams {
            context, album_name, album_slug, band_name, band_slug, contributor, song_name, song_slug
        } = params;

        let mut can_create_band = false;
        let mut can_create_album = false;
        let mut can_create_lyrics = false;
        let mut can_edit_band = false;
        let mut can_edit_album = false;
        let mut can_edit_lyrics = false;
        let mut can_delete_band = false;
        let mut can_delete_album = false;
        let mut can_delete_lyrics = false;

        if let Some(user) = &context.user {
            can_create_band = user.permissions.contains(&UserPermission::CreateBand);
            can_create_album = user.permissions.contains(&UserPermission::CreateAlbum);
            can_create_lyrics = user.permissions.contains(&UserPermission::CreateOwnLyrics);
            can_edit_band = user.permissions.contains(&UserPermission::EditBand);
            can_delete_band = user.permissions.contains(&UserPermission::DeleteBand);
            if let Some(_) = &album_slug {
                can_edit_album = user.permissions.contains(&UserPermission::EditAlbum);
                can_delete_album = user.permissions.contains(&UserPermission::DeleteAlbum);
                if let Some(_) = &song_slug {
                    can_edit_lyrics =
                        user.permissions.contains(&UserPermission::EditLyrics) || (
                            user.permissions.contains(&UserPermission::EditOwnLyrics) &&
                            contributor == user.username
                        );
                    can_delete_lyrics =
                        user.permissions.contains(&UserPermission::DeleteLyrics) || (
                            user.permissions.contains(&UserPermission::DeleteOwnLyrics) &&
                            contributor == user.username
                        );
                }
            }
        }

        Ok(LyricsEditBarTemplate {
            phantom: PhantomData,
            album_name: album_name.unwrap_or_else(|| String::from("")),
            album_slug: album_slug.unwrap_or_else(|| String::from("")),
            band_name,
            band_slug,
            can_create_band,
            can_create_album,
            can_create_lyrics,
            can_edit_band,
            can_edit_album,
            can_edit_lyrics,
            can_delete_band,
            can_delete_album,
            can_delete_lyrics,
            song_name: song_name.unwrap_or_else(|| String::from("")),
            song_slug: song_slug.unwrap_or_else(|| String::from("")),
        })
    }
}
