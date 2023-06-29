macro_rules! instrument_command {
    ($name:expr, $msg:ident, $body:block) => {{
        use serenity::model::prelude::{ChannelId, MessageId};
        use tracing::Instrument;

        async move { $body }
            .instrument(error_span!(
                $name,
                msg_id = <u64 as From<MessageId>>::from($msg.id),
                channel_id = <u64 as From<ChannelId>>::from($msg.channel_id)
            ))
            .await
    }};
}

use std::collections::HashSet;

use serenity::{
    framework::{
        standard::{buckets::LimitedFor, macros::hook, CommandResult, DispatchError},
        StandardFramework,
    },
    model::prelude::{Message, UserId},
    prelude::*,
};

mod general;
mod help;

pub const COMMAND_PREFIX: &str = "q!";

pub async fn framework(owners: HashSet<UserId>) -> StandardFramework {
    StandardFramework::new()
        .configure(|cfg| cfg.prefix(COMMAND_PREFIX).owners(owners))
        .group(&general::GENERAL_GROUP)
        .help(&help::HELP)
        .on_dispatch_error(dispatch_error_hook)
        .after(after_hook)
        .bucket("unsplash", |b| {
            b.limit_for(LimitedFor::Global).time_span(3600).limit(50)
        })
        .await
}

#[hook]
async fn after_hook(_ctx: &Context, _msg: &Message, command_name: &str, result: CommandResult) {
    if let Err(err) = result {
        error!(command = command_name, "Command error occurred: {err:?}");
    }
}

#[hook]
async fn dispatch_error_hook(
    ctx: &Context,
    msg: &Message,
    error: DispatchError,
    command_name: &str,
) {
    match error {
        DispatchError::NotEnoughArguments { min, given } => {
            let reply_content = format!(
                "`{COMMAND_PREFIX}{command_name}` requires {min} arguments, but only received {given}."
            );
            let reply = msg.reply_ping(ctx, reply_content).await;

            if reply.is_err() {
                error!(
                    command = command_name,
                    "Unhandled dispatch error: {error:?}",
                )
            }
        }
        DispatchError::TooManyArguments { max, given } => {
            let reply_content = format!(
                "`{COMMAND_PREFIX}{command_name}` only accepts {max} arguments, but received {given}."
            );
            let reply = msg.reply_ping(ctx, reply_content).await;

            if reply.is_err() {
                error!(
                    command = command_name,
                    "Unhandled dispatch error: {error:?}"
                )
            }
        }
        DispatchError::OnlyForDM => {
            let reply_content =
                format!("`{COMMAND_PREFIX}{command_name}` can only be used in DMs.");
            let reply = msg.reply_ping(ctx, reply_content).await;

            if reply.is_err() {
                error!(
                    command = command_name,
                    "Unhandled dispatch error: {error:?}"
                )
            }
        }
        DispatchError::OnlyForGuilds => {
            let reply_content =
                format!("`{COMMAND_PREFIX}{command_name}` can only be used in servers.");
            let reply = msg.reply_ping(ctx, reply_content).await;

            if reply.is_err() {
                error!(
                    command = command_name,
                    "Unhandled dispatch error: {error:?}"
                )
            }
        }
        DispatchError::OnlyForOwners
        | DispatchError::LackingRole
        | DispatchError::LackingPermissions(_) => {
            let reply_content =
                format!("You don't have permission to use `{COMMAND_PREFIX}{command_name}`.");
            let reply = msg.reply_ping(ctx, reply_content).await;

            if reply.is_err() {
                error!(
                    command = command_name,
                    "Unhandled dispatch error: {error:?}"
                )
            }
        }
        DispatchError::Ratelimited(_) => {
            let reply = msg
                .reply_ping(ctx, "Rate limit reached, please try again soon.")
                .await;

            if reply.is_err() {
                error!(
                    command = command_name,
                    "Unhandled dispatch error: {error:?}"
                )
            }
        }
        DispatchError::BlockedUser
        | DispatchError::BlockedGuild
        | DispatchError::BlockedChannel => {}
        _ => error!(
            command = command_name,
            "Unhandled dispatch error: {error:?}"
        ),
    }
}
