# blog-server

HTTP + gRPC сервер для системы блога. Actix-Web + Tonic, PostgreSQL через SQLx, JWT-аутентификация (HS256), хэширование паролей через Argon2id.

## Архитектура

Слоистая архитектура со строгим направлением зависимостей:

```
presentation → application → domain ← data
```

### Слои

**`domain/`** — чистые типы, без I/O

| Файл | Содержимое |
|------|-----------|
| `base.rs` | Value-объекты: `Email` (валидация формата), `Password` (хэширование Argon2id при создании), `Pagination` |
| `user.rs` | Сущность `User` |
| `post.rs` | Сущность `Post` |
| `error.rs` | `DomainError`, `UserError`, `PostError`; `DomainError` реализует `ResponseError` для маппинга в HTTP-статусы |

**`data/`** — трейты репозиториев и PostgreSQL-реализации

| Файл | Содержимое |
|------|-----------|
| `user_repository.rs` | Трейт `UserRepository` + реализация для `PgRepository` |
| `post_repository.rs` | Трейт `PostRepository` + реализация для `PgRepository` |
| `pg_repository.rs` | Структура `PgRepository(PgPool)`, которую используют оба трейта |

**`application/`** — бизнес-логика

| Файл | Содержимое |
|------|-----------|
| `auth_service.rs` | `AuthService<R: UserRepository>` — регистрация, вход, генерация токена; экстрактор `AuthenticatedUser` для Actix |
| `blog_service.rs` | `BlogService<R: PostRepository>` — CRUD постов |

**`presentation/`** — HTTP и gRPC интерфейсы

| Файл | Содержимое |
|------|-----------|
| `http_handlers.rs` | Конфигурация роутера Actix-Web, все маршруты под `/api` |
| `handler/auth.rs` | `POST /api/auth/register`, `POST /api/auth/login` |
| `handler/post.rs` | CRUD-маршруты для `/api/posts` |
| `middleware.rs` | `JwtAuthMiddleware` — верификация токена, добавляет `AuthenticatedUser` в extensions запроса |
| `dto.rs` | Типы запросов/ответов; `Email` и `Password` десериализуются с валидацией |
| `grpc_service.rs` | `BlogServiceImpl` — реализация gRPC-сервиса |

**`infrastructure/`**

| Файл | Содержимое |
|------|-----------|
| `config.rs` | `AppConfig::from_env()` — чтение переменных окружения |
| `database.rs` | Создание пула соединений, запуск SQLx-миграций |
| `jwt.rs` | `JwtKeys` — кодирование/верификация HS256; функции Argon2id |
| `logging.rs` | Настройка `tracing-subscriber` |

## REST API

Базовый URL: `http://localhost:8080/api`

### Аутентификация

| Метод | Путь | Тело | Ответ | Auth |
|-------|------|------|-------|------|
| POST | `/auth/register` | `{ username, email, password }` | `201 { token, user }` | — |
| POST | `/auth/login` | `{ username, password }` | `200 { token, user }` | — |

### Посты

| Метод | Путь | Параметры / тело | Ответ | Auth |
|-------|------|-----------------|-------|------|
| GET | `/posts` | `?page=0&limit=10` | `200 { posts, total, limit, offset }` | — |
| GET | `/posts/{id}` | — | `200 { id, title, content, author_id }` | — |
| POST | `/posts` | `{ title, content }` | `201 { id, title, content, author_id }` | JWT |
| PUT | `/posts/{id}` | `{ title, content }` | `200 { id, title, content, author_id }` | JWT |
| DELETE | `/posts/{id}` | — | `204 No Content` | JWT |

Для защищённых маршрутов передавать заголовок: `Authorization: Bearer <token>`

### Коды ошибок

| Статус | Ситуация |
|--------|----------|
| 400 | Ошибка валидации |
| 401 | Невалидный или отсутствующий токен |
| 403 | Нет прав на операцию |
| 404 | Ресурс не найден |
| 500 | Внутренняя ошибка сервера |

Тело ошибки: `{ "error": "сообщение" }`

## gRPC API

Адрес: `localhost:50051`  
Определение: [`blog-proto/proto/blog.proto`](../blog-proto/proto/blog.proto)

JWT передаётся в метаданных запроса: `authorization: Bearer <token>`

| RPC | Запрос | Ответ | Auth |
|-----|--------|-------|------|
| `Register` | `RegisterRequest` | `AuthResponse` | — |
| `Login` | `LoginRequest` | `AuthResponse` | — |
| `ListPosts` | `ListPostsRequest` | `ListPostsResponse` | — |
| `GetPost` | `GetPostRequest` | `PostResponse` | — |
| `CreatePost` | `CreatePostRequest` | `PostResponse` | JWT |
| `UpdatePost` | `UpdatePostRequest` | `PostResponse` | JWT |
| `DeletePost` | `DeletePostRequest` | `DeletePostResponse` | JWT |

## База данных

### Схема

**`users`**
```sql
id            BIGSERIAL PRIMARY KEY
username      VARCHAR NOT NULL UNIQUE
email         VARCHAR NOT NULL UNIQUE
password_hash VARCHAR NOT NULL
created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
```

**`posts`**
```sql
id         BIGSERIAL PRIMARY KEY
title      VARCHAR  NOT NULL
content    TEXT     NOT NULL
author_id  BIGINT   NOT NULL REFERENCES users(id) ON DELETE CASCADE
created_at TIMESTAMPTZ NOT NULL DEFAULT now()
updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
```

### Миграции

Миграции применяются автоматически при старте сервера. Запустить вручную:

```bash
cd blog-server && sqlx migrate run
```

Создать новую миграцию:

```bash
cd blog-server && sqlx migrate add <name>
```

## Запуск

```bash
# Поднять PostgreSQL
docker compose up -d

# Запустить сервер
cargo run -p blog_server
```

Сервер запускает HTTP и gRPC параллельно через `tokio::select!`.

### Переменные окружения

| Переменная | Обязательная | По умолчанию | Описание |
|-----------|-------------|-------------|----------|
| `DATABASE_URL` | Да | — | `postgres://user:pass@host/db` |
| `JWT_SECRET` | Да | — | Секрет для подписи HS256 токенов |
| `HOST` | Нет | `127.0.0.1` | Адрес HTTP-сервера |
| `PORT` | Нет | `8080` | Порт HTTP-сервера |
| `GRPC_HOST` | Нет | `127.0.0.1` | Адрес gRPC-сервера |
| `GRPC_PORT` | Нет | `50051` | Порт gRPC-сервера |
| `ALLOWED_ORIGINS` | Нет | `*` | CORS: список через запятую |
| `RUST_LOG` | Нет | — | Уровень логирования, например `info,blog_server=debug` |

## SQLx offline-режим

Проект использует `sqlx::query!` / `sqlx::query_as!` с проверкой на этапе компиляции. Кэш запросов хранится в `.sqlx/`. После добавления нового запроса нужно обновить кэш (при запущенной БД):

```bash
cargo sqlx prepare --workspace
```
