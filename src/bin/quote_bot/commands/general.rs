use std::{env, io::Cursor};

use anyhow::Context as _;
use chrono::Utc;
use image::ImageOutputFormat;
use quote_bot::QuoteRenderer;
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::prelude::Message,
    prelude::*,
};

#[group]
#[commands(ping, test)]
struct General;

#[command]
#[description("Ping pong!")]
#[num_args(0)]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    instrument_command!("ping", msg, {
        msg.reply(ctx, "Pong!").await?;

        Ok(())
    })
}

#[command]
async fn test(ctx: &Context, msg: &Message) -> CommandResult {
    instrument_command!("test", msg, {
        let _typing = msg.channel_id.start_typing(&ctx.http)?;

        let unsplash_access_key = env::var("UNSPLASH_KEY")
            .context("Failed to load `UNSPLASH_KEY` environment variable")?;

        let renderer = QuoteRenderer::new(&unsplash_access_key);

        let image = renderer
            .render("test quote", "test author", Utc::now())
            .await?;

        let mut image_bytes: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        image.write_to(&mut image_bytes, ImageOutputFormat::Jpeg(75))?;
        let image_bytes = image_bytes.into_inner();

        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.add_file((image_bytes.as_slice(), "quote.jpg"))
            })
            .await?;

        Ok(())
    })
}

// #[command]
// #[description("Echoes your message back to you.")]
// #[usage("<message>")]
// async fn echo(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
//     instrument_command!("echo", msg, {
//         args.trimmed().quoted();

//         let reply_content = args.remains().unwrap_or("*(silence)*");

//         msg.reply(ctx, reply_content).await?;

//         Ok(())
//     })
// }

// #[command]
// #[description("Says hello!")]
// #[usage("[name='world']")]
// #[example("Wumpus")]
// #[max_args(1)]
// async fn greet(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
//     instrument_command!("greet", msg, {
//         args.trimmed().quoted();

//         let name = args.single::<String>().unwrap_or(String::from("world"));
//         let reply_content = format!("Hello {name}!");

//         msg.reply(ctx, reply_content).await?;

//         Ok(())
//     })
// }
