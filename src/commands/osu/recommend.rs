use itertools::Itertools;
use poise_i18n::PoiseI18NTrait;
use rusty18n::t_prefix;
use sqlx::{postgres::PgRow, Pool, Postgres, QueryBuilder};

use crate::{
    commands::osu::{Context, Mode},
    error,
    osaka_sqlx::osu::{
        CatchPerformance, ManiaPerformance, OsakaOsuPerformance, OsuPerformance, OsuScore,
        TaikoPerformance,
    },
    OsakaContext, OsakaData, OsakaResult,
};

pub fn get_weighter<T>(
    vec: Vec<T>,
) -> impl Fn(for<'a> fn(&'a T) -> f32) -> Result<f32, error::Osaka> {
    move |f: fn(&T) -> f32| {
        vec.iter()
            .map(f)
            .enumerate()
            .map(|(i, value)| i32::try_from(i).map(|weight| (value, 0.95f32.powi(weight))))
            .map_ok(|(value, weight_by)| (value * weight_by, weight_by))
            .fold_ok((0f32, 0f32), |(pp_sum, weight), (value, weight_by)| {
                (pp_sum + value, weight + weight_by)
            })
            .map(|(total_pp_sum, total_pp_weight)| total_pp_sum / total_pp_weight)
            .ok()
            .ok_or(error::Osaka::SimplyUnexpected)
    }
}

fn mid_interval(x: f32, delta: f32) -> (f32, f32) {
    let d = delta / 2.;
    (x * (1. - d), x * (1. + d))
}

fn create_weighter<T>(
    performance_values: Vec<T>,
    range: f32,
) -> impl Fn(for<'a> fn(&'a T) -> f32) -> Result<(f32, f32), error::Osaka> {
    let weight_to = get_weighter(performance_values);
    move |field: fn(&T) -> f32| weight_to(field).map(|w| mid_interval(w, range))
}

async fn fetch_performance<
    T: Into<OsakaOsuPerformance> + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
>(
    osu_id: i64,
    pool: &Pool<Postgres>,
    mode: super::Mode,
) -> Result<Vec<T>, error::Osaka> {
    let row = sqlx::query_as(&format!(
        "
        SELECT pp.* FROM osu_score s
        JOIN {}_performance pp ON s.id = pp.score_id
        WHERE osu_user_id = $1
        ORDER BY pp.overall DESC
        ",
        mode.to_string().to_lowercase()
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
    mode: &'a str,
    values: Vec<(&'a str, (f32, f32))>,
) -> Result<OsuScore, sqlx::Error> {
    let mut query = QueryBuilder::new(format!(
        "
        SELECT s.*
        FROM osu_score s
        JOIN {mode}_performance pp ON s.id = pp.score_id
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
        .push(" ORDER BY RAND() ")
        .build_query_as()
        .fetch_one(pool)
        .await
}

#[macro_export]
macro_rules! init_recommendation {
    ($dollar:tt, $db:expr, $ctx:expr, $range:expr, $mode:ident) => {
        let i18n = $ctx.i18n();
        t_prefix!($dollar, i18n.osu.recommend);

        let range = $range.unwrap_or(0.3);
        let (.., osu_id) = $ctx.linked_osu_user().await?;

        create_weighter!(fetch_performance!($mode, osu_id, $db), range);
    };
}

#[poise::command(slash_command)]
pub async fn recommend(ctx: OsakaContext<'_>, mode: Mode, range: Option<f32>) -> OsakaResult {
    let i18n = ctx.i18n();
    t_prefix!($i18n.osu.recommend);

    let OsakaData { pool, .. } = ctx.data();

    let range = range.unwrap_or(0.3);
    let osu_id = ctx.get_linked_user().await?;

    match mode {
        Mode::Osu => {
            let w = create_weighter(
                fetch_performance::<OsuPerformance>(osu_id, pool, mode).await?,
                range,
            );

            vec![
                w(|v| v.overall)?,
                w(|v| v.aim)?,
                w(|v| v.speed)?,
                w(|v| v.accuracy)?,
                w(|v| v.flashlight)?,
            ]
        }
        Mode::Taiko => {
            let w = create_weighter(
                fetch_performance::<TaikoPerformance>(osu_id, pool, mode).await?,
                range,
            );

            vec![w(|v| v.overall)?, w(|v| v.accuracy)?, w(|v| v.difficulty)?]
        }
        Mode::Catch => {
            let w = create_weighter(
                fetch_performance::<CatchPerformance>(osu_id, pool, mode).await?,
                range,
            );

            vec![w(|v| v.overall)?]
        }
        Mode::Mania => {
            let w = create_weighter(
                fetch_performance::<ManiaPerformance>(osu_id, pool, mode).await?,
                range,
            );

            vec![w(|v| v.overall)?, w(|v| v.difficulty)?]
        }
    };

    Ok(())
}
