use leptos::prelude::*;

use crate::structures::word_bank_entry::UiLanguage;

pub fn gate_locale_signal(
    ui_language: ReadSignal<UiLanguage>,
) -> Signal<solana_gate::GateLocale> {
    Signal::derive(move || {
        if ui_language.get() == UiLanguage::Zh {
            solana_gate::GateLocale::Zh
        } else {
            solana_gate::GateLocale::En
        }
    })
}

pub fn component_locale_signal(
    ui_language: ReadSignal<UiLanguage>,
) -> Signal<solana_leptos_component::GateLocale> {
    Signal::derive(move || {
        if ui_language.get() == UiLanguage::Zh {
            solana_leptos_component::GateLocale::Zh
        } else {
            solana_leptos_component::GateLocale::En
        }
    })
}
