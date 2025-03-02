use totp_rs::{ Algorithm, TOTP, Secret };
use qrcode::QrCode;
use qrcode::render::unicode;
use qrcode::render::minipng;

pub fn generate_secret() {
    Secret::generate_secret();
}

pub fn generate_qr_code(secret: &str, username: &str) {
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.to_bytes(),
    ).unwrap();

    let otpath_url = totp.get_url(username, "supercarband.com");
    let code = QrCode::new(otpath_url.as_bytes()).unwrap();

    let qr_text = code.render::<unicode::Dense1x2>().build();

    let png_bytes = code.render::<minipng::Color>().build();
    // let image = code.render::<Luma<u8>>().build();
}
