use sqlx::postgres::PgQueryResult;

use crate::{
    error::NotifyError,
    responses::{emojis::OsakaMoji, templates::cool_text},
    OsakaContext, OsakaResult,
};

pub mod clear;
pub mod remove;

pub async fn provide_delete_feedback<F: Fn(bool) -> String>(
    ctx: OsakaContext<'_>,
    result: PgQueryResult,
    provide_message: F,
) -> OsakaResult {
    let success = result.rows_affected() < 1;
    let message = provide_message(success);

    if success {
        Err(NotifyError::Warn(message))?;
    } else {
        ctx.say(cool_text(OsakaMoji::ZanyFace, &message)).await?;
    }

    Ok(())
}
