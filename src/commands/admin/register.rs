use crate::{OsakaContext, OsakaResult};
use poise::{command, ChoiceParameter};

#[derive(ChoiceParameter, Default)]
pub enum RegisterKind {
    #[default]
    Development,
    Local,
    Global,
}

#[command(slash_command)]
pub async fn register(_ctx: OsakaContext<'_>, _on: Option<RegisterKind>) -> OsakaResult {
    Ok(())
}
