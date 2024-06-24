use crate::{error::NotifyError, osaka_sqlx::booru_setting::SettingKind, responses::{markdown::mono, templates::cool_text}};
use poise_i18n::PoiseI18NTrait;
use rusty18n::t_prefix;

use crate::{responses::emojis::OsakaMoji, OsakaContext, OsakaData, OsakaResult};


#[poise::command(slash_command)]
pub async fn link(ctx: OsakaContext<'_>, name: String) -> OsakaResult {
    let i18n = ctx.i18n();
    t_prefix!($i18n.osu.link);

    let OsakaData { pool, rosu,  .. } = ctx.data();

    let osu_user = rosu
        .user(&name)
        .await
        .map_err(|_| NotifyError::Warn(t!(failed).with(mono(name))))?;

    let osu_user_id = osu_user.user_id as i64;

    let insertion = SettingKind::User.get_sqlx_id(ctx)?;

    let mut tx = pool.begin().await?;

    sqlx::query!(
        "
        INSERT INTO discord_user (id, osu_user_id)
        VALUES ($1, $2)
        ON CONFLICT (id) DO UPDATE
            SET osu_user_id = EXCLUDED.osu_user_id
        ",
        insertion,
        osu_user_id 
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "
        INSERT INTO osu_user (id) VALUES ($1)
        ON CONFLICT DO NOTHING
        ",
        osu_user_id
    ).execute(&mut *tx).await?;


    tx.commit().await?;

    ctx.say(cool_text(
        OsakaMoji::ZanyFace,
        &t!(linked).with(mono(osu_user.username.to_string())),
    ))
    .await?;

    Ok(())
}
