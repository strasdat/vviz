//! Utilities. Collection of functions which don't fit in other modules.

/// Union error type.
#[derive(Debug)]
pub enum ImageFrumUrlError {
    /// error from reqwest crate
    Reqwest(reqwest::Error),
    /// error from image-rs crate
    Image(image::ImageError),
}

impl From<reqwest::Error> for ImageFrumUrlError {
    fn from(e: reqwest::Error) -> Self {
        ImageFrumUrlError::Reqwest(e)
    }
}

impl From<image::ImageError> for ImageFrumUrlError {
    fn from(e: image::ImageError) -> Self {
        ImageFrumUrlError::Image(e)
    }
}

/// Load image from web.
pub fn load_image_from_url<T: reqwest::IntoUrl>(
    url: T,
) -> Result<image::DynamicImage, ImageFrumUrlError> {
    let bytes = reqwest::blocking::get(url)?.bytes()?;
    let cursor = std::io::Cursor::new(bytes);
    let img =
        image::io::Reader::with_format(std::io::BufReader::new(cursor), image::ImageFormat::Png)
            .decode()?;
    Ok(img)
}
