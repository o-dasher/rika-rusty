use paste::paste;

use crate::{
    commands::booru::{
        self, autocomplete_tag_single,
        blacklist::{self, BigID, ID},
        get_all_setting_kind_db_ids_only_allowing_this_kind, SettingKind,
    },
    error::NotifyError,
    responses::{emojis::OsakaMoji, markdown::mono, templates::cool_text},
    OsakaContext, OsakaData, OsakaResult,
};
use itertools::Itertools;
use poise::command;
use poise_i18n::PoiseI18NTrait;
use rusty18n::t;

#[command(slash_command)]
pub async fn add(
    ctx: OsakaContext<'_>,
    kind: SettingKind,
    #[autocomplete = "autocomplete_tag_single"] tag: String,
) -> OsakaResult {
    let tag = tag.trim().to_lowercase();
    blacklist::check_permissions(ctx, kind).await?;

    let OsakaData { pool, .. } = ctx.data();
    let i18n = ctx.i18n();

    let mut tx = pool.begin().await?;

    let used_setting_kind = booru::get_setting_kind_db_id(ctx, kind)?;

    sqlx_conditional_queries_layering::create_conditional_query_as!(
        $conditional_id_kind_query,
        #id_kind = match kind {
            SettingKind::Guild => "guild",
            SettingKind::Channel => "channel",
            SettingKind::User => "user"
        }
    );

    conditional_id_kind_query!(
        BigID,
        "
        INSERT INTO discord_{#id_kind} (id) VALUES ({used_setting_kind})
        ON CONFLICT (id) DO UPDATE SET id={used_setting_kind} RETURNING id
        "
    )
    .fetch_one(&mut *tx)
    .await?;

    let [inserted_guild, inserted_channel, inserted_user] =
        get_all_setting_kind_db_ids_only_allowing_this_kind(ctx, kind)?;

    let booru_setting_insertion = conditional_id_kind_query!(
        ID,
        "
        INSERT INTO booru_setting AS s (guild_id, channel_id, user_id)
        VALUES ({inserted_guild}, {inserted_channel}, {inserted_user})
        ON CONFLICT ({#id_kind}_id) DO UPDATE SET id=s.id RETURNING id 
        "
    )
    .fetch_one(&mut *tx)
    .await?;

    let split_tags = tag.split(' ').collect_vec();
    let inserted_tag: Result<_, sqlx::Error> = sqlx::query!(
        "
        INSERT INTO booru_blacklisted_tag
        (booru_setting_id, blacklisted) VALUES ($1, $2)
        ",
        booru_setting_insertion.id,
        split_tags.first()
    )
    .execute(&mut *tx)
    .await;

    tx.commit().await?;

    let response = if inserted_tag.is_err() {
        Err(NotifyError::Warn(
            t!(i18n.booru.blacklist.everything_blacklisted_already).with(mono(tag)),
        ))?
    } else {
        t!(i18n.booru.blacklist.blacklisted).with(mono(tag))
    };

    ctx.say(cool_text(OsakaMoji::ZanyFace, &response)).await?;

    Ok(())
}
