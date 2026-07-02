# blog-cli

Утилита командной строки для работы с [blog-server](../blog-server). Поддерживает HTTP и gRPC транспорт.

## Установка

```bash
cargo install --path .
```

Или запустить напрямую из корня workspace:

```bash
cargo run -p blog-cli -- <команда>
```

## Использование

По умолчанию используется HTTP клиент (`http://localhost:8080/api`).  
Флаг `--grpc` переключает на gRPC (`http://localhost:50051`).  
Флаг `--server <адрес>` переопределяет адрес сервера.

### Аутентификация

```bash
# Регистрация (токен сохраняется автоматически в ~/.blog_token)
blog-cli register --username "ivan" --email "ivan@example.com" --password "secret123"

# Вход
blog-cli login --username "ivan" --password "secret123"
```

### Посты

```bash
# Создать пост (требует токен)
blog-cli create --title "Мой первый пост" --content "Содержание поста"

# Получить пост по id
blog-cli get --id 1

# Список постов
blog-cli list
blog-cli list --limit 20 --offset 0

# Обновить пост (можно передать только title или только content)
blog-cli update --id 1 --title "Новый заголовок"
blog-cli update --id 1 --content "Новое содержание"
blog-cli update --id 1 --title "Новый заголовок" --content "Новое содержание"

# Удалить пост
blog-cli delete --id 1
```

### gRPC

```bash
# Те же команды с флагом --grpc
blog-cli --grpc register --username "ivan" --email "ivan@example.com" --password "secret123"
blog-cli --grpc create --title "Пост через gRPC" --content "Содержание"
blog-cli --grpc list
```

### Кастомный адрес сервера

```bash
blog-cli --server "http://prod.example.com/api" list
blog-cli --grpc --server "http://prod.example.com:50051" list
```

## Токен

После `register` или `login` токен сохраняется в `~/.blog_token`.  
Команды, требующие аутентификации (`create`, `update`, `delete`), читают токен оттуда автоматически.
