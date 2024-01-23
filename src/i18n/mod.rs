pub mod en;
pub mod pt_br;

use strum::{Display, EnumString};

nestruct::nest!(
    #[derive(bevy_reflect::Reflect)]
    OsakaI18N {
        errors: {
            unexpected: rusty18n::R?,
            user_missing_permissions: rusty18n::R?
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
                blacklisted: rusty18n::DR<String>?
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
