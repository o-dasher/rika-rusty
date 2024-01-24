use crate::commands::booru;
use crate::commands::booru::autocomplete_tag;
use crate::commands::booru::blacklist;
use crate::commands::booru::blacklist::ID;
use crate::commands::booru::SettingKind;
use crate::responses::emojis::OsakaMoji;
use crate::responses::markdown::mono;
use crate::responses::templates::cool_text;
use crate::OsakaData;
use crate::{OsakaContext, OsakaResult};
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
    blacklist::check_permissions(ctx, kind).await?;

    let OsakaData { pool, .. } = ctx.data();
    let i18n = ctx.i18n();

    let mut tx = pool.begin().await?;

    let owner_id = booru::get_owner_insert_option(ctx, kind)?;
    let [inserted_guild, inserted_channel, inserted_user] =
        booru::get_all_owner_insert_options_some_owner(ctx, kind)?;

    sqlx_conditional_queries::conditional_query_as!(
        ID,
        "
        INSERT INTO discord_{#id_kind} (id{#args}) VALUES ({owner_id}{#binds})
        ON CONFLICT (id) DO UPDATE SET id={owner_id} RETURNING id
        ",
        #(id_kind, args, binds) = match kind {
            SettingKind::Guild => ("guild", "", ""),
            SettingKind::Channel => ("channel", ", guild_id", ", {inserted_guild}"),
            SettingKind::User => ("user", "", "")
        },
    )
    .fetch_one(&mut *tx)
    .await?;

    let booru_setting_insertion = sqlx::query!(
        "
        INSERT INTO booru_setting AS s (guild_id, channel_id, user_id) VALUES ($1, $2, $3)
        ON CONFLICT (id) DO UPDATE SET id=s.id RETURNING id 
        ",
        inserted_guild,
        inserted_channel,
        inserted_user
    )
    .fetch_one(&mut *tx)
    .await?;

    let all_blacklisted_tags = tag.split(' ').map(str::trim).map(str::to_lowercase);
    for blacklisted_tag in all_blacklisted_tags {
        sqlx::query!(
            "
            INSERT INTO booru_blacklisted_tag
            (booru_setting_id, blacklisted) VALUES ($1, $2)
            ",
            booru_setting_insertion.id,
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
