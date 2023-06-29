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
        standard::{buckets::LimitedFor, macros::hook},
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
                .delay_action(unsplash_limit_action)
        })
        .await
}

#[hook]
async fn unsplash_limit_action(ctx: &Context, msg: &Message) {
    let reaction = msg.react(ctx, '⏱').await;
    if reaction.is_err() {
        let reply = msg
            .channel_id
            .send_message(ctx, |m| {
                m.content("Global daily rate limit reached! Please try again tomorrow.")
            })
            .await;

        if reply.is_err() {
            error!("Failed to send Unsplash rate limit message!");
        }
    }
}
