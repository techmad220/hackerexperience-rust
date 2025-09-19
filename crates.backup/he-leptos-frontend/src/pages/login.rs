//! Login Page - Authentication interface

use leptos::*;
use leptos::html::Input;
use serde::{Deserialize, Serialize};
use crate::api::login;

#[derive(Clone, Default, Serialize, Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let (login_form, set_login_form) = create_signal(LoginForm::default());
    let (error_message, set_error_message) = create_signal(None::<String>);
    let (is_loading, set_is_loading) = create_signal(false);

    let username_input: NodeRef<Input> = create_node_ref();
    let password_input: NodeRef<Input> = create_node_ref();

    let handle_login = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let form = login_form.get();
        set_is_loading(true);
        set_error_message(None);

        spawn_local(async move {
            match login(&form.username, &form.password).await {
                Ok(response) => {
                    // Store token and redirect to dashboard
                    if let Some(window) = web_sys::window() {
                        let storage = window.local_storage().unwrap().unwrap();
                        storage.set_item("auth_token", &response.token).unwrap();
                        let _ = window.location().set_href("/dashboard");
                    }
                }
                Err(e) => {
                    set_error_message(Some(format!("Login failed: {}", e)));
                    set_is_loading(false);
                }
            }
        });
    };

    view! {
        <div class="login-container">
            <div class="login-box">
                <div class="login-logo">
                    <h1>"HackerExperience"</h1>
                    <p class="tagline">"Become the ultimate hacker"</p>
                </div>

                <form on:submit=handle_login class="login-form">
                    <div class="form-group">
                        <input
                            type="text"
                            placeholder="Username"
                            class="form-control"
                            node_ref=username_input
                            on:input=move |ev| {
                                let mut form = login_form.get();
                                form.username = event_target_value(&ev);
                                set_login_form(form);
                            }
                        />
                    </div>

                    <div class="form-group">
                        <input
                            type="password"
                            placeholder="Password"
                            class="form-control"
                            node_ref=password_input
                            on:input=move |ev| {
                                let mut form = login_form.get();
                                form.password = event_target_value(&ev);
                                set_login_form(form);
                            }
                        />
                    </div>

                    {move || error_message.get().map(|msg| view! {
                        <div class="alert alert-danger">{msg}</div>
                    })}

                    <button
                        type="submit"
                        class="btn btn-primary btn-block"
                        disabled=move || is_loading.get()
                    >
                        {move || if is_loading.get() {
                            "Logging in..."
                        } else {
                            "Login"
                        }}
                    </button>

                    <div class="login-links">
                        <a href="/register">"Create Account"</a>
                        <span>" | "</span>
                        <a href="/forgot-password">"Forgot Password?"</a>
                    </div>
                </form>

                <div class="login-footer">
                    <p>"By logging in, you agree to our Terms of Service"</p>
                </div>
            </div>
        </div>
    }
}