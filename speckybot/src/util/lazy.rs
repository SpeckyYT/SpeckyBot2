#[macro_export]
macro_rules! holy_cow {
    ($name:ident $($expr:expr),* $(,)?) => {
        $crate::lazy_lock!{
            static ref $name: [std::borrow::Cow<'static, str>; [$([stringify!($expr)].len()),*].len()] = [
                $($expr.into(),)*
            ];
        }
    };
}

#[macro_export]
macro_rules! lazy_lock {
    ($($pub:vis static ref $name:ident: $typ:ty = $code:expr;)*) => {
        $( $pub static $name: std::sync::LazyLock<$typ> = std::sync::LazyLock::new(|| $code); )*
    };
}
