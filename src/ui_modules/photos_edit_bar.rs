use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ UserPermission };
use crate::router::routes::photos::{ PhotosPageContext };

pub struct PhotosEditBarParams<'a, PhotosPageContext> {
    pub context: &'a PhotosPageContext,
    pub photo_album_contributor: Option<String>,
    pub photo_contributor: Option<String>,
    pub photo_title: String,
    pub photo_album_title: String,
}

#[derive(Template)]
#[template(path = "ui_modules/photos_edit_bar.html")]
pub struct PhotosEditBarTemplate<'a, PhotosPageContext> {
    phantom: PhantomData<&'a PhotosPageContext>,
    can_create_photo_album: bool,
    can_edit_photo_album: bool,
    can_delete_photo_album: bool,
    can_create_photo: bool,
    can_edit_photo: bool,
    can_delete_photo: bool,
    photo_album_slug: &'a str,
    photo_id: i32,
    photo_title: String,
    photo_album_title: String,
}
impl<'a> PhotosEditBarTemplate<'a, PhotosPageContext> {
    pub async fn new(
        params: PhotosEditBarParams<'a, PhotosPageContext>,
    ) -> Result<PhotosEditBarTemplate<'a, PhotosPageContext>, Box<dyn Error>> {
        let PhotosEditBarParams {
            context, photo_album_contributor, photo_contributor, photo_title, photo_album_title
        } = params;

        let mut can_create_photo_album = false;
        let mut can_edit_photo_album = false;
        let mut can_delete_photo_album = false;
        let mut can_create_photo = false;
        let mut can_edit_photo = false;
        let mut can_delete_photo = false;

        if let Some(user) = &context.user {
            can_create_photo_album = user.permissions.contains(&UserPermission::CreateOwnPhotoAlbum);
            can_create_photo = user.permissions.contains(&UserPermission::UploadOwnPhoto);
            if !context.params.album.is_empty() {
                if let Some(photo_album_contributor) = &photo_album_contributor {
                    can_edit_photo_album =
                        user.permissions.contains(&UserPermission::EditPhotoAlbum) || (
                            user.permissions.contains(&UserPermission::EditOwnPhotoAlbum) &&
                            photo_album_contributor == &user.username
                        );
                    can_delete_photo_album =
                        user.permissions.contains(&UserPermission::DeletePhotoAlbum) || (
                            user.permissions.contains(&UserPermission::DeleteOwnPhotoAlbum) &&
                            photo_album_contributor == &user.username
                        );
                }

                if context.params.photo > -1 {
                    if let Some(photo_contributor) = &photo_contributor {
                        can_edit_photo =
                            user.permissions.contains(&UserPermission::EditPhoto) || (
                                user.permissions.contains(&UserPermission::EditOwnPhoto) &&
                                photo_contributor == &user.username
                            );
                        can_delete_photo =
                            user.permissions.contains(&UserPermission::DeletePhoto) || (
                                user.permissions.contains(&UserPermission::DeleteOwnPhoto) &&
                                photo_contributor == &user.username
                            );
                    }
                }
            }
        }

        Ok(PhotosEditBarTemplate {
            phantom: PhantomData,
            can_create_photo_album,
            can_edit_photo_album,
            can_delete_photo_album,
            can_create_photo,
            can_edit_photo,
            can_delete_photo,
            photo_album_slug: context.params.album.as_str(),
            photo_id: context.params.photo,
            photo_title,
            photo_album_title,
        })
    }
}
