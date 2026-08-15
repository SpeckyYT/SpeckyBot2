use serenity::all::{Context, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, Message};
use speckycards::{CARDS, Rarity, SpeckyCard};

use crate::util::embed::default_embed;

crate::command! {
    names: "puttanesimo",
    category: "lollolacustre",
    run: |ctx, msg, data| {
        let rarity = Rarity::pick_random();
        let result_card = CARDS.get_random(rarity).unwrap();

        let message = get_formatted_message(result_card, ctx, msg);
        
        Ok(
            msg.channel_id
                .send_message(&ctx.http, CreateMessage::new().embed(message))
                .await
                .map(|_| ())?
        )
    }
}

pub fn get_formatted_message(speckycard: &'static SpeckyCard, ctx: &Context, msg: &Message) -> CreateEmbed {
    let embed = default_embed(Some(ctx))
        .author(CreateEmbedAuthor::new(&msg.author.name))
        .footer(CreateEmbedFooter::new(speckycard.rarity().text()));
    
    use speckycards::ResultType::*;

    match speckycard.res() {
        Text(text) => embed.description(text),
        Image { text, image } => embed.description(text).image(image),
        ImageOnly(image) => embed.image(image)
    }
}
