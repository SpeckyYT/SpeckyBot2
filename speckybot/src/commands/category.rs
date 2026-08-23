use std::str::FromStr;

use enum_map::Enum;

macro_rules! categories {
    ($($(#[$m:meta])* $var:ident $(= $int:literal)?: $lc:literal|$pp:literal)*) => {
        #[derive(Debug, Clone, Copy, Default, Enum)]
        pub enum Category {
            $( $(#[$m])* $var $(= $int)?, )*
        }

        impl Into<&'static str> for Category {
            fn into(self) -> &'static str {
                match self {
                    $(Category::$var => $lc,)*
                }
            }
        }

        impl FromStr for Category {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($lc|$pp => Ok(Category::$var),)*
                    _ => Err(()),
                }
            }
        }

        impl Category {
            pub const fn pretty_name(self) -> &'static str {
                match self {
                    $(Category::$var => $pp,)*
                }
            }
        }
    };
}

categories!{
    #[default]
    Uncategorized = 0: "uncategorized"|"Uncategorized"
    Important: "important"|"Important"
    Games: "games"|"Games"
    Math: "math"|"Math"
    Nsfw: "nsfw"|"NSFW"
    Owner: "owner"|"Owner"
}
