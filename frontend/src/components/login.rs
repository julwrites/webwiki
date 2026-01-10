use crate::api::login;
use crate::Route;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Login)]
pub fn login_component() -> Html {
    let username = use_state(|| String::new());
    let password = use_state(|| String::new());
    let error = use_state(|| Option::<String>::None);
    let navigator = use_navigator().unwrap();

    let onsubmit = {
        let username = username.clone();
        let password = password.clone();
        let error = error.clone();
        let navigator = navigator.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let username = username.clone();
            let password = password.clone();
            let error = error.clone();
            let navigator = navigator.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match login((*username).clone(), (*password).clone()).await {
                    Ok(_) => {
                        error.set(None);
                        // Force a full reload to update auth state or just navigate
                        // Since AuthWrapper checks /check-auth, we can just navigate to home.
                        // But we might need to update a global auth context if we had one.
                        // For now, let's just navigate.
                        // Actually, we might need to reload to ensure session cookies are sent/recognized cleanly if needed,
                        // but SPA navigation should work if cookies are set.
                        navigator.push(&Route::Home);
                    }
                    Err(_) => {
                        error.set(Some("Invalid username or password".to_string()));
                    }
                }
            });
        })
    };

    let oninput_username = {
        let username = username.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target().unwrap().unchecked_into();
            username.set(input.value());
        })
    };

    let oninput_password = {
        let password = password.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target().unwrap().unchecked_into();
            password.set(input.value());
        })
    };

    html! {
        <div class="login-container">
            <h1>{"Login"}</h1>
            <form {onsubmit}>
                <div class="field">
                    <label>{"Username"}</label>
                    <input type="text" value={(*username).clone()} oninput={oninput_username} />
                </div>
                <div class="field">
                    <label>{"Password"}</label>
                    <input type="password" value={(*password).clone()} oninput={oninput_password} />
                </div>
                if let Some(err) = (*error).clone() {
                    <div class="error">{err}</div>
                }
                <button type="submit">{"Login"}</button>
            </form>
        </div>
    }
}

// Re-export structs so they can be used by api.rs
pub use self::structs::{LoginRequest, LoginResponse};

mod structs {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct LoginRequest {
        pub username: String,
        pub password: String,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct LoginResponse {
        pub success: bool,
        pub message: String,
    }
}
