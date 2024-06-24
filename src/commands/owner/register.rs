use crate::{
    default_args,
    error::NotifyError,
    managers::register_command_manager::{RegisterCommandManager, RegisterKind},
    responses::{emojis::OsakaMoji, templates::cool_text},
    OsakaContext, OsakaResult,
};
use poise::{command, ChoiceParameter};
use poise_i18n::PoiseI18NTrait;
use rusty18n::t;

#[derive(ChoiceParameter, Default)]
enum RegisterChoice {
    #[default]
    Development,
    Local,
    Global,
}

#[command(slash_command)]
pub async fn register(ctx: OsakaContext<'_>, on: Option<RegisterChoice>) -> OsakaResult {
    let i18n = ctx.i18n();
    default_args!(on);

    RegisterCommandManager::register_commands(
        ctx.serenity_context(),
        &ctx.data().config,
        &ctx.framework().options().commands,
        match on {
            RegisterChoice::Development => RegisterKind::Development,
            RegisterChoice::Local => match ctx.guild_id() {
                Some(guild_id) => RegisterKind::Local(guild_id),
                None => Err(NotifyError::Warn(
                    t!(i18n.errors.must_be_used_on_guild).clone(),
                ))?,
            },
            RegisterChoice::Global => RegisterKind::Global,
        },
    )
    .await?;

    ctx.reply(cool_text(
        OsakaMoji::ZanyFace,
        &t!(i18n.owner.register.success),
    ))
    .await?;

    Ok(())
}
