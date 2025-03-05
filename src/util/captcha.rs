/**
 * This generates captcha images/audio and stores them in memory for 15 minutes.
 * It serves as a basic protection against bot spam, that of which this site has had a lot
 * of for whatever reason. There's nothing for bots to gain here...
 */

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::RwLock;
use chrono::prelude::{ DateTime, Utc };
use captcha::{ Captcha, Geometry, CaptchaName };
use captcha::filters::{ Cow, Grid, Noise, Wave, Dots };
use rand::{ thread_rng, Rng };
use sha2::{ Sha256, Digest };
use tokio::sync::OnceCell;
use uuid::Uuid;

pub static CAPTCHAS: OnceCell<RwLock<HashMap<String, GeneratedCaptcha>>> = OnceCell::const_new();
static CAPTCHA_CHARSET: &str = "ABCDEFGHJKMNPQRSTUVWXYZabcdefghijkmnpqrstuv23456789";
static CAPTCHA_WIDTH: u32 = 200;
static CAPTCHA_HEIGHT: u32 = 110;
static CAPTCHA_NAMES: &[CaptchaName] = &[CaptchaName::Amelia, CaptchaName::Lucy, CaptchaName::Mila];
static CAPTCHA_STORE_TIME: i64 = 900000;

#[derive(Debug)]
pub struct CaptchaValidationError {
    details: String,
}
impl fmt::Display for CaptchaValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Captcha validation error occurred: {}", self.details)
    }
}

#[derive(Clone, Debug)]
pub struct GeneratedCaptcha {
    pub chars: String,
    pub timestamp: DateTime<Utc>,
    pub pow_challenge: String,
}

pub fn init_captchas() {
    CAPTCHAS
        .set(RwLock::new(HashMap::new()))
        .expect("Captchas already initialized.");
}

fn generate_captcha_chars() -> String {
    let string_length = 5;
    (0..string_length)
        .map(|_| {
            let mut rng = thread_rng();
            let idx = rng.gen_range(0..CAPTCHA_CHARSET.len());
            CAPTCHA_CHARSET.chars().nth(idx).unwrap()
        })
        .collect()
}

fn generate_captcha_image(chars: Vec<char>) -> Option<Vec<u8>> {
    let mut captcha = Captcha::new();
    for char in chars {
        captcha.set_chars(&[char]);
        captcha.add_chars(1);
    }
    let captcha_name = &CAPTCHA_NAMES[
        thread_rng().gen::<usize>() % CAPTCHA_NAMES.len()
    ];
    match &captcha_name {
        CaptchaName::Amelia => {
            captcha
                .apply_filter(Noise::new(0.2))
                .apply_filter(Grid::new(8, 8))
                .apply_filter(Wave::new(2.0, 10.0)).view(CAPTCHA_WIDTH, CAPTCHA_HEIGHT)
                .apply_filter(Dots::new(10).max_radius(7).min_radius(3));
        },
        CaptchaName::Lucy => {
            captcha
                .apply_filter(Noise::new(0.1))
                .apply_filter(Grid::new(8, 8))
                .view(CAPTCHA_WIDTH, CAPTCHA_HEIGHT);
        },
        CaptchaName::Mila => {
            captcha
                .apply_filter(Noise::new(0.2))
                .apply_filter(Wave::new(2.0, 20.0))
                .view(CAPTCHA_WIDTH, CAPTCHA_HEIGHT)
                .apply_filter(
                    Cow::new()
                        .min_radius(40)
                        .max_radius(50)
                        .circles(1)
                        .area(Geometry::new(40, 150, 50, 70)),
                );
        },
    }
    captcha.as_png()
}

pub fn get_captcha_image_by_id(id: &str) -> Option<Vec<u8>> {
    let captchas = CAPTCHAS.get().expect("Captchas not initialized.");
    let captchas_read = captchas.read().unwrap_or_else(|poisoned| poisoned.into_inner());
    let generated_captcha = captchas_read.get(id).cloned();
    match generated_captcha {
        Some(generated_captcha) => {
            let chars: Vec<char> = generated_captcha.chars.chars().collect();
            generate_captcha_image(chars)
        },
        _ => None,
    } 
}

pub fn get_captcha_pow_challenge_by_id(id: &str) -> Option<String> {
    let captchas = CAPTCHAS.get().expect("Captchas not initialized.");
    let captchas_read = captchas.read().unwrap_or_else(|poisoned| poisoned.into_inner());
    match captchas_read.get(id) {
        Some(captcha) => Some(captcha.pow_challenge.clone()),
        _ => None,
    }
}

pub fn generate_captcha() -> Result<String, Box<dyn Error>> {
    let captchas = CAPTCHAS.get().expect("Captchas not initialized.");
    let mut captchas_write = captchas.write().unwrap_or_else(|poisoned| poisoned.into_inner());

    let now: DateTime<Utc> = Utc::now();
    captchas_write.retain(
        |_, value| now.signed_duration_since(value.timestamp).num_milliseconds() >= CAPTCHA_STORE_TIME
    );

    let generated_captcha = GeneratedCaptcha {
        chars: generate_captcha_chars(),
        timestamp: Utc::now(),
        pow_challenge: format!("challenge_{}", Uuid::new_v4().to_string()),
    };
    let id = Uuid::new_v4().to_string();
    captchas_write.insert(id.clone(), generated_captcha);

    let captcha_count = captchas_write.len();
    if captcha_count == 100 || captcha_count == 500 || captcha_count == 1000 {
        tracing::warn!("The number of generated captchas just passed {}", captcha_count);
    }

    Ok(id)
}

pub fn validate_captcha(id: &str, captcha_entry: &str) -> Result<(), CaptchaValidationError> {
    let captchas = CAPTCHAS.get().expect("Captchas not initialized.");
    let mut captchas_write = captchas.write().unwrap_or_else(|poisoned| poisoned.into_inner());
    let generated_captcha: Option<GeneratedCaptcha> = captchas_write.get(id).cloned();
    match generated_captcha {
        Some(generated_captcha) => {
            if &generated_captcha.chars == captcha_entry {
                captchas_write.remove(id);
                return Ok(());
            } else {
                return Err(CaptchaValidationError { details: format!("Captcha mismatch with entry {}.", captcha_entry) });
            }
        },
        _ => Err(CaptchaValidationError { details: format!("Captcha with id {} not found.", id) }),
    }
}

pub fn validate_pow_challenge(challenge: &str, nonce: u64, difficulty: usize) -> bool {
    let prefix = "0".repeat(difficulty);
    let input = format!("{}{}", challenge, nonce);

    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    let hash_hex = hex::encode(result);

    hash_hex.starts_with(&prefix)
}
