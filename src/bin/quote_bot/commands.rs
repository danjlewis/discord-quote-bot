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
        standard::{buckets::LimitedFor, macros::hook, DispatchError},
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
        .bucket("unsplash", |b| {
            b.limit_for(LimitedFor::Global)
                .time_span(86400)
                .limit(50)
                .delay_action(unsplash_limit_hook)
        })
        .await
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
                error!("Unhandled dispatch error in {}: {:?}", command_name, error)
            }
        }
        DispatchError::TooManyArguments { max, given } => {
            let reply_content = format!(
                "`{COMMAND_PREFIX}{command_name}` only accepts {max} arguments, but received {given}."
            );
            let reply = msg.reply_ping(ctx, reply_content).await;

            if reply.is_err() {
                error!("Unhandled dispatch error in {}: {:?}", command_name, error)
            }
        }
        DispatchError::OnlyForDM => {
            let reply_content =
                format!("`{COMMAND_PREFIX}{command_name}` can only be used in DMs.");
            let reply = msg.reply_ping(ctx, reply_content).await;

            if reply.is_err() {
                error!("Unhandled dispatch error in {}: {:?}", command_name, error)
            }
        }
        DispatchError::OnlyForGuilds => {
            let reply_content =
                format!("`{COMMAND_PREFIX}{command_name}` can only be used in servers.");
            let reply = msg.reply_ping(ctx, reply_content).await;

            if reply.is_err() {
                error!("Unhandled dispatch error in {}: {:?}", command_name, error)
            }
        }
        DispatchError::OnlyForOwners
        | DispatchError::LackingRole
        | DispatchError::LackingPermissions(_) => {
            let reply_content =
                format!("You don't have permission to use `{COMMAND_PREFIX}{command_name}`.");
            let reply = msg.reply_ping(ctx, reply_content).await;

            if reply.is_err() {
                error!("Unhandled dispatch error in {}: {:?}", command_name, error)
            }
        }
        DispatchError::Ratelimited(_)
        | DispatchError::BlockedUser
        | DispatchError::BlockedGuild
        | DispatchError::BlockedChannel => {}
        _ => error!("Unhandled dispatch error in {}: {:?}", command_name, error),
    }
}

#[hook]
async fn unsplash_limit_hook(ctx: &Context, msg: &Message) {
    let reaction = msg.react(ctx, '⏱').await;
    if reaction.is_err() {
        let reply = msg
            .reply_ping(
                ctx,
                "Global daily rate limit reached! Please try again tomorrow.",
            )
            .await;

        if reply.is_err() {
            error!("Failed to send Unsplash rate limit message!");
        }
    }
}
