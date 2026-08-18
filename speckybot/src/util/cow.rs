#[macro_export]
macro_rules! holy_cow {
    ($name:ident $($expr:expr),* $(,)?) => {
        lazy_static::lazy_static!{
            static ref $name: [Cow<'static, str>; [$([stringify!($expr)].len()),*].len()] = [
                $($expr.into(),)*
            ];
        }
    };
}
