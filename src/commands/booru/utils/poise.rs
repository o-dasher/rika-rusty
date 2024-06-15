use poise::{
    serenity_prelude::{async_trait, json, Context, CreateApplicationCommandOption},
    ApplicationCommandOrAutocompleteInteraction, SlashArgError, SlashArgument,
};

#[derive(Clone)]
pub struct OsakaBooruTag(pub String);

#[async_trait]
impl SlashArgument for OsakaBooruTag {
    async fn extract(
        ctx: &Context,
        interaction: ApplicationCommandOrAutocompleteInteraction<'_>,
        value: &json::Value,
    ) -> Result<Self, SlashArgError>
where {
        Ok(OsakaBooruTag(
            poise::extract_slash_argument!(String, ctx, interaction, value)
                .await?
                .trim()
                .to_lowercase(),
        ))
    }

    fn create(builder: &mut CreateApplicationCommandOption) {
        poise::create_slash_argument!(String, builder)
    }
}
