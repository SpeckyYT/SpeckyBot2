use enum_map::Enum;

macro_rules! categories {
    ($($(#[$m:meta])* $var:ident: $lc:literal|$pp:literal)*) => {
        #[derive(Debug, Clone, Copy, Default, Enum)]
        pub enum Category {
            $( $(#[$m])* $var, )*
        }

        impl Into<&'static str> for Category {
            fn into(self) -> &'static str {
                match self {
                    $(Category::$var => $lc,)*
                }
            }
        }

        impl Category {
            pub const fn pretty_name(self) -> &'static str {
                match self {
                    $(Category::$var => $pp,)*
                }
            }
            pub fn from_str(string: &str) -> Option<Category> {
                match string {
                    $($lc|$pp => Some(Category::$var),)*
                    _ => None,
                }
            }
        }
    };
}

categories!{
    Important: "important"|"Important"
    Games: "games"|"Games"
    Math: "math"|"Math"
    Nsfw: "nsfw"|"NSFW"
    Owner: "owner"|"Owner"
    #[default]
    Uncategorized: "uncategorized"|"Uncategorized"
}
