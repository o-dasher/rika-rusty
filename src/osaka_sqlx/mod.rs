pub mod booru_blacklisted_tag;
pub mod booru_setting;

pub enum Fall {
    Through,
}

pub struct ID<T> {
    pub id: T,
}

pub type BigID = ID<i64>;
