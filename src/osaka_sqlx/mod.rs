use sqlx::types::BigDecimal;

pub mod booru_blacklisted_tag;
pub mod booru_setting;
pub mod discord;
pub mod osu;

pub enum Fall {
    Through,
}

#[derive(sqlx::FromRow)]
pub struct ID<T> {
    pub id: T,
}

pub type I64ID = ID<i64>;
pub type BigDecimalID = ID<BigDecimal>;
