use chrono::Duration;
use poise::{serenity_prelude::CollectComponentInteraction, CreateReply};

use crate::{error::OsakaError, responses::emojis::OsakaMoji, OsakaContext, OsakaResult};

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
        let ctx = self.ctx;
        let amount_pages = self.amount_pages;

        let ctx_id = ctx.id();
        let all_buttons = ["prev", "close", "next"].map(|v| format!("{}{v}", ctx_id));
        let [prev_button, close_button, next_button] = &all_buttons;

        let mut current_idx = 0;

        let create_base_reply = |idx: usize| -> Result<CreateReply<'a>, OsakaError> {
            create_reply(idx, &mut CreateReply::default())
        };

        let forward_reply = for<'b> |base: CreateReply<'a>,
                                     r: &'b mut CreateReply<'a>|
                 -> &'b mut CreateReply<'a> {
            r.clone_from(&base);
            r.components = r
                .components
                .clone()
                .unwrap_or_default()
                .create_action_row(|b| {
                    b.create_button(|b| b.custom_id(prev_button).emoji(OsakaMoji::ArrowBackward))
                        .create_button(|b| b.custom_id(close_button).emoji(OsakaMoji::X))
                        .create_button(|b| b.custom_id(next_button).emoji(OsakaMoji::ArrowForward))
                })
                .clone()
                .into();
            r
        };

        let response = create_base_reply(current_idx)?;
        let sent = ctx.send(|b| forward_reply(response, b)).await?;

        while let Some(press) = CollectComponentInteraction::new(ctx)
            .filter(move |press| {
                press
                    .data
                    .custom_id
                    .starts_with(&ctx_id.to_owned().to_string())
            })
            .timeout(Duration::hours(1).to_std()?)
            .await
        {
            current_idx = match &press.data.custom_id {
                x if x == prev_button => {
                    current_idx.checked_sub(1).unwrap_or(amount_pages - 1usize)
                }
                x if x == next_button => (current_idx + 1) % amount_pages,
                x if x == close_button => {
                    sent.delete(ctx).await?;
                    continue;
                }
                _ => continue,
            };

            press.defer(self.ctx).await?;

            let response = create_base_reply(current_idx)?;
            sent.edit(ctx, |b| forward_reply(response, b)).await?;
        }

        let response = create_base_reply(current_idx)?;
        sent.edit(ctx, |b| forward_reply(response, b).components(|b| b))
            .await?;

        Ok(())
    }
}
