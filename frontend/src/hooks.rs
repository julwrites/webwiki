use crate::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[hook]
pub fn use_create_file(current_volume: String) -> Callback<()> {
    let navigator = use_navigator().expect("use_create_file must be used inside a BrowserRouter");

    Callback::from(move |_| {
        if let Some(path) = gloo_dialogs::prompt("Enter file path (e.g. folder/note.md):", None) {
            if !path.trim().is_empty() {
                navigator.push(&Route::Wiki {
                    volume: current_volume.clone(),
                    path: path.trim().to_string(),
                });
            }
        }
    })
}
