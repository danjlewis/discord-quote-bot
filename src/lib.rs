use std::cmp;

use anyhow::Result;
use chrono::{DateTime, Utc};
use image::{DynamicImage, GenericImageView};
use unsplash::{GetRandomPhotoOptions, UnsplashClient};

mod unsplash;

pub struct QuoteRenderer {
    unsplash_client: UnsplashClient,
}

impl QuoteRenderer {
    pub fn new(unsplash_access_key: &str) -> Self {
        let unsplash_client = UnsplashClient::new(unsplash_access_key);

        Self { unsplash_client }
    }

    // TODO: implement proper error handling
    pub async fn render(
        &self,
        quote: &str,
        author: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<DynamicImage> {
        let background_image = self.generate_background_image().await?;

        Ok(background_image)
    }

    async fn generate_background_image(&self) -> Result<DynamicImage> {
        let api_options = GetRandomPhotoOptions {
            collections: Some(String::from("11649432")),
            ..Default::default()
        };
        let img = self.unsplash_client.get_random_photo(api_options).await?;

        let img = Self::crop_background_image(&img);

        Ok(img)
    }

    fn crop_background_image(img: &DynamicImage) -> DynamicImage {
        let original_size = img.dimensions();
        let side_length = cmp::min(original_size.0, original_size.1);

        let x = (original_size.0 - side_length) / 2;
        let y = (original_size.1 - side_length) / 2;

        img.crop_imm(x, y, side_length, side_length)
    }
}
