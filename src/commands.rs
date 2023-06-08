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

use serenity::framework::StandardFramework;

mod general;
mod help;

pub const COMMAND_PREFIX: &str = "!";

pub fn framework() -> StandardFramework {
    StandardFramework::new()
        .configure(|cfg| cfg.prefix(COMMAND_PREFIX))
        .group(&general::GENERAL_GROUP)
        .help(&help::HELP)
}
