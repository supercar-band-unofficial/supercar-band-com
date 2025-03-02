# SupercarBand.com

This repository contains the source code of https://SupercarBand.com. It does not contain any site data. In other words, if you use this code to host the website you will end up with an empty website with no users, lyric translations, comments, etc.

After cloning this repository, there are a few manual steps to set up the application before it can run.

## Operating System Dependencies

OpenSSL must be installed.

```sh
# Ubuntu/Debian
sudo apt install pkg-config libssl-dev
# Fedora
sudo dnf install openssl-devel
# OSX
brew install openssl@3
```

A load balancer (like Nginx) needs to set the `X-Forwarded-For` header with the original requester IP address. This is **required**, otherwise rate limit checkers will unfairly impact all visitors.

```nginx
proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
```

## Configuration

You must create a "secrets.toml" file at `/config/secrets.toml` to provide the credentials to connect to a SQL database and SMTP relay server. Here is an example of how that file should look:

```
[database]
host = "localhost"
port = 3306
user = "admin"
password = "password"

[smtp]
username = "admin"
password = "password"
relay_server_name = "smtp.example.com"
```

Without this file, the application will not run. Ensure that the `secrets.toml` file has restrictive file permissions.

### 1. Database

SupercarBand.com stores all of its data in a MySQL database. It may work in similar SQL databases like MariaDB, but this is not guaranteed.

### 2. SMTP Relay Server

This is coded to connect via STARTTLS (port 587), so ensure that your firewall and network provider isn't blocking outgoing traffic via this port.

## Uploads Folder Structure

You must create an "uploads" folder next to the executable (or next to the src folder during development) that uses this folder structure:

```
uploads/
└── assets/
    └── images/
        ├── album-covers/
        ├── booklets/
        ├── photos/
        |   └── thumbs/
        ├── profile-pictures/
        └── tmp/
```

Give only the user that runs this application access to this folder.

## Building / Running

This is a standard Rust application. To build:

```sh
cargo build
```

To run:

```sh
cargo run
```

To run while watching for source code changes (for development):

```sh
cargo watch -x run
```

If you encounter weird errors, sometimes it is helpful to clean and rebuild.

```sh
cargo clean
```

## Site Permissions

User permissions are granular, and designed to prevent anyone from totally trashing the content on the site. Many permissions are obtained just by creating an account. Permissions marked for admin doesn't mean every moderator will have every admin permission, those can be individually assigned.

**Examples:**

1. A user who has contributed a lot of content to the site had their password guessed and a bad actor logged in to their account. The bad actor comments with a bunch of spam or starts deleting things that the user has contributed. All permissions can be removed until that user does a password reset on their account.

2. A user won't stop uploading images that break the terms of service after being warned. `upload_own_photo` and `upload_own_profile_picture` permissions can be removed.

3. A user wants to help moderate comments on the site (deleting spam, etc.), the `delete_comment` permission can be added, which also gives the site admin final approval on the deletion.

| Permission | Description | How to Obtain |
|:-----------|:------------|:--------------|
| create_band | Allows creating a band, affects the lyrics and tabs pages. | Admin |
| edit_band | Allows editing the info of an existing band. | Admin |
| delete_band | Allows deleting a band. | Admin |
| create_album | Allows creating an album for any band. | Admin |
| edit_album | Allows editing the album info for any band. | Admin |
| delete_album | Allows deleting an album for any band. | Admin |
| create_own_lyrics | Allows creating lyric translations for any song. | New User |
| edit_own_lyrics | Allows editing your own lyric translations. | New User |
| edit_lyrics | Allows editing any lyric translations. | Admin |
| delete_own_lyrics | Allows deleting your own lyric translations. | New User |
| delete_lyrics | Allows deleting any lyric translations. | Admin |
| create_own_tabs | Allows creating tabs on the tabs page. | New User |
| edit_own_tabs | Allows editing your own tabs on the tabs page. | New User |
| edit_tabs | Allows editing any tabs on the tabs page. | Admin |
| delete_own_tabs | Allows deleting your own tabs on the tabs page. | New User |
| delete_tabs | Allows deleting any tabs on the tabs page. | Admin |
| create_own_photo_album | Allows creating a photo album on the photos page. | New User |
| edit_own_photo_album | Allows editing your own photo album on the photos page. | New User |
| edit_photo_album | Allows editing any photo album on the photos page. | Admin |
| delete_own_photo_album | Allows deleting your own photo album on the photos page, if it is empty. | New User |
| delete_photo_album | Allows deleting any photo album on the photos page. | Admin |
| upload_own_photo | Allows uploading a photo to the photos page. | New User |
| edit_own_photo | Allows editing your own photo on the photos page. | New User |
| edit_photo | Allows editing any photo on the photos page. | Admin |
| delete_own_photo | Allows deleting your own photo on the photos page. | New User |
| delete_photo | Allows deleting any photo on the photos page. | Admin |
| create_own_video_category | Allows creating a video category on the videos page. | New User |
| edit_own_video_category | Allows editing your own video category on the videos page. | New User |
| edit_video_category | Allows editing any video category on the videos page. | Admin |
| delete_own_video_category | Allows deleting your own video category on the videos page, if it is empty. | New User |
| delete_video_category | Allows deleting any video category on the videos page. | Admin |
| upload_own_video | Allows uploading a video on the videos page. | New User |
| edit_own_video | Allows editing your own video on the videos page. | New User |
| edit_video | Allows editing any video on the videos page. | Admin |
| delete_own_video | Allows deleting your own video on the videos page. | New User |
| delete_video | Allows deleting any video on the videos page. | Admin |
| create_own_comment | Allows creating a comment on any page in the site (or chatbox). | New User |
| delete_own_comment | Allows deleting your own comments on the site. | New User |
| delete_comment | Allows deleting any comment on the site. | Admin |
| edit_own_profile_info | Allows editing information on your own profile. | New User |
| upload_own_profile_picture | Allows uploading a custom profile image. | New User |
| send_dms | Allows sending direct messages to other users. | New User |
| delete_user | Allows deleting a user account. | Admin |
| approve_queued_deletion | All deletions on the site are immediately hidden from public view and added to a review queue for final deletion. This allows approval for final deletion, removing it from the database. | Admin |
| undo_queued_deletion | Allows undoing a deletion sitting in the deletion review queue, making it reappear on the site. | Admin |
| ban_ips | Allows initiating ip address bans to prevent site abuse. | Admin |
| edit_user_permissions | Allows changing any user's permissions. | Admin |

## User Profile Preferences

These are preferences specific to each user profile that have an effect on permissions or notifications.

| Permission | Description |
|:-----------|:------------|
| allow_profile_comments | Allows commenting on the public profile in general. |
| allow_profile_guest_comments | Allows guests to comment on the public profile. |
| allow_dms | Allows any user to send dms. |
| notify_lyric_post_comments | Sends a notification when anyone comments on a song translation the user has created. |
| notify_tabs_comments | Sends a notification when anyone comments on tabs the user has created. |
| notify_photo_comments | Sends a notification when anyone comments on a photo the user has uploaded. |
| notify_video_comments | Sends a notification when anyone comments on a video the user has posted. |
| notify_profile_comments | Sends a notification when anyone comments on their profile. |
| notify_dms | Sends a notification when anyone sends a direct message. |
| notify_comment_replies | Sends a notification when anyone replies to one of the user's comments on the site. |
| notify_global_feed | Posts user activity on the site events on the home page. |
