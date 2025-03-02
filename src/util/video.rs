use url::Url;

fn get_root_domain(url_str: &str) -> Option<String> {
    let url = Url::parse(url_str).ok()?;
    let host = url.host_str()?;

    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() >= 2 {
        Some(format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]))
    } else {
        Some(host.to_string())
    }
}

/**
 * Check if the video url supports embeds on this website.
 */
pub fn is_supported_video_url(video_url: &str) -> bool {
    if let Some(root_domain) = get_root_domain(video_url) {
        let root_domain_str = root_domain.as_str();
        root_domain_str == "youtube.com" || root_domain_str == "youtu.be"
        || root_domain_str == "dailymotion.com" || root_domain_str == "nicovideo.jp"
        || root_domain_str == "nico.ms"
    } else {
        false
    }
}

/**
 * With the given video URL, returns the URL to its thumbnail image.
 */
pub fn get_video_thumbnail_url(video_url: &str) -> String {
    if let Some(root_domain) = get_root_domain(video_url) {
        let root_domain_str = root_domain.as_str();
        match root_domain_str {
            "youtube.com" | "youtu.be" => {
                format!("https://img.youtube.com/vi/{}/hqdefault.jpg", get_video_id(video_url, root_domain_str))
            },
            "dailymotion.com" => {
                format!("https://www.dailymotion.com/thumbnail/video/{}", get_video_id(video_url, root_domain_str))
            },
            "nicovideo.jp" | "nico.ms" => {
                let video_id = get_video_id(video_url, root_domain_str);
                format!("https://nicovideo.cdn.nimg.jp/thumbnails/{}/{}", &video_id, &video_id)
            },
            _ => String::from("https://img.youtube.com/vi/0/hqdefault.jpg"),
        }
    } else {
        String::from("https://img.youtube.com/vi/0/hqdefault.jpg")
    }
}

/**
 * Gets embed markup for the given video URL.
 */
 pub fn create_video_embed_iframe(video_url: &str) -> String {
    if let Some(root_domain) = get_root_domain(video_url) {
        let root_domain_str = root_domain.as_str();
        let video_id = get_video_id(video_url, root_domain_str);
        match root_domain_str {
            "youtube.com" | "youtu.be" => {
                format!(r#"<iframe frameborder=0 width="720" height="400" src="https://www.youtube.com/embed/{}"></iframe>"#, video_id)
            },
            "dailymotion.com" => {
                format!(r#"<iframe frameborder=0 width="720" height="400" src="https://www.dailymotion.com/embed/video/{}"></iframe>"#, video_id)
            },
            "nicovideo.jp" | "nico.ms" => {
                format!(r#"<iframe frameborder=0 width="720" height="400" src="https://embed.nicovideo.jp/watch/sm{}?ap=1"></iframe>"#, video_id)
            },
            _ => {
                String::from("")
            }
        }
    } else {
        String::from("")
    }
}

/**
 * Retrieves the id of a video from its URL.
 */
fn get_video_id<'a>(video_url: &'a str, root_domain: &'a str) -> &'a str {
    match root_domain {
        "youtube.com" => {
            if let Some(p0) = video_url.find("/shorts/") {
                if let Some(p1) = video_url.find("?") {
                    &video_url[p0+8..p1]
                } else {
                    &video_url[p0+8..]
                }
            } else if let Some(p0) = video_url.find("watch?v=") {
                if let Some(p1) = video_url.find("&") {
                    &video_url[p0+8..p1]
                } else {
                    &video_url[p0+8..]
                }
            } else if let Some(p0) = video_url.find("/watch/") {
                if let Some(p1) = video_url.find("?") {
                    &video_url[p0+7..p1]
                } else {
                    &video_url[p0+7..]
                }
            } else {
                ""
            }
        },
        "youtu.be" => {
            if let Some(p0) = video_url.find("youtu.be/") {
                if let Some(p1) = video_url.find("?") {
                    &video_url[p0+9..p1]
                } else {
                    &video_url[p0+9..]
                }
            } else {
                ""
            }
        },
        "dailymotion.com" => {
            if let Some(p0) = video_url.find("video/") {
                if let Some(p1) = video_url.find("_") {
                    &video_url[p0+6..p1]
                } else if let Some(p1) = video_url.find("?") {
                    &video_url[p0+6..p1]
                } else {
                    &video_url[p0+6..]
                }
            } else {
                ""
            }
        },
        "nicovideo.jp" => {
            if let Some(p0) = video_url.find("watch/sm") {
                if let Some(p1) = video_url.find("?") {
                    &video_url[p0+8..p1]
                } else {
                    &video_url[p0+8..]
                }
            } else {
                ""
            }
        },
        "nico.ms" => {
            if let Some(p0) = video_url.find("/sm") {
                if let Some(p1) = video_url.find("?") {
                    &video_url[p0+3..p1]
                } else {
                    &video_url[p0+3..]
                }
            } else {
                ""
            }
        },
        _ => {
            ""
        }
    }
}
