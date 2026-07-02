# blog-dto

Общие DTO-типы (Data Transfer Objects), используемые совместно сервером (`blog-server`) и WASM-фронтендом (`blog-wasm`). Единственная зависимость — `serde`.

## Типы

| Тип | Описание |
|-----|----------|
| `User` | `{ id, username, email }` — данные пользователя |
| `AuthResponse` | `{ token, user }` — ответ на login/register |
| `PostResponse` | `{ id, title, content, author_id }` — пост |
| `ListPostsResponse` | `{ posts, total, limit, offset }` — страница постов |
| `ErrorBody` | `{ error }` — тело HTTP-ошибки |

Все типы реализуют `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`.

## Использование

### В `blog-server`

`blog-server/src/presentation/dto.rs` реэкспортирует типы:

```rust
pub use blog_dto::{AuthResponse, ListPostsResponse, PostResponse, User};
```

`impl From<Post> for PostResponse` остаётся в `blog-server` (правило сирот: `Post` — локальный тип).

### В `blog-wasm`

`blog-wasm/src/api.rs` использует типы напрямую для десериализации JSON-ответов:

```rust
use blog_dto::{AuthResponse, ListPostsResponse, PostResponse};

pub async fn login(username: &str, password: &str) -> Result<AuthResponse, String> { ... }
pub async fn load_posts(limit: u32, offset: u32) -> Result<ListPostsResponse, String> { ... }
```

## Подключение

```toml
[dependencies]
blog-dto = { path = "../blog-dto" }
```

Или через workspace:

```toml
[dependencies]
blog-dto = { workspace = true }
```
