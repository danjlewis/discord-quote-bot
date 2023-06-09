use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::prelude::Message,
    prelude::*,
};

#[group]
#[commands(ping, echo, greet)]
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
#[description("Echoes your message back to you.")]
#[usage("<message>")]
async fn echo(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    instrument_command!("echo", msg, {
        args.trimmed().quoted();

        let reply_content = args.remains().unwrap_or("*(silence)*");

        msg.reply(ctx, reply_content).await?;

        Ok(())
    })
}

#[command]
#[description("Says hello!")]
#[usage("[name='world']")]
#[example("Wumpus")]
#[max_args(1)]
async fn greet(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    instrument_command!("greet", msg, {
        args.trimmed().quoted();

        let name = args.single::<String>().unwrap_or(String::from("world"));
        let reply_content = format!("Hello {name}!");

        msg.reply(ctx, reply_content).await?;

        Ok(())
    })
}
