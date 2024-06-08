use sqlx_conditional_queries::conditional_query_as;

use crate::{
    commands::booru::{self, blacklist::BigID, SettingKind},
    error::NotifyError,
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
    let [inserted_guild, inserted_channel, inserted_user] =
        booru::get_all_setting_kind_db_ids_only_allowing_this_kind(ctx, kind)?;

    let result = conditional_query_as!(
        BigID,
        "
        DELETE FROM booru_blacklisted_tag t
        USING booru_setting s
        WHERE {#extra_query} s.id=t.booru_setting_id
        AND 
            s.guild_id={inserted_guild} OR
            s.channel_id={inserted_channel} OR
            s.user_id={inserted_user}
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
