pub mod en;
pub mod pt_br;

use strum::{Display, EnumString};

nestruct::nest!(
    #[derive(bevy_reflect::Reflect)]
    OsakaI18N {
        errors: {
            unexpected: rusty18n::R?,
            user_missing_permissions: rusty18n::R?,
            must_be_used_on_guild: rusty18n::R?,
            register: {
                no_development_guild_set: rusty18n::R?
            }
        },
        fun: {
            coinflip: {
                show: rusty18n::R?,
                heads: rusty18n::R?,
                tails:  rusty18n::R?
            }
        },
        user: {
            avatar: {
                footer: {
                    eq: rusty18n::R?,
                    other: rusty18n::R?
                }
            }
        },
        booru: {
            blacklist: {
                blacklisted: rusty18n::DR<String>?,
                everything_blacklisted_already: rusty18n::DR<String>?,
                remove: {
                    failed: rusty18n::DR<String>?,
                    removed: rusty18n::DR<String>?
                },
                clear: {
                    cleared: rusty18n::R?,
                    failed: rusty18n::R?
                }
            }
        },
        owner: {
            register: {
                success: rusty18n::R?
            }
        },
        osu: {
            link: {
                failed: rusty18n::DR<String>?,
                linked: rusty18n::DR<String>?
            },
            submit: {
                started: rusty18n::R?,
                processing: rusty18n::DR<(String, String)>?,
                finished: rusty18n::R?
            },
            errors: {
                not_linked: rusty18n::R?
            }
        }
    }
);

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, Default, Display, EnumString)]
pub enum OsakaLocale {
    #[default]
    #[strum(serialize = "en-US")]
    UnitedStatesEnglish,

    #[strum(serialize = "pt-BR")]
    BrazilianPortuguese,
}
