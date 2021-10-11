use serenity::{
    async_trait,
    model::{interactions::message_component::InteractionMessage, prelude::*},
    prelude::*,
};
use tracing::info;

use crate::{commands::is_owner, data::ConfigContainer};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!(
            "CalBot started on {} guilds with the user {}",
            ready.guilds.len(),
            ready.user.tag()
        );

        let data = ctx.data.read().await;
        let cfg = data.get::<ConfigContainer>().expect("get ConfigContainer");

        ctx.set_presence(Some(cfg.discord.status.clone()), OnlineStatus::Online)
            .await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::MessageComponent(interaction) = interaction {
            let mut custom_id = interaction.data.custom_id.split(';');
            let name = custom_id.next().unwrap();
            match name {
                "clean" => {
                    let orig_author = custom_id.next().unwrap().parse::<u64>().unwrap();
                    let orig_author = UserId(orig_author);

                    if let InteractionMessage::Regular(ref msg) = interaction.message {
                        if interaction.user.id == orig_author
                            || interaction
                                .member
                                .as_ref()
                                .unwrap()
                                .permissions
                                .unwrap()
                                .manage_messages()
                            || is_owner(&ctx, interaction.user.id).await
                        {
                            msg.delete(&ctx)
                                .await
                                .expect("delete message in cleanup handler");
                        } else {
                            interaction
                                .create_interaction_response(&ctx, |b| {
                                    b.interaction_response_data(|b| {
                                        b.content("You don't have permission to do this!").flags(
                                        InteractionApplicationCommandCallbackDataFlags::EPHEMERAL,
                                    )
                                    })
                                })
                                .await
                                .expect("send ephemeral in cleanup handler");
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
