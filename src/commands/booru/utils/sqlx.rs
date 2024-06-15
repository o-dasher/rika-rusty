#[macro_export]
macro_rules! get_conditional_id_kind_query {
    ($kind:ident) => {
        sqlx_conditional_queries_layering::create_conditional_query_as!(
            conditional_id_kind_query,
            #id_kind = match $kind {
                SettingKind::Guild => "guild",
                SettingKind::Channel => "channel",
                SettingKind::User => "user"
            }
        );
    };
}
