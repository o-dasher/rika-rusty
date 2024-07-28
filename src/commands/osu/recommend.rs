use itertools::Itertools;
use poise_i18n::PoiseI18NTrait;
use rosu_v2::prelude::GameModsLegacy;
use rusty18n::t_prefix;
use sqlx::{postgres::PgRow, Pool, Postgres, QueryBuilder};

use crate::{
    commands::osu::{Context, Mode},
    error,
    osaka_sqlx::osu::{
        DatabaseCatchPerformance, DatabaseGeneralOsuPerformance, DatabaseManiaPerformance,
        DatabaseOsuPerformance, DatabaseOsuScore, DatabaseTaikoPerformance,
    },
    responses::{emojis::OsakaMoji, markdown::mono, templates::cool_text},
    OsakaContext, OsakaData, OsakaResult,
};

pub fn get_weighter<T>(
    vec: Vec<T>,
) -> impl Fn(for<'a> fn(&'a T) -> f64) -> Result<f64, error::Osaka> {
    move |f: fn(&T) -> f64| {
        vec.iter()
            .map(f)
            .enumerate()
            .map(|(i, value)| i32::try_from(i).map(|weight| (value, 0.95f64.powi(weight))))
            .map_ok(|(value, weight_by)| (value * weight_by, weight_by))
            .fold_ok((0f64, 0f64), |(pp_sum, weight), (value, weight_by)| {
                (pp_sum + value, weight + weight_by)
            })
            .map(|(total_pp_sum, total_pp_weight)| total_pp_sum / total_pp_weight)
            .ok()
            .ok_or(error::Osaka::SimplyUnexpected)
    }
}

fn mid_interval(x: f64, delta: f64) -> (f64, f64) {
    let d = delta / 2.;
    (x * (1. - d), x * (1. + d))
}

fn create_weighter<T>(
    performance_values: Vec<T>,
    range: f64,
) -> impl Fn(for<'a> fn(&'a T) -> f64) -> Result<(f64, f64), error::Osaka> {
    let weight_to = get_weighter(performance_values);
    move |field: fn(&T) -> f64| Ok(mid_interval(weight_to(field)?, range))
}

async fn fetch_performance<
    T: Into<DatabaseGeneralOsuPerformance> + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
>(
    osu_id: i64,
    pool: &Pool<Postgres>,
    mode: super::Mode,
) -> Result<Vec<T>, error::Osaka> {
    let row = sqlx::query_as(&format!(
        "
        SELECT pp.* FROM osu_score s
        JOIN {mode}_performance pp ON s.score_id = pp.score_id
        WHERE osu_user_id = $1
        ORDER BY pp.overall DESC
        ",
    ))
    .bind(osu_id)
    .fetch_all(pool)
    .await?;

    if row.is_empty() {
        return Err(super::Error::NotLinked)?;
    }

    Ok(row)
}

async fn query_recommendation<'a>(
    pool: &Pool<Postgres>,
    mode: Mode,
    values: Vec<(&'a str, (f64, f64))>,
) -> Result<DatabaseOsuScore, sqlx::Error> {
    let mut query = QueryBuilder::new(format!(
        "
        SELECT s.*
        FROM osu_score s
        JOIN {mode}_performance pp ON s.score_id = pp.score_id
        WHERE
        "
    ));

    let mut separated = query.separated(" AND ");
    for (name, (min, max)) in values {
        separated
            .push(format!("pp.{name} BETWEEN "))
            .push_bind_unseparated(min)
            .push_bind(max);
    }

    query
        .push(" ORDER BY RANDOM() ")
        .build_query_as()
        .fetch_one(pool)
        .await
}

#[poise::command(slash_command)]
pub async fn recommend(ctx: OsakaContext<'_>, mode: Mode, range: Option<f64>) -> OsakaResult {
    let i18n = ctx.i18n();
    t_prefix!($i18n.osu.recommend);

    let OsakaData { pool, .. } = ctx.data();

    let range = range.unwrap_or(0.3);
    let osu_id = ctx.get_linked_user().await?;

    let labeled_weights = match mode {
        Mode::Osu => {
            let w = create_weighter(
                fetch_performance::<DatabaseOsuPerformance>(osu_id, pool, mode).await?,
                range,
            );

            vec![
                ("overall", w(|v| v.overall)?),
                ("aim", w(|v| v.aim)?),
                ("speed", w(|v| v.speed)?),
                ("accuracy", w(|v| v.accuracy)?),
                ("flashlight", w(|v| v.flashlight)?),
            ]
        }
        Mode::Taiko => {
            let w = create_weighter(
                fetch_performance::<DatabaseTaikoPerformance>(osu_id, pool, mode).await?,
                range,
            );

            vec![
                ("overall", w(|v| v.overall)?),
                ("accuracy", w(|v| v.accuracy)?),
                ("difficulty", w(|v| v.difficulty)?),
            ]
        }
        Mode::Catch => {
            let w = create_weighter(
                fetch_performance::<DatabaseCatchPerformance>(osu_id, pool, mode).await?,
                range,
            );

            vec![("overall", w(|v| v.overall)?)]
        }
        Mode::Mania => {
            let w = create_weighter(
                fetch_performance::<DatabaseManiaPerformance>(osu_id, pool, mode).await?,
                range,
            );

            vec![
                ("overall", w(|v| v.overall)?),
                ("difficulty", w(|v| v.difficulty)?),
            ]
        }
    };

    let recommendation = query_recommendation(pool, mode, labeled_weights).await?;
    let beatmap_link = format!("https://osu.ppy.sh/b/{}", recommendation.map_id);

    // This will be deprecated need to not depend on legacy mods.
    let displayable_mods = GameModsLegacy::try_from_bits(
        recommendation
            .mods
            .try_into()
            .ok()
            .ok_or(error::Osaka::SimplyUnexpected)?,
    )
    .unwrap_or_default();

    let content = t!(recommendation).with((beatmap_link, mono(displayable_mods.to_string())));

    ctx.say(cool_text(OsakaMoji::ZanyFace, &content)).await?;

    Ok(())
}
