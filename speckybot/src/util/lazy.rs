#[macro_export]
macro_rules! holy_cow {
    ($pub:vis $name:ident<$life:lifetime, $ty:ty> $($expr:expr),* $(,)?) => {
        $crate::lazy_lock!{
            $pub static ref $name: [std::borrow::Cow<$life, $ty>; [$([stringify!($expr)].len()),*].len()] = [
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
