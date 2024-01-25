use crate::{
    commands::{
        booru,
        booru::{
            autocomplete_tag, blacklist,
            blacklist::{BigID, ID},
            SettingKind,
        },
    },
    responses::{emojis::OsakaMoji, markdown::mono, templates::cool_text},
    OsakaContext, OsakaData, OsakaResult,
};
use poise::command;
use poise_i18n::PoiseI18NTrait;
use rusty18n::{t, I18NAccessible};

#[command(slash_command)]
pub async fn add(
    ctx: OsakaContext<'_>,
    kind: SettingKind,
    #[autocomplete = "autocomplete_tag"] tag: String,
) -> OsakaResult {
    let tag = tag.trim().to_lowercase();
    blacklist::check_permissions(ctx, kind).await?;

    let OsakaData { pool, .. } = ctx.data();
    let i18n = ctx.i18n();

    let mut tx = pool.begin().await?;

    let owner_id = booru::get_owner_insert_option(ctx, kind)?;
    let [inserted_guild, inserted_channel, inserted_user] =
        booru::get_all_owner_insert_options_some_owner(ctx, kind)?;

    sqlx_conditional_queries::conditional_query_as!(
        BigID,
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

    let booru_setting_insertion = sqlx_conditional_queries::conditional_query_as!(
        ID,
        "
        INSERT INTO booru_setting AS s (guild_id, channel_id, user_id)
        VALUES ({inserted_guild}, {inserted_channel}, {inserted_user})
        ON CONFLICT ({#id_kind}_id) DO UPDATE SET id=s.id RETURNING id 
        ",
        #(id_kind) = match kind {
            SettingKind::Guild => "guild",
            SettingKind::Channel => "channel",
            SettingKind::User => "user"
        }
    )
    .fetch_one(&mut *tx)
    .await?;

    let all_blacklisted_tags = tag.split(' ');
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

    tx.commit().await?;

    ctx.say(cool_text(
        OsakaMoji::ZanyFace,
        &t!(i18n.booru.blacklist.blacklisted).access(mono(tag)),
    ))
    .await?;

    Ok(())
}
