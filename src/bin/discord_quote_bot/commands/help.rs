use std::collections::HashSet;

use serenity::{
    framework::standard::{
        help_commands, macros::help, Args, CommandGroup, CommandResult, HelpOptions,
    },
    model::prelude::{Message, UserId},
    prelude::*,
};

#[help]
#[usage_sample_label("Example(s)")]
#[checks_label("")]
#[available_text("")]
#[strikethrough_commands_tip_in_dm("")]
#[strikethrough_commands_tip_in_guild("")]
#[lacking_role("strike")]
#[lacking_ownership("hide")]
#[lacking_permissions("strike")]
#[lacking_conditions("strike")]
#[wrong_channel("strike")]
#[embed_success_colour("#5865F2")]
#[embed_error_colour("#5865F2")]
#[max_levenshtein_distance(3)]
async fn help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    instrument_command!("help", msg, {
        help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await?;

        Ok(())
    })
}
