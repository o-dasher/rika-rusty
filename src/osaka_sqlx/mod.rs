pub mod booru_blacklisted_tag;
pub mod booru_setting;

use sqlx::types::BigDecimal;

pub enum Jib {
    Jab,
}

pub struct ID<T> {
    pub id: T,
}

pub type BigID = ID<BigDecimal>;
