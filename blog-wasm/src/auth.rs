use crate::{storage, AppCtx};
use dioxus::prelude::*;

#[component]
pub fn AuthPanel() -> Element {
    let mut tab = use_signal(|| "login");
    rsx! {
        div { class: "panel",
            div { class: "tabs",
                button {
                    class: if tab() == "login" { "tab-btn active" } else { "tab-btn" },
                    onclick: move |_| tab.set("login"),
                    "Вход"
                }
                button {
                    class: if tab() == "register" { "tab-btn active" } else { "tab-btn" },
                    onclick: move |_| tab.set("register"),
                    "Регистрация"
                }
            }
            if tab() == "login" { LoginForm {} } else { RegisterForm {} }
        }
    }
}

#[component]
fn LoginForm() -> Element {
    let mut ctx = use_context::<AppCtx>();
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut loading = use_signal(|| false);

    rsx! {
        form {
            onsubmit: move |e| {
                e.prevent_default();
                let u = username();
                let p = password();
                if u.is_empty() || p.is_empty() {
                    error.set("Заполните все поля".into());
                    return;
                }
                error.set(String::new());
                loading.set(true);
                spawn(async move {
                    match crate::api::login(&u, &p).await {
                        Ok(auth) => {
                            storage::save_token(&auth.token);
                            storage::save_user(&auth.user);
                            ctx.token.set(Some(auth.token));
                            ctx.user.set(Some(auth.user));
                            *ctx.refresh.write() += 1;
                        }
                        Err(e) => error.set(e),
                    }
                    loading.set(false);
                });
            },
            label { "Имя пользователя" }
            input { r#type: "text", value: "{username}",
                oninput: move |e| username.set(e.value()) }
            label { "Пароль" }
            input { r#type: "password", value: "{password}",
                oninput: move |e| password.set(e.value()) }
            button { r#type: "submit", class: "btn-primary", disabled: loading(), "Войти" }
            if !error().is_empty() {
                div { class: "msg error", "{error}" }
            }
        }
    }
}

#[component]
fn RegisterForm() -> Element {
    let mut ctx = use_context::<AppCtx>();
    let mut username = use_signal(String::new);
    let mut email    = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error    = use_signal(String::new);
    let mut loading  = use_signal(|| false);

    rsx! {
        form {
            onsubmit: move |e| {
                e.prevent_default();
                let u  = username();
                let em = email();
                let p  = password();
                if u.is_empty() || em.is_empty() || p.is_empty() {
                    error.set("Заполните все поля".into());
                    return;
                }
                error.set(String::new());
                loading.set(true);
                spawn(async move {
                    match crate::api::register(&u, &em, &p).await {
                        Ok(auth) => {
                            storage::save_token(&auth.token);
                            storage::save_user(&auth.user);
                            ctx.token.set(Some(auth.token));
                            ctx.user.set(Some(auth.user));
                            *ctx.refresh.write() += 1;
                        }
                        Err(e) => error.set(e),
                    }
                    loading.set(false);
                });
            },
            label { "Имя пользователя" }
            input { r#type: "text", value: "{username}",
                oninput: move |e| username.set(e.value()) }
            label { "Email" }
            input { r#type: "email", value: "{email}",
                oninput: move |e| email.set(e.value()) }
            label { "Пароль" }
            input { r#type: "password", value: "{password}",
                oninput: move |e| password.set(e.value()) }
            button { r#type: "submit", class: "btn-primary", disabled: loading(),
                "Зарегистрироваться" }
            if !error().is_empty() {
                div { class: "msg error", "{error}" }
            }
        }
    }
}
