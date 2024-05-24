use crate::{OsakaContext, OsakaResult};
use poise::{
    command,
    serenity_prelude::{self, Colour},
};
use poise_i18n::PoiseI18NTrait;
use rusty18n::{t_prefix};

#[command(slash_command)]
pub async fn avatar(ctx: OsakaContext<'_>, user: Option<serenity_prelude::User>) -> OsakaResult {
    let i18n = ctx.i18n();
    t_prefix!($, i18n.user.avatar);

    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let avatar = u.avatar_url().unwrap_or_default();

    let footer = if u == ctx.author() {
        t!(footer.eq)
    } else {
        t!(footer.other)
    };

    ctx.send(|r| {
        r.embed(|e| {
            e.color(Colour::DARK_BLUE)
                .image(avatar)
                .footer(|f| f.text(footer))
                .title(u.name.clone())
        })
    })
    .await?;

    Ok(())
}
