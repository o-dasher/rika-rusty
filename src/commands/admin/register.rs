use crate::{
    default_args,
    managers::register_command_manager::{RegisterCommandManager, RegisterError, RegisterKind},
    OsakaContext, OsakaResult,
};
use poise::{command, ChoiceParameter};

#[derive(ChoiceParameter, Default)]
enum RegisterChoice {
    #[default]
    Development,
    Local,
    Global,
}

#[command(slash_command)]
pub async fn register(ctx: OsakaContext<'_>, on: Option<RegisterChoice>) -> OsakaResult {
    default_args!(on);

    RegisterCommandManager::register_commands(
        ctx.serenity_context(),
        &ctx.data().config,
        &ctx.framework().options().commands,
        match on {
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
