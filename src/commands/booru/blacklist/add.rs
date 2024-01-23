use crate::commands::booru::autocomplete_tag;
use crate::commands::booru::blacklist::try_begin_blacklist_storing;
use crate::responses::emojis::OsakaMoji;
use crate::responses::markdown::mono;
use crate::responses::templates::cool_text;
use crate::{commands::booru::SettingKind, OsakaContext, OsakaResult};
use poise::command;
use poise_i18n::PoiseI18NTrait;
use rusty18n::t;
use rusty18n::I18NAccessible;

#[command(slash_command)]
pub async fn add(
    ctx: OsakaContext<'_>,
    kind: SettingKind,
    #[autocomplete = "autocomplete_tag"] tag: String,
) -> OsakaResult {
    let i18n = ctx.i18n();
    let (inserted_setting_id, mut tx) = try_begin_blacklist_storing(ctx, kind).await?;

    let all_blacklisted_tags = tag.split(' ').map(str::trim).map(str::to_lowercase);
    for blacklisted_tag in all_blacklisted_tags {
        sqlx::query!(
            "
            INSERT INTO booru_blacklisted_tag
            (booru_setting_id, blacklisted) VALUES ($1, $2)
            ",
            inserted_setting_id,
            blacklisted_tag
        )
        .execute(&mut *tx)
        .await?;
    }

    ctx.say(cool_text(
        OsakaMoji::ZanyFace,
        &t!(i18n.booru.blacklist.blacklisted).access(mono(tag)),
    ))
    .await?;

    Ok(())
}
