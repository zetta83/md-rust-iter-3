mod api;
mod auth;
mod posts;
mod storage;

use blog_dto::User;
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct AppCtx {
    pub token: Signal<Option<String>>,
    pub user: Signal<Option<User>>,
    pub refresh: Signal<u32>,
}

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let token = use_signal(storage::load_token);
    let user = use_signal(storage::load_user);
    let refresh = use_signal(|| 0u32);

    use_context_provider(|| AppCtx {
        token,
        user,
        refresh,
    });

    rsx! {
        Header {}
        main {
            div {
                if user.read().is_none() {
                    auth::AuthPanel {}
                } else {
                    posts::CreatePostPanel {}
                }
            }
            div { class: "posts-col",
                h2 { "Посты" }
                posts::PostList {}
            }
        }
    }
}

#[component]
fn Header() -> Element {
    let mut ctx = use_context::<AppCtx>();
    let label = ctx
        .user
        .read()
        .as_ref()
        .map(|u| u.username.clone())
        .unwrap_or_else(|| "Не авторизован".into());
    let logged_in = ctx.user.read().is_some();

    rsx! {
        header {
            h1 { "Blog Platform" }
            div { id: "auth-status",
                span { "{label}" }
                if logged_in {
                    button {
                        id: "btn-logout",
                        onclick: move |_| {
                            ctx.token.set(None);
                            ctx.user.set(None);
                            storage::clear_token();
                            storage::clear_user();
                            *ctx.refresh.write() += 1;
                        },
                        "Выйти"
                    }
                }
            }
        }
    }
}
