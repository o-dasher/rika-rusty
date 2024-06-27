use crate::{
    default_args,
    error::NotifyError,
    managers::{self, register_command},
    responses::{emojis::OsakaMoji, templates::cool_text},
    OsakaContext, OsakaData, OsakaResult,
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

    let OsakaData { managers, .. } = ctx.data().as_ref();
    let managers::Osaka {
        register_command_manager,
        ..
    } = managers.as_ref();

    register_command_manager
        .register_commands(
            managers::register_command::Context::Poise(&ctx),
            match on {
                RegisterChoice::Development => register_command::Kind::Development,
                RegisterChoice::Local => match ctx.guild_id() {
                    Some(guild_id) => register_command::Kind::Local(guild_id),
                    None => Err(NotifyError::Warn(
                        t!(i18n.errors.must_be_used_on_guild).clone(),
                    ))?,
                },
                RegisterChoice::Global => register_command::Kind::Global,
            },
        )
        .await?;

    ctx.reply(cool_text(
        OsakaMoji::ZanyFace,
        t!(i18n.owner.register.success).as_ref(),
    ))
    .await?;

    Ok(())
}
