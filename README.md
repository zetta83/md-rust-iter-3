# Blog Platform

Учебный проект — полноценная система блога на Rust, оформленная как Cargo workspace из шести крейтов.

## Крейты

| Крейт | Описание | Статус |
|-------|----------|--------|
| [`blog-server`](./blog-server) | HTTP + gRPC сервер, PostgreSQL | Готово |
| [`blog-proto`](./blog-proto) | Protobuf-определения и Tonic-код | Готово |
| [`blog-client`](./blog-client) | Клиентская библиотека: HTTP + gRPC | Готово |
| [`blog-cli`](./blog-cli) | CLI-инструмент | Готово |
| [`blog-wasm`](./blog-wasm) | WASM-фронтенд | Готово |
| [`blog-dto`](./blog-dto) | Общие DTO-типы (server + wasm) | Готово |

## Быстрый старт

### Требования

- Rust 1.85+
- Docker и Docker Compose
- [`sqlx-cli`](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli): `cargo install sqlx-cli`
- [`trunk`](https://trunkrs.dev/) (для WASM-фронтенда): `cargo install trunk`
- WASM-цель: `rustup target add wasm32-unknown-unknown`

### Запуск бэкенда

```bash
# 1. Поднять базу данных
docker compose up -d

# 2. Создать .env в корне workspace
cp .env.example .env   # или создать вручную, см. ниже

# 3. Запустить сервер (миграции применяются автоматически)
cargo run -p blog_server
```

### Запуск фронтенда

```bash
# Из директории blog-wasm/
cd blog-wasm && trunk serve -p 8000
```

Откроется `http://localhost:8000`. Сервер должен быть запущен на порту `8080`.

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
RUST_LOG=info
ALLOWED_ORIGINS=http://localhost:8000
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
