use std::cmp;

use chrono::{DateTime, Utc};
use image::{buffer::ConvertBuffer, imageops, Pixel, Rgb, RgbImage, Rgba, RgbaImage};
use imageproc::drawing;
use rusttype::Scale;

use crate::assets::fonts::Lato;

// TODO: implement proper error handling
pub fn render(
    background_image: &RgbImage,
    quote: &str,
    _author: &str,
    _timestamp: DateTime<Utc>,
) -> RgbImage {
    let mut image: RgbaImage = background_image.clone().convert();
    let dimensions = image.dimensions();

    let average_color = calculate_average_color(background_image);

    const TEXT_BOX_OPACITY: f64 = 0.6;

    const MARGIN_PERCENT: f64 = 0.2;
    let margin_size = (dimensions.1 as f64 * MARGIN_PERCENT) as u32;

    const QUOTE_PADDING_PERCENT: f64 = 0.05;
    let quote_padding_size = (dimensions.1 as f64 * QUOTE_PADDING_PERCENT) as u32;

    let max_quote_box_dimensions = (
        dimensions.0 - margin_size * 2,
        dimensions.1 - margin_size * 2,
    );
    let max_quote_box_position = (margin_size, margin_size);

    let max_quote_text_dimensions = (
        max_quote_box_dimensions.0 - quote_padding_size * 2,
        max_quote_box_dimensions.1 - quote_padding_size * 2,
    );

    let quote_text = render_quote_text(quote, &average_color, max_quote_text_dimensions);
    let quote_text_dimensions = quote_text.dimensions();

    let quote_box_dimensions = (
        quote_text_dimensions.0 + quote_padding_size * 2,
        quote_text_dimensions.1 + quote_padding_size * 2,
    );
    // center-aligned within max quote box
    let quote_box_position = (
        max_quote_box_position.0 + max_quote_box_dimensions.0 / 2 - quote_box_dimensions.0 / 2,
        max_quote_box_position.1 + max_quote_box_dimensions.1 / 2 - quote_box_dimensions.1 / 2,
    );

    let quote_text_position = (
        quote_box_position.0 + quote_padding_size,
        quote_box_position.1 + quote_padding_size,
    );

    let quote_box = RgbaImage::from_pixel(
        quote_box_dimensions.0,
        quote_box_dimensions.1,
        Rgba([255, 255, 255, (255.0 * TEXT_BOX_OPACITY) as u8]),
    );
    imageops::overlay(
        &mut image,
        &quote_box,
        quote_box_position.0 as i64,
        quote_box_position.1 as i64,
    );

    imageops::overlay(
        &mut image,
        &quote_text,
        quote_text_position.0 as i64,
        quote_text_position.1 as i64,
    );

    image.convert()
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

fn render_quote_text(
    quote: &str,
    color: &impl Pixel<Subpixel = u8>,
    max_dimensions: (u32, u32),
) -> RgbaImage {
    let color = color.to_rgba();

    let quote = {
        let mut quote = String::from(quote);

        quote = quote.replace('\n', "");

        if quote.starts_with('\'') || quote.starts_with('"') || quote.starts_with('\u{201C}') {
            quote.remove(0);
        }
        if quote.ends_with('\'') || quote.ends_with('"') || quote.ends_with('\u{201D}') {
            quote.remove(quote.len() - 1);
        }

        quote.insert(0, '\u{201C}');
        quote.insert(quote.len(), '\u{201D}');

        quote
    };

    let font = Lato::bold();

    let height = {
        let height_max_dimensions =
            drawing::text_size(Scale::uniform(max_dimensions.1 as f32), &font, &quote);

        let width_max_scale_factor = max_dimensions.0 as f64 / height_max_dimensions.0 as f64;
        let width_max_height = (height_max_dimensions.1 as f64 * width_max_scale_factor) as u32;

        cmp::min_by_key(max_dimensions.1, width_max_height, |scale| {
            let dimensions = drawing::text_size(Scale::uniform(*scale as f32), &font, &quote);

            dimensions.1
        })
    };
    let scale = Scale::uniform(height as f32);
    let dimensions = (drawing::text_size(scale, &font, &quote).0 as u32, height);

    let mut image = RgbaImage::new(dimensions.0, dimensions.1);
    drawing::draw_text_mut(&mut image, color, 0, 0, scale, &font, &quote);

    image
}
