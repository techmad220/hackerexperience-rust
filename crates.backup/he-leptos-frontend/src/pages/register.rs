//! Registration Page - New user signup

use leptos::*;
use serde::{Deserialize, Serialize};
use crate::api::register;

#[derive(Clone, Default, Serialize, Deserialize)]
struct RegisterForm {
    username: String,
    email: String,
    password: String,
    confirm_password: String,
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    let (form, set_form) = create_signal(RegisterForm::default());
    let (errors, set_errors) = create_signal(Vec::<String>::new());
    let (is_loading, set_is_loading) = create_signal(false);
    let (success, set_success) = create_signal(false);

    let validate_form = move || -> Result<(), Vec<String>> {
        let mut validation_errors = Vec::new();
        let current_form = form.get();

        if current_form.username.len() < 3 {
            validation_errors.push("Username must be at least 3 characters".to_string());
        }

        if !current_form.email.contains('@') {
            validation_errors.push("Invalid email address".to_string());
        }

        if current_form.password.len() < 8 {
            validation_errors.push("Password must be at least 8 characters".to_string());
        }

        if current_form.password != current_form.confirm_password {
            validation_errors.push("Passwords do not match".to_string());
        }

        if validation_errors.is_empty() {
            Ok(())
        } else {
            Err(validation_errors)
        }
    };

    let handle_register = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        match validate_form() {
            Ok(()) => {
                set_is_loading(true);
                set_errors(Vec::new());

                let current_form = form.get();
                spawn_local(async move {
                    match register(&current_form.username, &current_form.email, &current_form.password).await {
                        Ok(_) => {
                            set_success(true);
                            set_is_loading(false);
                        }
                        Err(e) => {
                            set_errors(vec![format!("Registration failed: {}", e)]);
                            set_is_loading(false);
                        }
                    }
                });
            }
            Err(validation_errors) => {
                set_errors(validation_errors);
            }
        }
    };

    view! {
        <div class="register-container">
            <div class="register-box">
                <h2>"Create Your Hacker Account"</h2>

                {move || if success.get() {
                    view! {
                        <div class="alert alert-success">
                            <h3>"Registration Successful!"</h3>
                            <p>"Your account has been created. You can now "</p>
                            <a href="/login" class="btn btn-primary">"Login"</a>
                        </div>
                    }
                } else {
                    view! {
                        <form on:submit=handle_register class="register-form">
                            <div class="form-group">
                                <label>"Username"</label>
                                <input
                                    type="text"
                                    class="form-control"
                                    placeholder="Choose your hacker name"
                                    on:input=move |ev| {
                                        let mut current = form.get();
                                        current.username = event_target_value(&ev);
                                        set_form(current);
                                    }
                                />
                            </div>

                            <div class="form-group">
                                <label>"Email"</label>
                                <input
                                    type="email"
                                    class="form-control"
                                    placeholder="your@email.com"
                                    on:input=move |ev| {
                                        let mut current = form.get();
                                        current.email = event_target_value(&ev);
                                        set_form(current);
                                    }
                                />
                            </div>

                            <div class="form-group">
                                <label>"Password"</label>
                                <input
                                    type="password"
                                    class="form-control"
                                    placeholder="Minimum 8 characters"
                                    on:input=move |ev| {
                                        let mut current = form.get();
                                        current.password = event_target_value(&ev);
                                        set_form(current);
                                    }
                                />
                            </div>

                            <div class="form-group">
                                <label>"Confirm Password"</label>
                                <input
                                    type="password"
                                    class="form-control"
                                    placeholder="Re-enter password"
                                    on:input=move |ev| {
                                        let mut current = form.get();
                                        current.confirm_password = event_target_value(&ev);
                                        set_form(current);
                                    }
                                />
                            </div>

                            {move || if !errors.get().is_empty() {
                                view! {
                                    <div class="alert alert-danger">
                                        <ul>
                                            {errors.get().into_iter().map(|error| {
                                                view! { <li>{error}</li> }
                                            }).collect_view()}
                                        </ul>
                                    </div>
                                }
                            } else {
                                view! { <div></div> }
                            }}

                            <button
                                type="submit"
                                class="btn btn-success btn-block"
                                disabled=move || is_loading.get()
                            >
                                {move || if is_loading.get() {
                                    "Creating Account..."
                                } else {
                                    "Create Account"
                                }}
                            </button>

                            <div class="register-links">
                                <p>"Already have an account? "<a href="/login">"Login here"</a></p>
                            </div>
                        </form>
                    }
                }}
            </div>
        </div>
    }
}