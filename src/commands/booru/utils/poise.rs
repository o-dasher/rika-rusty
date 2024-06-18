use std::str::FromStr;

use poise::{
    serenity_prelude::{
        async_trait, json, ArgumentConvert, Context, CreateApplicationCommandOption,
    },
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

#[derive(Clone)]
pub struct DefaultSlash<T: Default + FromStr>(pub T);

impl<E, T: Default + FromStr<Err = E>> FromStr for DefaultSlash<T> {
    type Err = E;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(T::from_str(s)?))
    }
}

#[async_trait]
impl<T: Default + FromStr> SlashArgument for DefaultSlash<T>
where
    Option<T>: ArgumentConvert + SlashArgument + Sync,
{
    async fn extract(
        ctx: &Context,
        interaction: ApplicationCommandOrAutocompleteInteraction<'_>,
        value: &json::Value,
    ) -> Result<Self, SlashArgError>
where {
        Ok(DefaultSlash(
            poise::extract_slash_argument!(Option<T>, ctx, interaction, value)
                .await?
                .unwrap_or_default(),
        ))
    }

    fn create(builder: &mut CreateApplicationCommandOption) {
        poise::create_slash_argument!(Option<T>, builder)
    }
}
