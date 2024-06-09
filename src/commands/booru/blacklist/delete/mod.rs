use crate::{
    commands::booru::{self, blacklist::BigID, get_setting_kind_db_id, SettingKind},
    error::NotifyError,
    get_conditional_id_kind_query,
    responses::{emojis::OsakaMoji, templates::cool_text},
    OsakaContext, OsakaData, OsakaResult,
};

pub mod clear;
pub mod remove;

pub enum DeleteOperation {
    Remove(String),
    Clear,
}

pub async fn provide_delete_feedback<F: Fn(bool) -> String>(
    ctx: OsakaContext<'_>,
    kind: SettingKind,
    operation: DeleteOperation,
    provide_message: F,
) -> OsakaResult {
    let OsakaData { pool, .. } = ctx.data();

    let inserted_discord_id = get_setting_kind_db_id(ctx, kind)?;

    get_conditional_id_kind_query!(kind);
    let result = conditional_id_kind_query!(
        BigID,
        "
        DELETE FROM booru_blacklisted_tag t
        USING booru_setting s
        WHERE {#extra_query} s.id=t.booru_setting_id
        AND s.{#id_kind}_id={inserted_discord_id}
        RETURNING id
        ",
        #extra_query = match operation {
            DeleteOperation::Remove(tag) => "t.blacklisted={tag} AND",
            DeleteOperation::Clear => ""
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
