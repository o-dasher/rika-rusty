use crate::{
    commands::booru::{
        blacklist,
        utils::{autocompletes::autocomplete_tag_single, poise::OsakaBooruTag},
    },
    error::NotifyError,
    get_conditional_id_kind_query,
    osaka_sqlx::{booru_setting::SettingKind, BigID, ID},
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
    #[autocomplete = "autocomplete_tag_single"] tag: OsakaBooruTag,
) -> OsakaResult {
    blacklist::check_permissions(ctx, kind).await?;

    let OsakaData { pool, .. } = ctx.data();
    let i18n = ctx.i18n();

    let mut tx = pool.begin().await?;

    let used_setting_kind = kind.get_sqlx_id(ctx)?;

    get_conditional_id_kind_query!(kind);
    conditional_id_kind_query!(
        BigID,
        "
        INSERT INTO discord_{#id_kind} (id) VALUES ({used_setting_kind})
        ON CONFLICT (id) DO UPDATE SET id=EXCLUDED.id RETURNING id
        "
    )
    .fetch_one(&mut *tx)
    .await?;

    let booru_setting_insertion = conditional_id_kind_query!(
        ID,
        "
        INSERT INTO booru_setting ({#id_kind}_id)
        VALUES ({used_setting_kind})
        ON CONFLICT ({#id_kind}_id) DO UPDATE
            SET {#id_kind}_id=EXCLUDED.{#id_kind}_id
        RETURNING id 
        ",
    )
    .fetch_one(&mut *tx)
    .await?;

    let split_tags = tag.0.split(' ').collect_vec();
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
            t!(i18n.booru.blacklist.everything_blacklisted_already).with(mono(tag.0)),
        ))?
    } else {
        t!(i18n.booru.blacklist.blacklisted).with(mono(tag.0))
    };

    ctx.say(cool_text(OsakaMoji::ZanyFace, &response)).await?;

    Ok(())
}
