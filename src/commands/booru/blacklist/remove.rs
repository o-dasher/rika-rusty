use crate::commands::booru::{autocomplete_tag, self, SettingKind};
use crate::OsakaData;
use crate::{OsakaContext, OsakaResult};
use poise::command;

#[command(slash_command)]
pub async fn remove(
    ctx: OsakaContext<'_>,
    kind: SettingKind,
    #[autocomplete = "autocomplete_tag"] tag: String,
) -> OsakaResult {
    let OsakaData { pool, .. } = ctx.data();
    let [inserted_guild, inserted_channel, inserted_user] =
        booru::get_all_owner_insert_options_some_owner(ctx, kind)?;

    sqlx::query!(
        "
        DELETE FROM booru_blacklisted_tag t
        USING booru_setting s
        WHERE t.blacklisted=$1 AND s.id=t.booru_setting_id
        AND s.guild_id=$2 OR s.user_id=$3 OR s.channel_id=$4
        ",
        tag,
        inserted_guild,
        inserted_channel,
        inserted_user
    )
    .fetch_one(pool)
    .await?;

    Ok(())
}
