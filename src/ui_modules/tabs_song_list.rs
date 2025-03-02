use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ self, AlbumType, JoinedSongSlugs };

pub struct TabsSongListParams {
    pub band_id: i32,
}

pub struct AlbumSongGroup {
    pub cover_picture_filename: String,
    pub album_name: String,
    pub songs: Vec<JoinedSongSlugs>,
}

#[derive(Template)]
#[template(path = "ui_modules/tabs_song_list.html")]
pub struct TabsSongListTemplate<'a> {
    phantom: PhantomData<&'a ()>,
    album_groups: Vec<AlbumSongGroup>,
}
impl<'a> TabsSongListTemplate<'a> {
    pub async fn new(
        params: TabsSongListParams,
    ) -> Result<TabsSongListTemplate<'a>, Box<dyn Error>> {
        let TabsSongListParams { band_id } = params;

        let albums = database::get_albums_by_band_id(band_id).await?;
        let mut album_groups: Vec<AlbumSongGroup> = Vec::with_capacity(albums.len());
        for album in albums {
            if &album.album_type == &AlbumType::Full || &album.album_type == &AlbumType::Single {
                let songs = database::get_song_slugs_by_ids(&album.song_ids()).await?;
                album_groups.push(
                    AlbumSongGroup {
                        cover_picture_filename: album.cover_picture_filename,
                        album_name: album.album_name,
                        songs,
                    }
                );
            }
        }

        Ok(TabsSongListTemplate {
            phantom: PhantomData,
            album_groups,
        })
    }
}

fn create_song_href(song: &JoinedSongSlugs) -> String {
    format!("/tabs/{}/{}/", song.band_slug, song.song_slug)
}
