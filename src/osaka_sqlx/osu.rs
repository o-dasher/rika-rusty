use sqlx::{prelude::FromRow, types::BigDecimal};

#[derive(FromRow)]
pub struct DatabaseOsuScore {
    pub score_id: BigDecimal,
    pub mode: i16,

    pub mods: i64,
    pub map_id: i32,
    pub osu_user_id: i64,
}

#[derive(FromRow)]
pub struct DatabaseOsuPerformance {
    pub score_id: BigDecimal,
    pub mode: i16,

    pub overall: f64,
    pub aim: f64,
    pub speed: f64,
    pub accuracy: f64,
    pub flashlight: f64,
}

#[derive(FromRow)]
pub struct DatabaseTaikoPerformance {
    pub score_id: BigDecimal,
    pub mode: i16,

    pub overall: f64,
    pub accuracy: f64,
    pub difficulty: f64,
}

#[derive(FromRow)]
pub struct DatabaseCatchPerformance {
    pub score_id: BigDecimal,
    pub mode: i16,

    pub overall: f64,
}

#[derive(FromRow)]
pub struct DatabaseManiaPerformance {
    pub score_id: BigDecimal,
    pub mode: i16,

    pub overall: f64,
    pub difficulty: f64,
}

#[derive(derive_more::From)]
pub enum DatabaseGeneralOsuPerformance {
    Osu(DatabaseOsuPerformance),
    Taiko(DatabaseTaikoPerformance),
    Catch(DatabaseCatchPerformance),
    Mania(DatabaseManiaPerformance),
}
