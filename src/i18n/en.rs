use rusty18n::{r, I18NDynamicResource, I18NFallback};

use super::osaka_i_18_n::OsakaI18N;

type Braceable<T> = T;
macro_rules! ergo_braced {(
    $base:path, $T:ty {
        $(
            $field_name:ident
                // either
                $({ $($body:tt)* })?
                // or
                $(: $value:expr)?
        ),* $(,)?
    }
    $(,)?
) => (::paste::paste! {
    Braceable::<$T> {
        $(
            $field_name:
                // either
                $(
                    ergo_braced!(
                        $base::$field_name,
                        $base::$field_name::[< $field_name:camel >] {
                        $($body)*
                    })
                )? /* or */ $(
                    $value
                )?
            ,
        )*
    }
})}

impl I18NFallback for OsakaI18N {
    fn fallback() -> Self {
        ergo_braced!(super::osaka_i_18_n, Self {
            errors {
                unexpected: r!("Heh? Something unexpected happened with my brain."),
                user_missing_permissions: r!("You don't have the required permissions to execute this command at this level of privilege."),
                must_be_used_on_guild: r!("This command must be used on a server!"),
                register {
                    no_development_guild_set: r!("Failed to register, no development guild is set!"),
                }
            },
            user {
                avatar {
                    footer {
                        eq: r!("Nice, yourself!"),
                        other: r!("They are the..."),
                    },
                },
            },
            fun {
                coinflip {
                    show: r!("I flip a coin and it lands on..."),
                    heads: r!("Heads"),
                    tails: r!("Tails"),
                },
            },
            booru {
                blacklist {
                    blacklisted: r!(|tag| "Sure mistah, blacklisting {tag}!"),
                    everything_blacklisted_already: r!(|tag| "Hey, listen! {tag} is already on the blacklist..."),
                    remove {
                        removed: r!(|tag| "Ok, then! {tag} is not blacklisted anymore."),
                        failed: r!(|tag| "Hey, it seems that {tag} is not being blacklisted here!")
                    },
                    clear {
                        cleared: r!("Nipaa! removed everything from the blacklist for yah!"),
                        failed: r!("There is nothing to remove i'm affraid.")
                    }
                },
            },
            owner {
                register {
                    success: r!("Registered commands successfully!")
                },
            },
            osu {
                link {
                    failed: r!(|u| "Failed to link your osu! profile to {u}"),
                    linked: r!(|u| "Alright linked your osu! profile to {u}"),
                }
            }
        })
    }
}
