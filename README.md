# Blog Platform

Учебный проект — полноценная система блога на Rust, оформленная как Cargo workspace из пяти крейтов.

## Крейты

| Крейт | Описание | Статус |
|-------|----------|--------|
| [`blog-server`](./blog-server) | HTTP + gRPC сервер, PostgreSQL | Готово |
| [`blog-proto`](./blog-proto) | Protobuf-определения и Tonic-код | Готово |
| [`blog-client`](./blog-client) | Клиентская библиотека: HTTP + gRPC | Готово |
| `blog-cli` | CLI-инструмент | Заглушка |
| `blog-wasm` | WASM-фронтенд | Заглушка |

## Быстрый старт

### Требования

- Rust 1.85+
- Docker и Docker Compose
- [`sqlx-cli`](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli): `cargo install sqlx-cli`

### Запуск

```bash
# 1. Поднять базу данных
docker compose up -d

# 2. Создать .env в корне workspace
cp .env.example .env   # или создать вручную, см. ниже

# 3. Запустить сервер (миграции применяются автоматически)
cargo run -p blog_server
```

### Переменные окружения

Создать `.env` в корне workspace:

```env
DATABASE_URL=postgres://postgres:postgres@localhost/blog_app
JWT_SECRET=your-secret-key

# Опциональные
HOST=127.0.0.1
PORT=8080
GRPC_HOST=127.0.0.1
GRPC_PORT=50051
ALLOWED_ORIGINS=http://localhost:3000,http://localhost:5173
```

## Команды

```bash
# Сборка всего workspace
cargo build

# Проверка компиляции без сборки
cargo check

# Clippy по всему workspace
cargo clippy

# Обновить SQLx offline-кэш (нужна запущенная БД)
cargo sqlx prepare --workspace
```
