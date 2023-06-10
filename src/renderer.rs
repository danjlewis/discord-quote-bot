use anyhow::Result;
use chrono::{DateTime, Utc};
use image::{DynamicImage, GenericImage, Rgb, RgbImage};

// TODO: implement proper error handling
pub async fn render(
    background_image: &DynamicImage,
    _quote: &str,
    _author: &str,
    _timestamp: DateTime<Utc>,
) -> Result<RgbImage> {
    let background_image = background_image.to_rgb8();

    let average_color = calculate_average_color(&background_image);

    let mut image = background_image;
    image.copy_from(&RgbImage::from_pixel(100, 100, average_color), 0, 0)?;

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
