use sqlx::{prelude::FromRow, types::BigDecimal};

#[derive(FromRow)]
pub struct OsuScore {
    pub score_id: BigDecimal,
    pub mode: i8,

    pub mods: i32,
    pub map_id: i32,
    pub osu_user_id: i64,
}

#[derive(FromRow)]
pub struct OsuPerformance {
    pub score_id: BigDecimal,
    pub mode: i8,

    pub overall: f32,
    pub aim: f32,
    pub speed: f32,
    pub accuracy: f32,
    pub flashlight: f32,
}

#[derive(FromRow)]
pub struct TaikoPerformance {
    pub score_id: BigDecimal,
    pub mode: i8,

    pub overall: f32,
    pub accuracy: f32,
    pub difficulty: f32,
}

#[derive(FromRow)]
pub struct CatchPerformance {
    pub score_id: BigDecimal,
    pub mode: i8,

    pub overall: f32,
}

#[derive(FromRow)]
pub struct ManiaPerformance {
    pub score_id: BigDecimal,
    pub mode: i8,

    pub overall: f32,
    pub difficulty: f32,
}

#[derive(derive_more::From)]
pub enum OsakaOsuPerformance {
    Osu(OsuPerformance),
    Taiko(TaikoPerformance),
    Catch(CatchPerformance),
    Mania(ManiaPerformance),
}
