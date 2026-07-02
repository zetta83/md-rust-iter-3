# blog-wasm

WASM-фронтенд для системы блога, написанный на Rust с использованием фреймворка [Dioxus](https://dioxuslabs.com/). Компилируется в WebAssembly и запускается в браузере без единой строки JavaScript.

## Стек

| Зависимость | Роль |
|-------------|------|
| `dioxus 0.7` + feature `web` | Реактивный UI-фреймворк, рендеринг в DOM |
| `reqwest 0.12` | HTTP-запросы к `blog-server` через Fetch API |
| `gloo-storage` | Доступ к `localStorage` браузера |
| `blog-dto` | Общие DTO-типы, разделяемые с сервером |
| `serde` / `serde_json` | Сериализация JSON |
| `trunk` | Сборщик и dev-сервер для WASM |

## Структура

```
blog-wasm/
├── src/
│   ├── main.rs      # точка входа, AppCtx, компоненты App и Header
│   ├── api.rs       # HTTP-функции к /api/* (login, register, CRUD постов)
│   ├── auth.rs      # компоненты AuthPanel, LoginForm, RegisterForm
│   ├── posts.rs     # компоненты PostList, PostCard, CreatePostPanel, Pagination
│   └── storage.rs   # load/save/clear для token и user в localStorage
├── index.html       # шаблон trunk; точка монтирования #main
└── style.css        # стили приложения
```

## Архитектура

### AppCtx — глобальный контекст

```rust
#[derive(Clone, Copy)]
pub struct AppCtx {
    pub token:   Signal<Option<String>>,
    pub user:    Signal<Option<User>>,
    pub refresh: Signal<u32>,
}
```

`AppCtx` регистрируется в корневом компоненте `App` через `use_context_provider` и читается в любом дочернем компоненте через `use_context::<AppCtx>()`. Все три поля — `Signal<T>`, поэтому изменение любого из них автоматически перерисовывает зависимые компоненты.

### Реактивное обновление списка

`PostList` использует `use_resource`, который реагирует на `ctx.refresh`:

```rust
let posts_res = use_resource(move || async move {
    let _r = (ctx.refresh)(); // подписка на сигнал → перезагрузка при изменении
    api::load_posts(PAGE_SIZE, page() * PAGE_SIZE).await
});
```

После создания, редактирования или удаления поста вызов `*ctx.refresh.write() += 1` инвалидирует ресурс и список обновляется.

### Поток аутентификации

1. Пользователь заполняет `LoginForm` или `RegisterForm`
2. `api::login` / `api::register` → `POST /api/auth/*`
3. Получившийся `AuthResponse` сохраняется в `localStorage` (via `storage::save_token`, `storage::save_user`) и в `ctx.token` / `ctx.user`
4. При перезагрузке страницы `storage::load_token` / `storage::load_user` восстанавливают сессию
5. Кнопка «Выйти» сбрасывает сигналы и очищает `localStorage`

## Запуск

### Требования

```bash
# Установить trunk
cargo install trunk

# Добавить WASM-цель
rustup target add wasm32-unknown-unknown
```

### Dev-сервер

```bash
# Из директории blog-wasm/
trunk serve -p 8000
```

Приложение откроется на `http://localhost:8000`. Trunk следит за изменениями и пересобирает автоматически.

> **Важно:** `blog-server` должен быть запущен на `http://localhost:8080` и разрешать CORS для `http://localhost:8000` (`ALLOWED_ORIGINS=http://localhost:8000`).

### Production-сборка

```bash
trunk build --release
```

Результат в `dist/` — статические файлы, готовые для деплоя на любой HTTP-сервер.

## Функциональность

| Возможность | Требует авторизации |
|-------------|-------------------|
| Просмотр списка постов с пагинацией | Нет |
| Регистрация / вход | Нет |
| Создание поста | Да |
| Редактирование поста | Да (только автор) |
| Удаление поста | Да (только автор) |
| Выход из аккаунта | Да |
