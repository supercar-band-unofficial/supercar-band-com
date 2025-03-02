use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ self, JoinedSongSlugs };

pub struct SongListParams {
    pub band_id: i32,
}

pub struct AlphabeticalSongGroup {
    pub letter: char,
    pub songs: Vec<JoinedSongSlugs>,
}

#[derive(Template)]
#[template(path = "ui_modules/song_list.html")]
pub struct SongListTemplate<'a> {
    phantom: PhantomData<&'a ()>,
    alphabetical_groups: Vec<AlphabeticalSongGroup>,
}
impl<'a> SongListTemplate<'a> {
    pub async fn new(
        params: SongListParams,
    ) -> Result<SongListTemplate<'a>, Box<dyn Error>> {
        let SongListParams { band_id } = params;

        let songs = database::get_song_slugs_by_band_id(band_id).await?;
        let mut alphabetical_groups: Vec<AlphabeticalSongGroup> = Vec::new();
        let mut current_group = AlphabeticalSongGroup {
            letter: '\0',
            songs: Vec::new(),
        };
        for song in songs {
            if let Some(first_letter) = song.song_slug.chars().next() {
                let first_letter = first_letter.to_uppercase().next().unwrap();

                if current_group.letter != first_letter {
                    if current_group.letter != '\0' {
                        alphabetical_groups.push(current_group);
                    }
        
                    current_group = AlphabeticalSongGroup {
                        letter: first_letter,
                        songs: Vec::new(),
                    };
                }

                current_group.songs.push(song);
            }
        }
        if current_group.letter != '\0' {
            alphabetical_groups.push(current_group);
        }

        Ok(SongListTemplate {
            phantom: PhantomData,
            alphabetical_groups,
        })
    }
}

fn create_song_href(song: &JoinedSongSlugs) -> String {
    format!("/lyrics/{}/{}/{}/", song.band_slug, song.album_slug, song.song_slug)
}
