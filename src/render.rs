use std::cmp;

use chrono::NaiveDate;
use image::{buffer::ConvertBuffer, imageops, Pixel, Rgb, RgbImage, Rgba, RgbaImage};
use imageproc::drawing;
use rusttype::{Font, Scale};

use crate::assets::fonts::Lato;

pub fn render(
    background_image: &RgbImage,
    quote: &str,
    author: &str,
    timestamp: NaiveDate,
) -> RgbaImage {
    let mut image: RgbaImage = background_image.clone().convert();
    let dimensions = image.dimensions();

    let average_color = calculate_average_color(background_image);

    const TEXT_BOX_OPACITY: f64 = 0.6;
    let text_box_color = Rgba([255, 255, 255, (255.0 * TEXT_BOX_OPACITY) as u8]);

    const MARGIN_MULTIPLIER: f64 = 0.2;
    let margin_size = (dimensions.1 as f64 * MARGIN_MULTIPLIER) as u32;

    const QUOTE_PADDING_MULTIPLIER: f64 = 0.05;
    let quote_padding_size = (dimensions.1 as f64 * QUOTE_PADDING_MULTIPLIER) as u32;

    const QUOTE_BOX_HEIGHT_MULTIPLIER: f64 = 3.0 / 4.0;

    let max_quote_box_dimensions = (
        dimensions.0 - margin_size * 2,
        ((dimensions.1 - margin_size * 2) as f64 * QUOTE_BOX_HEIGHT_MULTIPLIER) as u32,
    );
    let max_quote_box_position = (margin_size, margin_size);

    let quote_box = render_quote_box(
        quote,
        &average_color,
        &text_box_color,
        max_quote_box_dimensions,
        quote_padding_size,
    );
    imageops::overlay(
        &mut image,
        &quote_box,
        max_quote_box_position.0 as i64,
        max_quote_box_position.1 as i64,
    );

    const BOX_GAP_MULTIPLIER: f64 = 0.025;
    let box_gap_size = (dimensions.1 as f64 * BOX_GAP_MULTIPLIER) as u32;

    const ATTRIBUTION_PADDING_MULTIPLIER: f64 = 0.025;
    let attribution_padding_size = (dimensions.1 as f64 * ATTRIBUTION_PADDING_MULTIPLIER) as u32;

    const ATTRIBUTION_BOX_HEIGHT_MULTIPLIER: f64 = 1.0 / 7.0;

    let max_attribution_box_dimensions = (
        max_quote_box_dimensions.0,
        ((dimensions.1 - margin_size * 2) as f64 * ATTRIBUTION_BOX_HEIGHT_MULTIPLIER) as u32,
    );
    let max_attribution_box_position = (
        max_quote_box_position.0,
        (max_quote_box_position.1 + max_quote_box_dimensions.1) + box_gap_size,
    );

    let attribution_box = render_attribution_box(
        author,
        timestamp,
        &average_color,
        &text_box_color,
        max_attribution_box_dimensions,
        attribution_padding_size,
    );
    imageops::overlay(
        &mut image,
        &attribution_box,
        max_attribution_box_position.0 as i64,
        max_attribution_box_position.1 as i64,
    );

    image
}

fn render_quote_box(
    quote: &str,
    text_color: &impl Pixel<Subpixel = u8>,
    text_box_color: &impl Pixel<Subpixel = u8>,
    max_dimensions: (u32, u32),
    padding_size: u32,
) -> RgbaImage {
    let max_quote_text_dimensions = (
        max_dimensions.0 - padding_size * 2,
        max_dimensions.1 - padding_size * 2,
    );

    let quote_text = render_quote_text(quote, text_color, max_quote_text_dimensions);
    let quote_text_dimensions = quote_text.dimensions();

    let quote_box_dimensions = (
        quote_text_dimensions.0 + padding_size * 2,
        quote_text_dimensions.1 + padding_size * 2,
    );
    // center-aligned within max quote box
    let quote_box_position = (
        max_dimensions.0 / 2 - quote_box_dimensions.0 / 2,
        max_dimensions.1 / 2 - quote_box_dimensions.1 / 2,
    );

    let quote_text_position = (padding_size, padding_size);

    let mut quote_box = RgbaImage::from_pixel(
        quote_box_dimensions.0,
        quote_box_dimensions.1,
        text_box_color.to_rgba(),
    );
    imageops::overlay(
        &mut quote_box,
        &quote_text,
        quote_text_position.0 as i64,
        quote_text_position.1 as i64,
    );

    let mut image = RgbaImage::new(max_dimensions.0, max_dimensions.1);
    imageops::overlay(
        &mut image,
        &quote_box,
        quote_box_position.0 as i64,
        quote_box_position.1 as i64,
    );

    image
}

fn render_quote_text(
    quote: &str,
    color: &impl Pixel<Subpixel = u8>,
    max_dimensions: (u32, u32),
) -> RgbaImage {
    let color = color.to_rgba();

    let font = Lato::bold();

    const MAX_LINE_COUNT: u32 = 5;
    const LINE_HEIGHT_MULTIPLIER: f64 = 1.3;

    let min_line_height =
        (max_dimensions.1 as f64 / (MAX_LINE_COUNT as f64 * LINE_HEIGHT_MULTIPLIER)) as u32;
    let min_line_gap =
        ((min_line_height as f64 / LINE_HEIGHT_MULTIPLIER) * (LINE_HEIGHT_MULTIPLIER - 1.0)) as u32;

    let min_scale = Scale::uniform((min_line_height - min_line_gap) as f32);

    let quote = {
        let mut quote = String::from(quote.trim());

        quote = quote.lines().collect::<Vec<&str>>().join("");

        if quote.starts_with('\'') || quote.starts_with('"') || quote.starts_with('\u{201C}') {
            quote.remove(0);
        }
        if quote.ends_with('\'') || quote.ends_with('"') || quote.ends_with('\u{201D}') {
            quote.pop();
        }

        quote.insert(0, '\u{201C}');
        quote.push('\u{201D}');

        quote = wrap_text(&quote, &font, min_scale, max_dimensions.0);

        let mut quote_lines: Vec<&str> = quote.lines().collect();
        if quote_lines.len() > MAX_LINE_COUNT as usize {
            quote_lines = quote_lines.into_iter().take(5).collect();
            quote = quote_lines.join("\n");
            quote.pop();
            quote.push('\u{2026}');
        }

        quote
    };

    let line_count: u32 = quote.lines().count() as u32;

    if line_count == 0 {
        panic!("wrapped quote should be made up of at least one line");
    }

    let (height, scale) = if line_count == 1 {
        let height = {
            let height_max_dimensions =
                drawing::text_size(Scale::uniform(max_dimensions.1 as f32), &font, &quote);

            let width_max_scale_factor = max_dimensions.0 as f64 / height_max_dimensions.0 as f64;
            let width_max_height = (height_max_dimensions.1 as f64 * width_max_scale_factor) as u32;

            cmp::min_by_key(max_dimensions.1, width_max_height, |scale| {
                let dimensions = drawing::text_size(Scale::uniform(*scale as f32), &font, &quote);

                dimensions.1 as u32
            })
        };
        let scale = Scale::uniform(height as f32);

        (height, scale)
    } else {
        let height = (min_line_height * line_count) - min_line_gap;

        (height, min_scale)
    };

    let max_line_width = quote
        .lines()
        .map(|line| {
            let dimensions = drawing::text_size(scale, &font, line);

            dimensions.0 as u32
        })
        .max()
        .expect("wrapped quote should be made up of at least one line");
    let dimensions = (max_line_width, height);

    let mut image = RgbaImage::new(dimensions.0, dimensions.1);
    for (line_index, line) in quote.lines().enumerate() {
        let line_width = drawing::text_size(scale, &font, line).0 as u32;
        let line_position = (
            dimensions.0 / 2 - line_width / 2,
            min_line_height * line_index as u32,
        );

        drawing::draw_text_mut(
            &mut image,
            color,
            line_position.0 as i32,
            line_position.1 as i32,
            scale,
            &font,
            line,
        );
    }

    image
}

fn render_attribution_box(
    author: &str,
    timestamp: NaiveDate,
    text_color: &impl Pixel<Subpixel = u8>,
    text_box_color: &impl Pixel<Subpixel = u8>,
    max_dimensions: (u32, u32),
    padding_size: u32,
) -> RgbaImage {
    let max_attribution_text_dimensions = (
        max_dimensions.0 - padding_size * 2,
        max_dimensions.1 - padding_size * 2,
    );

    let attribution_text = render_attribution_text(
        author,
        timestamp,
        text_color,
        max_attribution_text_dimensions,
    );
    let attribution_text_dimensions = attribution_text.dimensions();

    let attribution_box_dimensions = (
        attribution_text_dimensions.0 + padding_size * 2,
        attribution_text_dimensions.1 + padding_size * 2,
    );
    // center-aligned within max attribution box
    let attribution_box_position = (
        max_dimensions.0 / 2 - attribution_box_dimensions.0 / 2,
        max_dimensions.1 / 2 - attribution_box_dimensions.1 / 2,
    );

    let attribution_text_position = (padding_size, padding_size);

    let mut attribution_box = RgbaImage::from_pixel(
        attribution_box_dimensions.0,
        attribution_box_dimensions.1,
        text_box_color.to_rgba(),
    );
    imageops::overlay(
        &mut attribution_box,
        &attribution_text,
        attribution_text_position.0 as i64,
        attribution_text_position.1 as i64,
    );

    let mut image = RgbaImage::new(max_dimensions.0, max_dimensions.1);
    imageops::overlay(
        &mut image,
        &attribution_box,
        attribution_box_position.0 as i64,
        attribution_box_position.1 as i64,
    );

    image
}

fn render_attribution_text(
    author: &str,
    timestamp: NaiveDate,
    color: &impl Pixel<Subpixel = u8>,
    max_dimensions: (u32, u32),
) -> RgbaImage {
    let color = color.to_rgba();

    let font = Lato::semibold_italic();

    const TIMESTAMP_FORMAT: &str = "%d/%m/%Y";
    let attribution = {
        let mut author = String::from(author.trim());

        author = author.lines().collect::<Vec<&str>>().join("");

        if author.starts_with('-') {
            author.remove(0);
            author = String::from(author.trim_start());
        }

        format!("{}, {}", author, timestamp.format(TIMESTAMP_FORMAT))
    };

    let height = {
        let height_max_dimensions =
            drawing::text_size(Scale::uniform(max_dimensions.1 as f32), &font, &attribution);

        let width_max_scale_factor = max_dimensions.0 as f64 / height_max_dimensions.0 as f64;
        let width_max_height = (height_max_dimensions.1 as f64 * width_max_scale_factor) as u32;

        cmp::min_by_key(max_dimensions.1, width_max_height, |scale| {
            let dimensions = drawing::text_size(Scale::uniform(*scale as f32), &font, &attribution);

            dimensions.1 as u32
        })
    };
    let scale = Scale::uniform(height as f32);

    let dimensions = (
        drawing::text_size(scale, &font, &attribution).0 as u32,
        height,
    );

    let mut image = RgbaImage::new(dimensions.0, dimensions.1);
    drawing::draw_text_mut(&mut image, color, 0, 0, scale, &font, &attribution);

    image
}

fn wrap_text(text: &str, font: &Font, scale: Scale, max_width: u32) -> String {
    let text = String::from(text);

    if drawing::text_size(scale, font, &text).0 as u32 <= max_width {
        text
    } else {
        let mut last_fitting_whitespace_index: Option<usize> = None;
        let mut current_text = String::with_capacity(text.capacity());
        for (index, character) in text.chars().enumerate() {
            current_text.push(character);

            if drawing::text_size(scale, font, &current_text).0 as u32 <= max_width {
                if character.is_whitespace() {
                    last_fitting_whitespace_index = Some(index);
                }
            } else {
                if character.is_whitespace() || last_fitting_whitespace_index.is_none() {
                    last_fitting_whitespace_index = Some(index);
                }

                break;
            }
        }

        let last_fitting_whitespace_index = last_fitting_whitespace_index
            .expect("last_fitting_whitespace_index should not be None after for loop");

        let current_line: String = text
            .chars()
            .take(last_fitting_whitespace_index + 1)
            .collect();
        let remaining_text: String = text
            .chars()
            .skip(last_fitting_whitespace_index + 1)
            .collect();

        format!(
            "{}\n{}",
            current_line,
            wrap_text(&remaining_text, font, scale, max_width)
        )
    }
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
