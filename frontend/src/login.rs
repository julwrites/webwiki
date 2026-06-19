use gloo_net::http::Request;
use serde::Serialize;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Serialize)]
struct LoginRequest {
    username: String,
    password: String,
    stay_signed_in: bool,
}

#[function_component(Login)]
pub fn login() -> Html {
    let username = use_state(String::new);
    let password = use_state(String::new);
    let stay_signed_in = use_state(|| false);
    let error_msg = use_state(|| Option::<String>::None);
    let is_loading = use_state(|| false);

    let on_username_change = {
        let username = username.clone();
        Callback::from(move |e: Event| {
            let target: Option<web_sys::EventTarget> = e.target();
            if let Some(target) = target {
                use wasm_bindgen::JsCast;
                let input = target.unchecked_into::<HtmlInputElement>();
                username.set(input.value());
            }
        })
    };

    let on_password_change = {
        let password = password.clone();
        Callback::from(move |e: Event| {
            let target: Option<web_sys::EventTarget> = e.target();
            if let Some(target) = target {
                use wasm_bindgen::JsCast;
                let input = target.unchecked_into::<HtmlInputElement>();
                password.set(input.value());
            }
        })
    };

    let on_stay_signed_in_change = {
        let stay_signed_in = stay_signed_in.clone();
        Callback::from(move |e: Event| {
            let target: Option<web_sys::EventTarget> = e.target();
            if let Some(target) = target {
                use wasm_bindgen::JsCast;
                let input = target.unchecked_into::<HtmlInputElement>();
                stay_signed_in.set(input.checked());
            }
        })
    };

    let on_submit = {
        let username = username.clone();
        let password = password.clone();
        let stay_signed_in = stay_signed_in.clone();
        let error_msg = error_msg.clone();
        let is_loading = is_loading.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let username_val = (*username).clone();
            let password_val = (*password).clone();
            let stay_signed_in_val = *stay_signed_in;
            let is_loading = is_loading.clone();
            let error_msg = error_msg.clone();

            is_loading.set(true);
            error_msg.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let payload = LoginRequest {
                    username: username_val,
                    password: password_val,
                    stay_signed_in: stay_signed_in_val,
                };

                let request = match Request::post("/api/login").json(&payload) {
                    Ok(req) => req,
                    Err(e) => {
                        is_loading.set(false);
                        error_msg.set(Some(format!("Failed to prepare request: {}", e)));
                        return;
                    }
                };

                let resp = request.send().await;

                is_loading.set(false);

                match resp {
                    Ok(r) if r.ok() => {
                        let _ = gloo_utils::window().location().set_href("/");
                    }
                    Ok(r) => {
                        if r.status() == 401 {
                            error_msg.set(Some("Invalid username or password".to_string()));
                        } else {
                            error_msg.set(Some(format!("Error: {}", r.status())));
                        }
                    }
                    Err(_) => {
                        error_msg.set(Some("Network error while logging in".to_string()));
                    }
                }
            });
        })
    };

    html! {
        <div class="login-wrapper">
            <div class="login-card">
                <h2>{ "Login to WebWiki" }</h2>
                if let Some(msg) = (*error_msg).as_ref() {
                    <div class="error-message text-danger text-center mb-4" role="alert">{ msg }</div>
                }
                <form onsubmit={on_submit}>
                    <div class="form-group mb-4">
                        <label for="username" class="d-block mb-2">{ "Username" }</label>
                        <input
                            type="text"
                            id="username"
                            value={(*username).clone()}
                            onchange={on_username_change}
                            required=true
                            class="w-full"
                        />
                    </div>
                    <div class="form-group mb-4">
                        <label for="password" class="d-block mb-2">{ "Password" }</label>
                        <input
                            type="password"
                            id="password"
                            value={(*password).clone()}
                            onchange={on_password_change}
                            required=true
                            class="w-full"
                        />
                    </div>
                    <div class="form-group mb-4 flex items-center gap-2">
                        <input
                            type="checkbox"
                            id="stay-signed-in"
                            checked={*stay_signed_in}
                            onchange={on_stay_signed_in_change}
                            class="m-0 cursor-pointer"
                        />
                        <label for="stay-signed-in" class="cursor-pointer">
                            { "Stay signed in for 90 days" }
                        </label>
                    </div>
                    <button type="submit" disabled={*is_loading} aria-busy={(*is_loading).to_string()} class="btn btn-primary w-full p-3">
                        if *is_loading {
                            { "Logging in..." }
                        } else {
                            { "Login" }
                        }
                    </button>
                </form>
            </div>
        </div>
    }
}
