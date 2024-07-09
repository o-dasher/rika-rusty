pub mod booru;
pub mod fun;
pub mod gif;
pub mod osu;
pub mod owner;
pub mod user;

#[macro_export]
macro_rules! create_command_group {
    ($cmd_name:ident, [$($subcommands:expr),*]) => {
        use poise::command;
        use $crate::{OsakaContext, OsakaResult};

        #[command(slash_command, subcommands($($subcommands),*))]
        pub async fn $cmd_name(_ctx: OsakaContext<'_>) -> OsakaResult {
            Ok(())
        }
    };
}
