use crate::{api, AppCtx};
use blog_dto::PostResponse;
use dioxus::prelude::*;

const PAGE_SIZE: u32 = 10;

#[component]
pub fn CreatePostPanel() -> Element {
    let mut ctx = use_context::<AppCtx>();
    let mut title   = use_signal(String::new);
    let mut content = use_signal(String::new);
    let mut error   = use_signal(String::new);
    let mut success = use_signal(String::new);
    let mut loading = use_signal(|| false);

    rsx! {
        div { class: "panel",
            h2 { "Новый пост" }
            form {
                onsubmit: move |e| {
                    e.prevent_default();
                    let t = title();
                    let c = content();
                    if t.is_empty() || c.is_empty() {
                        error.set("Заполните все поля".into());
                        return;
                    }
                    let token = ctx.token.read().clone().unwrap_or_default();
                    error.set(String::new());
                    success.set(String::new());
                    loading.set(true);
                    spawn(async move {
                        match api::create_post(&token, &t, &c).await {
                            Ok(post) => {
                                success.set(format!("Пост \"{}\" опубликован", post.title));
                                title.set(String::new());
                                content.set(String::new());
                                *ctx.refresh.write() += 1;
                            }
                            Err(e) => error.set(e),
                        }
                        loading.set(false);
                    });
                },
                label { "Заголовок" }
                input { r#type: "text", value: "{title}",
                    oninput: move |e| title.set(e.value()) }
                label { "Содержание" }
                textarea { value: "{content}",
                    oninput: move |e| content.set(e.value()) }
                button { r#type: "submit", class: "btn-primary", disabled: loading(),
                    "Опубликовать" }
                if !error().is_empty() {
                    div { class: "msg error", "{error}" }
                }
                if !success().is_empty() {
                    div { class: "msg ok", "{success}" }
                }
            }
        }
    }
}

#[component]
pub fn PostList() -> Element {
    let ctx  = use_context::<AppCtx>();
    let page = use_signal(|| 0u32);

    let posts_res = use_resource(move || async move {
        let _r = (ctx.refresh)(); // subscribe to refresh
        let p  = page();
        api::load_posts(PAGE_SIZE, p * PAGE_SIZE).await
    });

    match &*posts_res.value().read() {
        None => rsx! { div { class: "empty", "Загрузка..." } },
        Some(Err(e)) => rsx! { div { class: "empty", "Ошибка: {e}" } },
        Some(Ok(data)) => {
            let posts = data.posts.clone();
            let total = data.total;
            rsx! {
                if posts.is_empty() {
                    div { class: "empty", "Постов ещё нет" }
                }
                for post in posts {
                    PostCard { key: "{post.id}", post }
                }
                Pagination { total, page }
            }
        }
    }
}

#[component]
fn Pagination(total: i64, mut page: Signal<u32>) -> Element {
    let pages = (total as f64 / PAGE_SIZE as f64).ceil() as u32;
    if pages <= 1 {
        return rsx! {};
    }
    rsx! {
        div { class: "pagination",
            for i in 0..pages {
                button {
                    class: if page() == i { "cur" } else { "" },
                    onclick: move |_| page.set(i),
                    "{i + 1}"
                }
            }
        }
    }
}

#[component]
fn PostCard(post: PostResponse) -> Element {
    let mut ctx = use_context::<AppCtx>();
    let mut is_editing   = use_signal(|| false);
    let mut edit_title   = use_signal(|| post.title.clone());
    let mut edit_content = use_signal(|| post.content.clone());
    let mut edit_error   = use_signal(String::new);

    let is_owner = ctx.user.read().as_ref().map(|u| u.id) == Some(post.author_id);
    let post_id  = post.id;

    rsx! {
        div { class: "post-card", id: "pc-{post_id}",
            div { class: "post-title", "{post.title}" }
            div { class: "post-meta", "Автор #{post.author_id} · Пост #{post_id}" }

            if !is_editing() {
                div { class: "post-body", "{post.content}" }
                if is_owner {
                    div { class: "post-actions",
                        button {
                            class: "btn-sm btn-edit",
                            onclick: move |_| {
                                edit_title.set(post.title.clone());
                                edit_content.set(post.content.clone());
                                is_editing.set(true);
                            },
                            "Редактировать"
                        }
                        button {
                            class: "btn-sm btn-del",
                            onclick: move |_| {
                                let token = ctx.token.read().clone().unwrap_or_default();
                                spawn(async move {
                                    if api::delete_post(&token, post_id).await.is_ok() {
                                        *ctx.refresh.write() += 1;
                                    }
                                });
                            },
                            "Удалить"
                        }
                    }
                }
            } else {
                div { class: "edit-area open",
                    input {
                        r#type: "text",
                        value: "{edit_title}",
                        oninput: move |e| edit_title.set(e.value())
                    }
                    textarea {
                        value: "{edit_content}",
                        oninput: move |e| edit_content.set(e.value())
                    }
                    div { class: "edit-btns",
                        button {
                            class: "btn-sm btn-save",
                            onclick: move |_| {
                                let t = edit_title();
                                let c = edit_content();
                                if t.is_empty() || c.is_empty() {
                                    edit_error.set("Заполните все поля".into());
                                    return;
                                }
                                let token = ctx.token.read().clone().unwrap_or_default();
                                spawn(async move {
                                    match api::update_post(&token, post_id, &t, &c).await {
                                        Ok(_) => {
                                            is_editing.set(false);
                                            *ctx.refresh.write() += 1;
                                        }
                                        Err(e) => edit_error.set(e),
                                    }
                                });
                            },
                            "Сохранить"
                        }
                        button {
                            class: "btn-sm btn-cancel",
                            onclick: move |_| {
                                is_editing.set(false);
                                edit_error.set(String::new());
                            },
                            "Отмена"
                        }
                    }
                    if !edit_error().is_empty() {
                        div { class: "edit-msg", "{edit_error}" }
                    }
                }
            }
        }
    }
}
