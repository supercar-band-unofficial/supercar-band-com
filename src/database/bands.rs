use std::error::Error;
use std::io;
use sqlx::{
    MySql,
    FromRow,
};
use super::get_pool;

use crate::util::sql::sanitize_like_clause_value;

#[allow(unused)]
#[derive(Debug, Default, Clone, FromRow)]
pub struct Band {
    pub id: i32,
    pub band_slug: String,
    pub band_name: String,
}

pub async fn get_all_bands() -> Result<Vec<Band>, Box<dyn Error>> {
    Ok(
        sqlx::query_as::<MySql, Band>("SELECT * FROM bands WHERE is_deleted=0 ORDER BY band_name ASC LIMIT 1000")
            .fetch_all(get_pool())
            .await?
    )
}

pub async fn get_band_by_id(id: i32) -> Result<Band, Box<dyn Error>> {
    Ok(
        sqlx::query_as::<MySql, Band>("SELECT * FROM bands WHERE id=? AND is_deleted=0 LIMIT 1")
            .bind(id)
            .fetch_one(get_pool())
            .await?
    )
}

pub async fn get_band_by_slug(band_slug: &str) -> Result<Band, Box<dyn Error>> {
    Ok(
        sqlx::query_as::<MySql, Band>("SELECT * FROM bands WHERE band_slug=? AND is_deleted=0 LIMIT 1")
            .bind(band_slug)
            .fetch_one(get_pool())
            .await?
    )
}

pub async fn find_bands_by_name(search: &str) -> Result<Vec<Band>, Box<dyn Error>> {
    if search.len() > 200 {
        return Err(
            Box::new(
                io::Error::new(io::ErrorKind::Other, "Search term was too long.")
            )
        );
    }

    let result = sqlx::query_as::<MySql, Band>(r#"
        SELECT * from bands
        WHERE band_name LIKE ? AND is_deleted=0
        LIMIT 15;
    "#)
        .bind(format!("%{}%", sanitize_like_clause_value(search)))
        .fetch_all(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn create_band(band: Band) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, Band>(r#"
        INSERT INTO bands (
            band_slug, band_name
        )
        VALUES (?, ?)
    "#)
        .bind(band.band_slug)
        .bind(band.band_name)
        .fetch_optional(get_pool())
        .await;

    match result {
        Ok(_) => {
            Ok(())
        }
        Err(e) => {
            Err(Box::new(e))
        }
    }
}

pub async fn update_band(band: Band) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, Band>(r#"
        UPDATE bands
        SET band_slug=?, band_name=?
        WHERE id=? AND is_deleted=0
        LIMIT 1
    "#)
        .bind(band.band_slug)
        .bind(band.band_name)
        .bind(band.id)
        .fetch_optional(get_pool())
        .await;

    match result {
        Ok(_) => {
            Ok(())
        }
        Err(e) => {
            Err(Box::new(e))
        }
    }
}

pub async fn mark_band_for_deletion(
    id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let delete_result = sqlx::query_as::<MySql, Band>(r#"
        UPDATE bands
        SET is_deleted=1
        WHERE id=?
        LIMIT 1
    "#)
        .bind(id)
        .fetch_optional(get_pool())
        .await;

    match delete_result {
        Ok(_) => {
            Ok(())
        }
        Err(e) => {
            Err(Box::new(e))
        }
    }
}
