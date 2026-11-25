use serenity::{
    all::{
        ComponentInteraction, CreateInteractionResponse, CreateInteractionResponseMessage,
        EditInteractionResponse, Interaction,
    },
    async_trait,
    client::EventHandler,
    http::Http,
    prelude::Context,
};

use crate::roles::ROLES;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Component(component) = interaction else {
            tracing::trace!("Ignoring non-component interaction");
            return;
        };

        tracing::info!(
            custom_id = component.data.custom_id,
            user = %component.user.name,
            user_id = %component.user.id,
            "User requested a role change"
        );

        let Some(role) = ROLES
            .iter()
            .find(|role| role.custom_id == component.data.custom_id)
        else {
            tracing::warn!(
                custom_id = component.data.custom_id,
                user = %component.user.name,
                user_id = %component.user.id,
                "Received interaction for unknown role button"
            );
            return;
        };

        let Some(guild_id) = component.guild_id else {
            tracing::warn!(
                custom_id = component.data.custom_id,
                user = %component.user.name,
                user_id = %component.user.id,
                "Received role interaction outside of a guild"
            );
            return;
        };

        // Acknowledge interaction immediately.
        if let Err(error) = component
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Defer(
                    CreateInteractionResponseMessage::new().ephemeral(true),
                ),
            )
            .await
        {
            tracing::error!("Failed to acknowledge interaction: {error}");
            return;
        }

        let member = match guild_id.member(&ctx.http, component.user.id).await {
            Ok(member) => member,
            Err(error) => {
                tracing::error!("Failed to fetch member: {error}");
                respond(
                    &ctx.http,
                    &component,
                    "Не удалось получить данные участника. Попробуй позже.",
                )
                .await;
                return;
            }
        };

        let has_role = member.roles.contains(&role.role_id);

        let result = if has_role {
            ctx.http
                .remove_member_role(
                    guild_id,
                    component.user.id,
                    role.role_id,
                    Some("Self-removed role"),
                )
                .await
        } else {
            ctx.http
                .add_member_role(
                    guild_id,
                    component.user.id,
                    role.role_id,
                    Some("Self-assigned role"),
                )
                .await
        };

        let response = match result {
            Ok(()) if has_role => {
                tracing::info!(
                    user = %component.user.name,
                    user_id = %component.user.id,
                    guild_id = %guild_id,
                    role = role.label,
                    role_id = %role.role_id,
                    action = "removed",
                    "Role changed successfully"
                );

                format!("Роль **{}** снята!", role.label)
            }
            Ok(()) => {
                tracing::info!(
                    user = %component.user.name,
                    user_id = %component.user.id,
                    guild_id = %guild_id,
                    role = role.label,
                    role_id = %role.role_id,
                    action = "added",
                    "Role changed successfully"
                );

                format!("Роль **{}** успешно выдана!", role.label)
            }
            Err(error) => {
                tracing::error!("Failed to toggle role: {error}");

                "Не удалось изменить роль. Возможно, у бота недостаточно прав.".to_string()
            }
        };

        respond(&ctx.http, &component, response).await;
    }
}

async fn respond(http: &Http, component: &ComponentInteraction, content: impl Into<String>) {
    if let Err(error) = component
        .edit_response(http, EditInteractionResponse::new().content(content))
        .await
    {
        tracing::error!("Failed to edit interaction response: {error}");
    }
}
