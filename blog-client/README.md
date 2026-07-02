# blog-client

Универсальная клиентская библиотека для работы с блогом. Поддерживает два транспорта — HTTP REST и gRPC — через единый трейт `BlogApi`.

## Структура

```
blog-client/
├── src/
│   ├── lib.rs          # Реэкспорт публичного API
│   ├── types.rs        # Трейт BlogApi и enum BlogClient
│   ├── http_client.rs  # HTTP-реализация (reqwest)
│   ├── grpc_client.rs  # gRPC-реализация (tonic)
│   └── error.rs        # BlogClientError
```

## Быстрый старт

```rust
use blog_client::{BlogClient, BlogApi};

// HTTP-клиент
let mut client = BlogClient::http("http://localhost:8080/api".into());

// gRPC-клиент
let mut client = BlogClient::grpc("http://localhost:50051".into()).await?;

// Одинаковый API для обоих
let user = client.login("alice", "secret").await?;
let posts = client.list_posts(Some(10), Some(0)).await?;
```

## Трейт `BlogApi`

Единый интерфейс, реализованный обоими транспортами:

| Метод | Описание | Auth |
|-------|----------|------|
| `register(username, email, password)` | Регистрация; сохраняет токен внутри клиента | — |
| `login(username, password)` | Вход; сохраняет токен внутри клиента | — |
| `list_posts(limit, offset)` | Список постов с пагинацией | — |
| `get_post(id)` | Получить пост по ID, `None` если не найден | — |
| `create_post(title, content)` | Создать пост | JWT |
| `update_post(id, title, content)` | Обновить пост | JWT |
| `delete_post(id)` | Удалить пост | JWT |

Токен сохраняется автоматически после `register` / `login` и подставляется в последующие запросы. Явная установка через `client.set_token(token)`.

## `BlogClientError`

```rust
pub enum BlogClientError {
    Http(reqwest::Error),          // сетевая ошибка HTTP
    GrpcTransport(tonic::transport::Error), // ошибка подключения gRPC
    GrpcStatus { code, message },  // gRPC-статус без семантического маппинга
    NotFound(String),
    Unauthorized,
    PermissionDenied,
    InvalidRequest(String),
    Internal(String),
}
```

`tonic::Status` конвертируется в семантические варианты автоматически: `NOT_FOUND` → `NotFound`, `UNAUTHENTICATED` → `Unauthorized`, `PERMISSION_DENIED` → `PermissionDenied`, `INVALID_ARGUMENT` → `InvalidRequest`.

## HTTP-клиент

Реализует `BlogApi` через REST API сервера (`blog-server`). Базовый URL передаётся при создании:

```rust
let mut client = BlogClient::http("http://localhost:8080/api".into());
```

Таймауты: подключение 5 с, запрос 15 с.

HTTP-статусы маппятся в `BlogClientError`: `401` → `Unauthorized`, `403` → `PermissionDenied`, `404` → `NotFound`, `400`/`422` → `InvalidRequest`, остальные → `Internal`. Тело ошибки читается из `{ "error": "..." }`.

## gRPC-клиент

Реализует `BlogApi` через gRPC-сервис `BlogService` (определён в `blog-proto`):

```rust
let mut client = BlogClient::grpc("http://localhost:50051".into()).await?;
```

JWT передаётся в metadata: `authorization: Bearer <token>`. Для защищённых методов (`create_post`, `update_post`, `delete_post`) токен добавляется автоматически.

## Использование в других крейтах

```toml
[dependencies]
blog-client = { path = "../blog-client" }
```

```rust
use blog_client::{BlogApi, BlogClient, BlogClientError};
```
