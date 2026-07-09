use leptos::prelude::*;

mod components;
mod pages;
mod structures;
mod utils;

fn main() {
    leptos::mount::mount_to_body(App)
}

#[component]
fn App() -> impl IntoView {
    let (current_page, set_current_page) = signal(pages::AppPage::Home);
    provide_context(set_current_page);

    view! {
        {move || match current_page.get() {
            pages::AppPage::Home => view! { <pages::home::HomePage/> }.into_any(),
            pages::AppPage::PracticeModeSelect => {
                view! { <pages::practice_mode::PracticeModePage/> }.into_any()
            }
            pages::AppPage::LexiconMode => {
                view! { <pages::lexicon_mode::LexiconModePage/> }.into_any()
            }
            pages::AppPage::LocalLexiconEditor => {
                view! { <pages::local_lexicon_editor::LocalLexiconEditorPage/> }.into_any()
            }
        }}
    }
}