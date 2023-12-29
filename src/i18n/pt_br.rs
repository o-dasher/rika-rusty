use super::osaka_i_18_n::OsakaI18N;
use rusty18n::{define_i18n, r, I18NFallback};

pub fn pt_br() -> OsakaI18N {
    define_i18n!(
        OsakaI18N,
        errors: {
            unexpected: r!("")
        },
        user: {
            avatar: {
                footer: {
                    eq: r!("Belexa, eh tu"),
                    other: r!("Eita ele...")
                }
            }
        },
        fun: {
            coinflip: {
                show: r!("Eu jogo uma moeda e..."),
                heads: r!("Cara"),
                tails: r!("Coroa")
            }
        }
    )
}
