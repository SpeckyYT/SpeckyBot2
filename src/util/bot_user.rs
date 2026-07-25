use serenity::all::{Context, User};

#[inline]
pub fn bot_avatar_url(ctx: &Context) -> String {
    let bot_user = ctx.cache.current_user();
    user_avatar_url(&bot_user)
}

#[inline]
pub fn user_avatar_url(user: &User) -> String {
    user.avatar_url().unwrap_or(user.default_avatar_url())
}
