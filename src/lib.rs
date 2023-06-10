use anyhow::Result;
use chrono::{DateTime, Utc};
use image::{GenericImage, Rgb, RgbImage};
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
    ) -> Result<RgbImage> {
        let background_image = self.generate_background_image().await?;

        let average_color = Self::calculate_average_color(&background_image);

        let mut image = background_image;
        image.copy_from(&RgbImage::from_pixel(100, 100, average_color), 0, 0)?;

        Ok(image)
    }

    async fn generate_background_image(&self) -> Result<RgbImage> {
        let api_options = GetRandomPhotoOptions {
            collections: Some(String::from("11649432")),
            ..Default::default()
        };
        let image = self.unsplash_client.get_random_photo(api_options).await?;

        Ok(image)
    }

    fn calculate_average_color(image: &RgbImage) -> Rgb<u8> {
        let pixels = image.pixels();

        let sum = pixels.fold([0; 3], |mut acc, pixel| {
            for (i, v) in acc.iter_mut().enumerate() {
                *v += pixel.0[i] as u32;
            }

            acc
        });

        let dimensions = image.dimensions();
        let pixel_count = dimensions.0 * dimensions.1;

        let mut mean = [0u8; 3];
        for (i, v) in mean.iter_mut().enumerate() {
            *v = u8::try_from(sum[i] / pixel_count)
                .expect("mean of u8 values should be within u8 range");
        }

        Rgb(mean)
    }
}
