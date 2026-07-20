use serenity::all::Context;

pub fn avatar_url(ctx: &Context) -> String {
    let bot_user = ctx.cache.current_user();
    bot_user.avatar_url().unwrap_or(bot_user.default_avatar_url())
}
