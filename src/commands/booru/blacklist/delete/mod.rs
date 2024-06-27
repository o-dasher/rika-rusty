use crate::{
    commands::booru::blacklist,
    error::NotifyError,
    get_id_kind_query,
    osaka_sqlx::{booru_setting::SettingKind, I64ID},
    responses::{emojis::OsakaMoji, templates::cool_text},
    OsakaContext, OsakaData, OsakaResult,
};

pub mod clear;
pub mod remove;

pub enum Operation {
    Remove(String),
    Clear,
}

pub async fn provide_delete_feedback<F: (Fn(bool) -> String) + Send>(
    ctx: OsakaContext<'_>,
    kind: SettingKind,
    operation: Operation,
    provide_message: F,
) -> OsakaResult {
    blacklist::check_permissions(ctx, kind).await?;

    let OsakaData { pool, .. } = ctx.data().as_ref();
    let inserted_discord_id = kind.get_sqlx_id(ctx)?;

    get_id_kind_query!(kind);
    let result = id_kind_query!(
        I64ID,
        r#"
        DELETE FROM booru_blacklisted_tag t
        USING booru_setting s
        WHERE {#extra_query} s.id=t.booru_setting_id
        AND s.{#id_kind}_id={inserted_discord_id}
        RETURNING id
        "#,
        #extra_query = match operation {
            Operation::Remove(tag) => "t.blacklisted={tag} AND",
            Operation::Clear => ""
        }
    )
    .fetch_all(pool)
    .await?;

    let success = !result.is_empty();
    let message = provide_message(success);

    if success {
        ctx.say(cool_text(OsakaMoji::ZanyFace, &message)).await?;
    } else {
        Err(NotifyError::Warn(message))?;
    }

    Ok(())
}
