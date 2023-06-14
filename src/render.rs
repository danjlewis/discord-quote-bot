use chrono::{DateTime, Utc};
use image::{GenericImage, Rgb, RgbImage};

// TODO: implement proper error handling
pub fn render(
    background_image: &RgbImage,
    _quote: &str,
    _author: &str,
    _timestamp: DateTime<Utc>,
) -> RgbImage {
    let mut image = background_image.clone();

    let average_color = calculate_average_color(background_image);

    image
        .copy_from(&RgbImage::from_pixel(100, 100, average_color), 0, 0)
        .unwrap();

    image
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
