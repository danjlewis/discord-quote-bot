use std::{env, io::Cursor};

use anyhow::Context as _;
use chrono::{NaiveDate, Utc};
use image::ImageOutputFormat;
use quote_bot::{
    render,
    unsplash::{
        GetRandomPhotoOptions, ImgixFitMode, ImgixFormat, ImgixParams, Orientation, UnsplashClient,
    },
};
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::prelude::Message,
    prelude::*,
};

#[group]
#[commands(quote)]
struct General;

#[command]
#[description("Generates an inspirational quote image.")]
#[usage("<quote> <author> [DD/MM/YYYY]")]
#[example("\"Man, I really hope this sentence doesn't get stolen for an example quote.\" \"Some Guy I Stole From\" 29/06/2023")]
#[min_args(2)]
#[max_args(3)]
#[bucket("unsplash")]
async fn quote(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    instrument_command!("quote", msg, {
        args.trimmed().quoted();

        let quote: String = args.single()?;
        let author: String = args.single()?;

        let timestamp_raw: Option<String> = args.single().ok();
        let timestamp = match timestamp_raw {
            None => Utc::now().date_naive(),
            Some(s) => NaiveDate::parse_from_str(&s, "%d/%m/%Y")?,
        };

        let _typing = msg.channel_id.start_typing(&ctx.http)?;

        let unsplash_access_key = env::var("UNSPLASH_KEY")
            .context("failed to load `UNSPLASH_KEY` environment variable")?;
        let unsplash_client = UnsplashClient::new(&unsplash_access_key);

        let get_random_photo_options = GetRandomPhotoOptions {
            collections: Some(String::from("11649432")),
            orientation: Some(Orientation::Landscape),
            imgix_params: ImgixParams {
                height: Some(1080),
                format: Some(ImgixFormat::Jpg),
                quality: Some(45),
                fit_mode: Some(ImgixFitMode::Crop),
                aspect_ratio: Some(String::from("3:2")),
                ..Default::default()
            },
            ..Default::default()
        };

        let background_image = unsplash_client
            .get_random_photo(get_random_photo_options)
            .await
            .context("failed to get random background image")?;

        let image = render::render(&background_image, &quote, &author, timestamp);

        let mut image_bytes: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        image
            .write_to(&mut image_bytes, ImageOutputFormat::Jpeg(75))
            .context("failed to encode quote image")?;
        let image_bytes = image_bytes.into_inner();

        msg.channel_id
            .send_message(ctx, |m| {
                m.reference_message(msg)
                    .allowed_mentions(|am| am.empty_parse())
                    .add_file((image_bytes.as_slice(), "quote.jpg"))
            })
            .await
            .context("failed to send quote image")?;

        Ok(())
    })
}
