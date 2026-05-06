# OxideBlog

Полноценный pet-проект блога на Rust в одном `Cargo workspace`.

Включает:
- HTTP API (`actix-web`)
- gRPC API (`tonic`)
- PostgreSQL (`sqlx`)
- JWT-аутентификацию
- CLI-клиент
- WASM-фронтенд

## Состав workspace

| Крейт | Что делает |
|---|---|
| `blog-server` | Backend: HTTP (`:8080`) + gRPC (`:50051`), БД, JWT, миграции |
| `blog-client` | Библиотека-клиент с единым API для HTTP/gRPC |
| `blog-cli` | CLI для ручной проверки сценариев |
| `blog-wasm` | WASM-клиент для браузера + `index.html` |

`protoc` отдельно ставить не нужно: используется `protoc-bin-vendored`.

## Быстрый старт (рекомендуемый)

### 1) Поднять PostgreSQL через Docker

```bash
docker rm -f oxideblog-postgres 2>/dev/null
docker run --name oxideblog-postgres \
  -e POSTGRES_USER=bloguser \
  -e POSTGRES_PASSWORD=blogpass \
  -e POSTGRES_DB=blog_db \
  -p 5432:5432 \
  -d postgres:16
```

### 2) Создать `.env` в корне репозитория

```env
DATABASE_URL=postgres://bloguser:blogpass@localhost:5432/blog_db
JWT_SECRET=минимум_32_символа_секрета_для_jwt
```

> Можно использовать `blog-server/.env`, но если запускаете из корня (`cargo run -p blog-server`), удобнее держать `.env` именно в корне.

### 3) Собрать и запустить сервер

```bash
cargo build --workspace
cargo run -p blog-server
```

После запуска:
- HTTP: `http://127.0.0.1:8080`
- gRPC: `http://127.0.0.1:50051`

## Проверка API через CLI

Токен после `register` / `login` сохраняется в `.blog_token`.

```bash
# HTTP режим (по умолчанию)
cargo run -p blog-cli -- register --username ivan --email ivan@example.com --password secret123
cargo run -p blog-cli -- login --username ivan --password secret123
cargo run -p blog-cli -- create --title "Первый пост" --content "Текст поста"
cargo run -p blog-cli -- list --limit 10 --offset 0
cargo run -p blog-cli -- get --id 1
cargo run -p blog-cli -- update --id 1 --title "Обновлённый заголовок" --content "Новый текст"
cargo run -p blog-cli -- delete --id 1

# gRPC режим
cargo run -p blog-cli -- --grpc list --limit 10 --offset 0
```

## Проверка API через curl

Регистрация:

```bash
curl -s -X POST http://127.0.0.1:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"u1","email":"u1@example.com","password":"secret123"}'
```

Список постов:

```bash
curl -s "http://127.0.0.1:8080/api/posts?limit=10&offset=0"
```

## Запуск WASM-фронтенда

### 1) Установка (один раз)

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

### 2) Сборка WASM и запуск статики

Из корня проекта:

```bash
wasm-pack build blog-wasm --target web --out-dir pkg
python3 -m http.server 8000
```

Откройте `http://127.0.0.1:8000`.

В интерфейсе в поле Base URL должен быть backend:
- `http://127.0.0.1:8080`

## Частые проблемы

- **`password authentication failed for user ...`**
  - Проверьте `DATABASE_URL` и креды контейнера Postgres.

- **`404` на `POST /api/posts`**
  - Проверьте, что сервер перезапущен после изменений.
  - Без токена должен быть `401`, а не `404`.

- **`TypeError: can't convert ... to BigInt` (WASM)**
  - Обновите страницу после пересборки WASM (`Cmd+Shift+R`).
  - Убедитесь, что использована свежая сборка `pkg/`.

- **`/pkg/blog_wasm.js` 404**
  - Выполните `wasm-pack ... --out-dir pkg` из корня.
  - `python3 -m http.server` тоже запускайте из корня.

## Архитектура сервера

`blog-server/src` организован по clean architecture:
- `domain`
- `application`
- `data`
- `infrastructure`
- `presentation` (HTTP handlers, gRPC service, JWT middleware)
