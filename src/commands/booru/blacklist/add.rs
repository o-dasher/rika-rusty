use crate::{
    commands::{
        booru,
        booru::{
            autocomplete_tag, blacklist,
            blacklist::{BigID, ID},
            SettingKind,
        },
    },
    error::NotifyError,
    responses::{emojis::OsakaMoji, markdown::mono, templates::cool_text},
    OsakaContext, OsakaData, OsakaResult,
};
use itertools::Itertools;
use poise::command;
use poise_i18n::PoiseI18NTrait;
use rusty18n::t;
use sqlx::postgres::any::AnyConnectionBackend;

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

    let owner_id = booru::get_setting_kind_db_id(ctx, kind)?;
    let [inserted_guild, inserted_channel, inserted_user] =
        booru::get_all_setting_kind_db_ids_only_allowing_this_kind(ctx, kind)?;

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

    let mut blacklisted_operations = vec![];

    for blacklisted_tag in all_blacklisted_tags {
        (*tx).commit().await?;

        let inserted_tag: Result<_, sqlx::Error> = sqlx::query!(
            "
            INSERT INTO booru_blacklisted_tag
            (booru_setting_id, blacklisted) VALUES ($1, $2)
            ",
            booru_setting_insertion.id,
            blacklisted_tag
        )
        .execute(&mut *tx)
        .await;

        if let Err(e) = inserted_tag {
            match e {
                sqlx::Error::Database(e) => {
                    if e.is_unique_violation() {
                        (*tx).rollback().await?;
                        blacklisted_operations.push((blacklisted_tag, Ok(())));
                    } else {
                        Err(sqlx::Error::Database(e))?
                    }
                }
                e => Err(e)?,
            }
        } else {
            blacklisted_operations.push((blacklisted_tag, Err(())))
        }
    }

    let [already_was_blacklisted, successfully_blacklisted] = [Ok, Err].map(|k| {
        blacklisted_operations
            .iter()
            .filter(|(.., result)| *result == k(()))
            .map(|(tag, ..)| *tag)
            .collect_vec()
    });

    tx.commit().await?;

    let response = if !already_was_blacklisted.is_empty() {
        if successfully_blacklisted.is_empty() {
            Err(NotifyError::Warn(
                t!(i18n.booru.blacklist.everything_blacklisted_already).with(mono(tag)),
            ))?
        } else {
            t!(i18n.booru.blacklist.partial_blacklist).with(
                [already_was_blacklisted, successfully_blacklisted]
                    .map(|v| mono(v.join(" ")))
                    .into(),
            )
        }
    } else {
        t!(i18n.booru.blacklist.blacklisted).with(mono(tag))
    };

    ctx.say(cool_text(OsakaMoji::ZanyFace, &response)).await?;

    Ok(())
}
