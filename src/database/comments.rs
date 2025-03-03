use std::error::Error;
use chrono::NaiveDateTime;
use futures::future;
use sqlx::{
    FromRow,
    MySql,
    Row,
    Type,
};
use strum_macros::{ Display, EnumString };

use super::QueryOrder;
use super::get_pool;

#[derive(Debug, Default, Display, EnumString, Type)]
#[sqlx(type_name = "section")]
#[sqlx(rename_all = "lowercase")]
#[strum(serialize_all = "snake_case")]
pub enum CommentSectionName {
    #[default]
    Home,
    Lyrics,
    Tabs,
    Photos,
    Videos,
    Members,
    Chatbox,
}

#[allow(unused)]
#[derive(Debug, Default, FromRow)]
pub struct Comment {
    pub id: i32,
    pub username: String,
    pub ip_address: Option<String>,
    pub post_time: NaiveDateTime,
    pub section: CommentSectionName,
    pub section_tag_id: Option<i32>,
    pub reply_id: i32,
    pub comment: String,
    pub visibility: Option<i32>,
    pub likes: Option<i32>,
    pub profile_picture_filename: String,
}

pub struct CommentWithReplies {
    pub comment: Comment,
    pub replies: Vec<CommentWithReplies>,
}

pub async fn get_comments_count(
    section: &CommentSectionName,
    section_tag_id: Option<i32>,
    reply_id: i32,
) -> Result<u32, Box<dyn Error>> {
    if let Some(section_tag_id) = section_tag_id {
        return Ok(
            u32::try_from(sqlx::query("SELECT COUNT(*) FROM comments WHERE section=? AND section_tag_id=? AND reply_id=? AND is_deleted=0")
                .bind(section)
                .bind(section_tag_id)
                .bind(reply_id)
                .fetch_one(get_pool())
                .await?
                .get::<i64, usize>(0)
            )?
        );
    }
    Ok(
        u32::try_from(sqlx::query("SELECT COUNT(*) FROM comments WHERE section=? AND reply_id=? AND is_deleted=0")
            .bind(section)
            .bind(reply_id)
            .fetch_one(get_pool())
            .await?
            .get::<i64, usize>(0)
        )?
    )
}

pub async fn get_comment_by_id(id: i32) -> Result<Comment, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, Comment>(r#"
            SELECT comments.*,
                CASE
                    WHEN comments.username = 'Guest' THEN 'Guest.jpeg'
                    ELSE users.profile_picture_filename 
                END AS profile_picture_filename
            FROM comments
            LEFT JOIN users ON comments.username = users.username AND comments.username <> 'Guest'
            WHERE comments.id=? AND comments.is_deleted = 0
            LIMIT 1;
    "#)
        .bind(id)
        .fetch_one(get_pool())
        .await?;

    Ok(result)
}

pub async fn get_comments_in_range(
    start: u32,
    length: u32,
    section: &CommentSectionName,
    section_tag_id: Option<i32>,
    reply_id: i32,
    query_order: &QueryOrder,
) -> Result<Vec<Comment>, Box<dyn Error>> {
    let order = if query_order == &QueryOrder::Asc { "ASC" } else { "DESC" };
    if let Some(section_tag_id) = section_tag_id {
        return Ok(
            sqlx::query_as::<MySql, Comment>(
                format!(r#"
                    SELECT
                        comments.*,
                        CASE
                            WHEN comments.username = 'Guest' THEN 'Guest.jpeg'
                            ELSE users.profile_picture_filename 
                        END AS profile_picture_filename
                    FROM comments
                    LEFT JOIN users ON comments.username = users.username AND comments.username <> 'Guest'
                    WHERE comments.section = ?
                        AND comments.section_tag_id = ?
                        AND comments.reply_id = ?
                        AND comments.is_deleted = 0
                    ORDER BY comments.post_time {}
                    LIMIT ? OFFSET ?;
                "#, order).as_str()
            )
                .bind(section)
                .bind(section_tag_id)
                .bind(reply_id)
                .bind(length)
                .bind(start)
                .fetch_all(get_pool())
                .await?
        );
    }
    Ok(
        sqlx::query_as::<MySql, Comment>(
            format!(r#"
                SELECT
                    comments.*,
                    CASE
                        WHEN comments.username = 'Guest' THEN 'Guest.jpeg'
                        ELSE users.profile_picture_filename
                    END AS profile_picture_filename
                FROM comments
                LEFT JOIN users ON comments.username = users.username AND comments.username <> 'Guest'
                WHERE comments.section = ?
                    AND comments.reply_id = ?
                    AND comments.is_deleted = 0
                ORDER BY comments.post_time {}
                LIMIT ? OFFSET ?;
            "#, order).as_str()
        )
            .bind(section)
            .bind(reply_id)
            .bind(length)
            .bind(start)
            .fetch_all(get_pool())
            .await?
    )
}

pub async fn get_comments_in_range_with_replies(
    start: u32,
    length: u32,
    section: &CommentSectionName,
    section_tag_id: Option<i32>,
    reply_id: i32,
) -> Result<Vec<CommentWithReplies>, Box<dyn Error>> {
    let query_order = if reply_id > -1 { &QueryOrder::Asc } else { &QueryOrder::Desc };
    let comments = get_comments_in_range(start, length, section, section_tag_id, reply_id, query_order).await?;
    
    let futures = comments
        .into_iter()
        .map(|comment| {
            async move {
                let replies = get_comments_in_range_with_replies(
                    0, 30, section, section_tag_id, comment.id,
                ).await.unwrap();
                CommentWithReplies { comment, replies }
            }
        });

    Ok(future::join_all(futures).await)
}

pub async fn create_comment(
    comment: Comment
) -> Result<i32, Box<dyn Error + Send + Sync>> {
    let pool = get_pool();

    let result = sqlx::query(r#"
        INSERT INTO comments (
            username, post_time, ip_address, section, section_tag_id, reply_id, comment
        )
        VALUES (?, NOW(), ?, ?, ?, ?, ?)
    "#)
        .bind(comment.username)
        .bind(comment.ip_address)
        .bind(comment.section)
        .bind(comment.section_tag_id)
        .bind(comment.reply_id)
        .bind(comment.comment)
        .execute(pool)
        .await;
    
    match result {
        Ok(result) => {
            Ok(i32::try_from(result.last_insert_id()).unwrap_or_else(|_| 999))
        }
        Err(e) => {
            Err(Box::new(e))
        }
    }
}
