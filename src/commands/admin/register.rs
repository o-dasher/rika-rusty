use crate::{
    managers::register_command_manager::{RegisterCommandManager, RegisterError, RegisterKind},
    OsakaContext, OsakaResult,
};
use poise::{command, ChoiceParameter};
use poise_default_slash_argument::DefaultSlash;

#[derive(ChoiceParameter, Default)]
enum RegisterChoice {
    #[default]
    Development,
    Local,
    Global,
}

#[command(slash_command)]
pub async fn register(ctx: OsakaContext<'_>, on: DefaultSlash<RegisterChoice>) -> OsakaResult {
    RegisterCommandManager::register_commands(
        ctx.serenity_context(),
        &ctx.data().config,
        &ctx.framework().options().commands,
        match on.0 {
            RegisterChoice::Development => RegisterKind::Development,
            RegisterChoice::Local => match ctx.guild_id() {
                Some(guild_id) => RegisterKind::Local(guild_id),
                None => Err(RegisterError::NoDevelopmentGuildSet)?,
            },
            RegisterChoice::Global => RegisterKind::Global,
        },
    )
    .await
}
