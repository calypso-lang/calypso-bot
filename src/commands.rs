mod prelude {
    pub use serenity::framework::standard::{
        help_commands,
        macros::{check, command, group, help},
        Args, CommandError, CommandGroup, CommandOptions, CommandResult, HelpOptions, Reason,
    };
    pub use serenity::model::prelude::*;
    pub use serenity::prelude::*;
    pub use serenity::utils::Color;

    pub use crate::embeds::{
        begin, begin_error, begin_with_color, build_cleanup, embed_color, embed_color_error,
    };

    pub use std::collections::HashSet;
}

use prelude::*;

pub mod owner;
pub use owner::*;

pub mod general;
pub use general::*;

pub mod sysf;
pub use sysf::*;

use crate::data::OwnersContainer;

#[help]
#[individual_command_tip = "If you want more information about a specific command, just pass the command as an argument."]
#[command_not_found_text = "Could not find command `{}`."]
#[max_levenshtein_distance(3)]
#[strikethrough_commands_tip_in_dm = "~~`Strikethrough commands`~~ are unavailable because they are only accessible in a server."]
#[strikethrough_commands_tip_in_guild = "~~`Strikethrough commands`~~ are unavailable because they are only acessible in DMs with the bot."]
#[lacking_permissions = "Hide"]
#[lacking_ownership = "Hide"]
#[lacking_conditions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let mut options = help_options.clone();
    options.embed_success_colour = Color::new(embed_color());
    options.embed_error_colour = Color::new(embed_color_error());
    help_commands::with_embeds(context, msg, args, &options, groups, owners).await;
    Ok(())
}

pub async fn is_owner(ctx: &Context, user_id: UserId) -> bool {
    let data = ctx.data.read().await;
    let owners = data.get::<OwnersContainer>().expect("get OwnersContainer");
    owners.contains(&user_id)
}
