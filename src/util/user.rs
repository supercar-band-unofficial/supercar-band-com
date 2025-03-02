/**
 * Common utilities relating to user/member accounts.
 */

use urlencoding::encode;

pub fn is_guest_user(username: &str) -> bool {
    username.to_lowercase() == "guest"
}

pub fn create_user_profile_href(username: &str) -> String {
    format!("/members/{}/", encode(username))
}
