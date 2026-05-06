# OxideBlog

Монорепозиторий (Cargo workspace) с четырьмя крейтами:

| Крейт | Назначение |
|-------|------------|
| `blog-server` | HTTP (actix-web, порт **8080**) + gRPC (tonic, **50051**), PostgreSQL (sqlx), JWT, миграции |
| `blog-client` | Общая библиотека: HTTP (reqwest) и gRPC (tonic) |
| `blog-cli` | CLI для проверки API (HTTP по умолчанию, `--grpc`) |
| `blog-wasm` | WASM-модуль + `index.html` для браузера (только HTTP) |

Генерация protobuf использует vendored `protoc` (`protoc-bin-vendored`), отдельная установка `protoc` не обязательна.

## Требования

- Rust (stable), PostgreSQL.
- Для WASM: `rustup target add wasm32-unknown-unknown`, [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/).

## Настройка сервера

1. Создайте базу данных PostgreSQL.
2. Файл `blog-server/.env` (или переменные окружения):

```env
DATABASE_URL=postgres://postgres:postgres@localhost/blog_db
JWT_SECRET=минимум_32_символа_для_подписи_jwt
```

3. Сборка workspace:

```bash
cargo build --workspace
```

4. Запуск:

```bash
cargo run -p blog-server
```

## CLI

Токен после `register` / `login` сохраняется в `.blog_token` (файл в текущей директории).

```bash
# HTTP (по умолчанию http://127.0.0.1:8080)
cargo run -p blog-cli -- register --username ivan --email ivan@example.com --password secret123
cargo run -p blog-cli -- login --username ivan --password secret123
cargo run -p blog-cli -- create --title "Привет" --content "Текст"
cargo run -p blog-cli -- list --limit 10 --offset 0
cargo run -p blog-cli -- get --id 1
cargo run -p blog-cli -- update --id 1 --title "Новый заголовок" --content "Опционально"
cargo run -p blog-cli -- delete --id 1

# gRPC (по умолчанию http://127.0.0.1:50051)
cargo run -p blog-cli -- --grpc register --username ivan --email ivan@example.com --password secret123
```

## WASM-фронтенд

```bash
rustup target add wasm32-unknown-unknown
wasm-pack build blog-wasm --target web --out-dir pkg
# из корня репозитория, где лежит index.html:
python3 -m http.server 8000
```

Откройте `http://localhost:8000`, при необходимости смените base URL сервера в форме на странице.

## Примеры curl

```bash
curl -s -X POST http://127.0.0.1:8080/api/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"u1","email":"u1@example.com","password":"secret123"}'

curl -s 'http://127.0.0.1:8080/api/posts?limit=5&offset=0'
```

## Архитектура сервера (clean architecture)

`blog-server/src`: `domain`, `application`, `data`, `infrastructure`, `presentation` (HTTP, gRPC, JWT middleware).
