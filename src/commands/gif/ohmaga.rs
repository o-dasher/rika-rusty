use std::str::FromStr;

use crate::{
    responses::{emojis::OsakaMoji, templates::cool_text},
    OsakaContext, OsakaResult,
};
use poise::command;

const OHMAGA_URL: &str = "https://media1.tenor.com/m/of_mwJmMNbsAAAAC/azumanga-azumanga-daioh.gif";

#[command(slash_command)]
pub async fn ohmaga(ctx: OsakaContext<'_>) -> OsakaResult {
    let url = url::Url::from_str(OHMAGA_URL)?;

    ctx.send(|r| {
        r.content(cool_text(OsakaMoji::ZanyFace, "OhMaga!"))
            .attachment(poise::serenity_prelude::AttachmentType::Image(url))
    })
    .await?;

    Ok(())
}
