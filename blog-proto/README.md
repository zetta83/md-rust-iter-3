# blog-proto

Protobuf-определения и сгенерированный Tonic-код для gRPC-сервиса блога.

## Структура

```
blog-proto/
├── proto/
│   └── blog.proto      # Определения сервиса и сообщений
├── src/
│   └── lib.rs          # Подключение сгенерированного кода
└── build.rs            # Вызов tonic-prost-build
```

## Сервис

`BlogService` — единственный gRPC-сервис. JWT для защищённых методов передаётся в метаданных запроса: `authorization: Bearer <token>`.

### Методы

| RPC | Описание | Auth |
|-----|----------|------|
| `Register(RegisterRequest)` | Регистрация нового пользователя | — |
| `Login(LoginRequest)` | Вход, получение JWT-токена | — |
| `ListPosts(ListPostsRequest)` | Список постов с пагинацией | — |
| `GetPost(GetPostRequest)` | Получение поста по ID | — |
| `CreatePost(CreatePostRequest)` | Создание поста | JWT |
| `UpdatePost(UpdatePostRequest)` | Обновление поста | JWT |
| `DeletePost(DeletePostRequest)` | Удаление поста | JWT |

### Сообщения

**Auth**

```protobuf
message RegisterRequest { string username = 1; string email = 2; string password = 3; }
message LoginRequest    { string username = 1; string password = 2; }
message AuthResponse    { string token = 1; User user = 2; }
message User            { int64 id = 1; string username = 2; string email = 3; }
```

**Posts**

```protobuf
message ListPostsRequest  { uint32 page = 1; uint32 limit = 2; }  // page=0, limit=10 по умолчанию
message ListPostsResponse { repeated PostResponse posts = 1; int64 total = 2; uint32 limit = 3; uint32 offset = 4; }
message GetPostRequest    { int64 id = 1; }
message CreatePostRequest { string title = 1; string content = 2; }
message UpdatePostRequest { int64 id = 1; string title = 2; string content = 3; }
message DeletePostRequest { int64 id = 1; }
message DeletePostResponse {}
message PostResponse      { int64 id = 1; string title = 2; string content = 3; int64 author_id = 4; }
```

## Генерация кода

Код генерируется автоматически при сборке через `build.rs`:

```rust
tonic_prost_build::configure()
    .build_server(true)
    .build_client(true)
    .compile_protos(&["proto/blog.proto"], &["proto"])?;
```

Генерируются серверные трейты (`blog_service_server::BlogService`) и клиентские стабы (`blog_service_client::BlogServiceClient`). Для ручной перегенерации достаточно `cargo build -p blog-proto`.

## Использование в других крейтах

```toml
[dependencies]
blog-proto = { path = "../blog-proto" }
```

```rust
// Серверная сторона
use blog_proto::blog::blog_service_server::{BlogService, BlogServiceServer};
use blog_proto::blog::{AuthResponse, RegisterRequest};

// Клиентская сторона
use blog_proto::blog::blog_service_client::BlogServiceClient;
```

## Изменение proto-файла

1. Отредактировать `proto/blog.proto`
2. Пересобрать крейт: `cargo build -p blog-proto`
3. Обновить реализации сервера в `blog-server/src/presentation/grpc_service.rs`
