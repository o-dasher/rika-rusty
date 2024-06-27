use itertools::Itertools;
use poise::command;
use poise_i18n::PoiseI18NTrait;
use rand::seq::SliceRandom;
use rusty18n::t_prefix;
use strum::{EnumIter, IntoEnumIterator};

use crate::{
    error,
    responses::{emojis::OsakaMoji, markdown::mono, templates::cool_text},
    OsakaContext, OsakaResult,
};

#[derive(EnumIter)]
enum PossiblePlay {
    Heads,
    Flips,
}

#[command(slash_command)]
pub async fn coinflip(ctx: OsakaContext<'_>) -> OsakaResult {
    let i18n = ctx.i18n();
    t_prefix!($i18n.fun.coinflip);

    let possible_plays = PossiblePlay::iter().collect_vec();
    let flip_result = possible_plays
        .choose(&mut rand::thread_rng())
        .ok_or(error::Osaka::SimplyUnexpected)?;

    let coin_string = match flip_result {
        PossiblePlay::Heads => t!(heads),
        PossiblePlay::Flips => t!(tails),
    };

    ctx.reply(cool_text(
        OsakaMoji::ZanyFace,
        &format!("{} {}", t!(show), mono(coin_string)),
    ))
    .await?;

    Ok(())
}
