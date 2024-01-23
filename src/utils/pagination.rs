use chrono::Duration;
use poise::{serenity_prelude::CollectComponentInteraction, CreateReply};

use crate::{error::OsakaError, responses::emojis::OsakaMoji, OsakaContext, OsakaResult};

pub struct ComponentContextId(pub u64);

impl ComponentContextId {
    pub fn create_id(&self, combined_id: &str) -> String {
        format!("{}{combined_id}", self.0)
    }
}

pub struct Paginator<'a> {
    pub ctx: OsakaContext<'a>,
    pub amount_pages: usize,
}

impl<'a> Paginator<'a> {
    pub fn new(ctx: OsakaContext<'a>, amount_pages: usize) -> Self {
        Self { ctx, amount_pages }
    }

    pub async fn paginate(
        &self,
        create_reply: impl Fn(usize, &mut CreateReply<'a>) -> Result<CreateReply<'a>, OsakaError>,
    ) -> OsakaResult {
        let amount_pages = self.amount_pages;
        let ctx = self.ctx;

        let ctx_id = ComponentContextId(ctx.id());

        let prev_button = ctx_id.create_id("prev");
        let close_button = ctx_id.create_id("close");
        let next_button = ctx_id.create_id("next");

        let mut current_idx = 0usize;

        let create_reply_inner = |idx: usize| -> Result<CreateReply<'a>, OsakaError> {
            create_reply(idx, &mut CreateReply::default())
        };

        fn forward_reply<'a, 'b>(
            based_on: CreateReply<'a>,
            r: &'b mut CreateReply<'a>,
            (prev_button, close_button, next_button): (&String, &String, &String),
        ) -> &'b mut CreateReply<'a> {
            r.clone_from(&based_on);

            let existing_components = r.components.clone();

            r.components(|b| {
                if let Some(existing) = &existing_components {
                    b.clone_from(existing)
                }

                b.create_action_row(|b| {
                    b.create_button(|b| b.custom_id(prev_button).emoji(OsakaMoji::ArrowBackward))
                        .create_button(|b| b.custom_id(close_button).emoji(OsakaMoji::X))
                        .create_button(|b| b.custom_id(next_button).emoji(OsakaMoji::ArrowForward))
                })
            });

            r
        }

        let buttons_tuple = (&prev_button, &close_button, &next_button);

        let response = create_reply_inner(current_idx)?;
        let sent = ctx
            .send(|b| forward_reply(response, b, buttons_tuple))
            .await?;

        while let Some(press) = CollectComponentInteraction::new(ctx)
            .filter(move |press| press.data.custom_id.starts_with(&ctx_id.0.to_string()))
            .timeout(Duration::hours(1).to_std()?)
            .await
        {
            current_idx = match &press.data.custom_id {
                x if x == &prev_button => {
                    current_idx.checked_sub(1).unwrap_or(amount_pages - 1usize)
                }
                x if x == &next_button => {
                    let new_idx = current_idx + 1;
                    if new_idx > amount_pages {
                        0
                    } else {
                        new_idx
                    }
                }
                x if x == &close_button => {
                    sent.delete(ctx).await?;
                    continue;
                }
                _ => continue,
            };

            press.defer(self.ctx).await?;

            let response = create_reply_inner(current_idx)?;
            sent.edit(ctx, |b| forward_reply(response, b, buttons_tuple))
                .await?;
        }

        let response = create_reply_inner(current_idx)?;
        sent.edit(ctx, |b| {
            forward_reply(response, b, buttons_tuple).components(|b| b)
        })
        .await?;

        Ok(())
    }
}
