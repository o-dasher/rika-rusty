use itertools::Itertools;
use poise::command;

use crate::{
    commands::booru::{self, SettingKind},
    responses::markdown::mono,
    utils::pagination::Paginator,
    OsakaContext, OsakaData, OsakaResult,
};

#[command(slash_command)]
pub async fn list(ctx: OsakaContext<'_>, kind: SettingKind) -> OsakaResult {
    let OsakaData { pool, .. } = ctx.data();
    let [inserted_guild, inserted_channel, inserted_user] =
        booru::get_all_setting_kind_db_ids_only_allowing_this_kind(ctx, kind)?;

    let result = sqlx::query!(
        "
        SELECT * FROM booru_blacklisted_tag t
        JOIN booru_setting s ON s.id = t.booru_setting_id
        WHERE s.id=t.booru_setting_id
        AND s.guild_id=$1 OR s.channel_id=$2 OR s.user_id=$3
        ",
        inserted_guild,
        inserted_channel,
        inserted_user
    )
    .fetch_all(pool)
    .await?;

    let chunk_result = result
        .iter()
        .map(|v| &v.blacklisted)
        .chunks(64)
        .into_iter()
        .map(Itertools::collect_vec)
        .collect_vec();

    let paginator = Paginator::new(ctx, chunk_result.len());
    paginator
        .paginate(|idx, r| {
            dbg!(idx);

            Ok(r.embed(|e| {
                e.title(format!("Blacklist for {kind}")).description({
                    if let Some(idx_values) = chunk_result.get(idx)
                        && !idx_values.is_empty()
                    {
                        format!("{}.", idx_values.iter().map(mono).join(", "))
                    } else {
                        "No blacklists here...".to_string()
                    }
                })
            })
            .to_owned())
        })
        .await?;

    Ok(())
}
