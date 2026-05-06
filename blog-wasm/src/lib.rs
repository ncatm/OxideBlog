use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use web_sys::window;

const TOKEN_KEY: &str = "blog_token";
const USER_KEY: &str = "blog_user";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBrief {
    pub id: i64,
    pub username: String,
}

#[wasm_bindgen]
pub struct BlogApp {
    base_url: String,
    token: Option<String>,
    user: Option<UserBrief>,
}

#[wasm_bindgen]
impl BlogApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut app = Self {
            base_url: "http://127.0.0.1:8080".to_string(),
            token: None,
            user: None,
        };
        app.reload_from_storage();
        app
    }

    #[wasm_bindgen]
    pub fn set_base_url(&mut self, url: String) {
        self.base_url = url.trim_end_matches('/').to_string();
    }

    #[wasm_bindgen]
    pub fn base_url(&self) -> String {
        self.base_url.clone()
    }

    #[wasm_bindgen]
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    /// Строковый id, чтобы избежать проблем с `BigInt`/`number` в JS при сравнении с JSON.
    #[wasm_bindgen]
    pub fn user_id(&self) -> Option<String> {
        self.user.as_ref().map(|u| u.id.to_string())
    }

    #[wasm_bindgen]
    pub fn username(&self) -> Option<String> {
        self.user.as_ref().map(|u| u.username.clone())
    }

    #[wasm_bindgen]
    pub fn logout(&mut self) {
        storage_remove(TOKEN_KEY);
        storage_remove(USER_KEY);
        self.token = None;
        self.user = None;
    }

    #[wasm_bindgen]
    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<JsValue, JsValue> {
        self.require_non_empty("username", &username)?;
        self.require_non_empty("email", &email)?;
        self.require_non_empty("password", &password)?;

        let url = format!("{}/api/auth/register", self.base_url);
        let body = json!({ "username": username, "email": email, "password": password });
        let resp = Request::post(&url).json(&body).map_err(js_err)?.send().await.map_err(js_err)?;
        if !resp.ok() {
            return Err(JsValue::from_str(&read_http_error(resp).await));
        }
        let value: serde_json::Value = resp.json().await.map_err(js_err)?;
        self.persist_auth(&value)?;
        to_value(&value).map_err(js_err)
    }

    #[wasm_bindgen]
    pub async fn login(&mut self, username: String, password: String) -> Result<JsValue, JsValue> {
        self.require_non_empty("username", &username)?;
        self.require_non_empty("password", &password)?;

        let url = format!("{}/api/auth/login", self.base_url);
        let body = json!({ "username": username, "password": password });
        let resp = Request::post(&url).json(&body).map_err(js_err)?.send().await.map_err(js_err)?;
        if !resp.ok() {
            return Err(JsValue::from_str(&read_http_error(resp).await));
        }
        let value: serde_json::Value = resp.json().await.map_err(js_err)?;
        self.persist_auth(&value)?;
        to_value(&value).map_err(js_err)
    }

    #[wasm_bindgen]
    pub async fn load_posts(&self) -> Result<JsValue, JsValue> {
        let url = format!("{}/api/posts?limit=50&offset=0", self.base_url);
        let resp = Request::get(&url).send().await.map_err(js_err)?;
        if !resp.ok() {
            return Err(JsValue::from_str(&read_http_error(resp).await));
        }
        let value: serde_json::Value = resp.json().await.map_err(js_err)?;
        // For JS interop stability, return plain JSON string and parse on frontend.
        Ok(JsValue::from_str(&value.to_string()))
    }

    #[wasm_bindgen]
    pub async fn create_post(&mut self, title: String, content: String) -> Result<JsValue, JsValue> {
        self.require_non_empty("title", &title)?;
        self.require_non_empty("content", &content)?;
        let token = self.token.clone().ok_or_else(|| JsValue::from_str("not authenticated"))?;

        let url = format!("{}/api/posts", self.base_url);
        let body = json!({ "title": title, "content": content });
        let resp = Request::post(&url)
            .header("Authorization", &format!("Bearer {token}"))
            .json(&body)
            .map_err(js_err)?
            .send()
            .await
            .map_err(js_err)?;
        if !resp.ok() {
            return Err(JsValue::from_str(&read_http_error(resp).await));
        }
        let value: serde_json::Value = resp.json().await.map_err(js_err)?;
        to_value(&value).map_err(js_err)
    }

    #[wasm_bindgen]
    pub async fn update_post(
        &mut self,
        id: String,
        title: String,
        content: Option<String>,
    ) -> Result<JsValue, JsValue> {
        self.require_non_empty("title", &title)?;
        let token = self.token.clone().ok_or_else(|| JsValue::from_str("not authenticated"))?;
        let id: i64 = id.trim().parse().map_err(|_| JsValue::from_str("invalid post id"))?;

        let url = format!("{}/api/posts/{id}", self.base_url);
        let body = json!({ "title": title, "content": content });
        let resp = Request::put(&url)
            .header("Authorization", &format!("Bearer {token}"))
            .json(&body)
            .map_err(js_err)?
            .send()
            .await
            .map_err(js_err)?;
        if !resp.ok() {
            return Err(JsValue::from_str(&read_http_error(resp).await));
        }
        let value: serde_json::Value = resp.json().await.map_err(js_err)?;
        to_value(&value).map_err(js_err)
    }

    #[wasm_bindgen]
    pub async fn delete_post(&mut self, id: String) -> Result<(), JsValue> {
        let token = self.token.clone().ok_or_else(|| JsValue::from_str("not authenticated"))?;
        let id: i64 = id.trim().parse().map_err(|_| JsValue::from_str("invalid post id"))?;
        let url = format!("{}/api/posts/{id}", self.base_url);
        let resp = Request::delete(&url)
            .header("Authorization", &format!("Bearer {token}"))
            .send()
            .await
            .map_err(js_err)?;
        if !resp.ok() {
            return Err(JsValue::from_str(&read_http_error(resp).await));
        }
        Ok(())
    }
}

impl BlogApp {
    fn reload_from_storage(&mut self) {
        self.token = storage_get(TOKEN_KEY);
        if let Some(raw) = storage_get(USER_KEY) {
            if let Ok(user) = serde_json::from_str::<UserBrief>(&raw) {
                self.user = Some(user);
            }
        }
    }

    fn persist_auth(&mut self, value: &serde_json::Value) -> Result<(), JsValue> {
        if let Some(token) = value.get("token").and_then(|t| t.as_str()) {
            storage_set(TOKEN_KEY, token);
            self.token = Some(token.to_string());
        }
        if let Some(user_val) = value.get("user") {
            let brief: UserBrief = serde_json::from_value(user_val.clone()).map_err(js_err)?;
            storage_set(USER_KEY, &serde_json::to_string(&brief).map_err(js_err)?);
            self.user = Some(brief);
        }
        Ok(())
    }

    fn require_non_empty(&self, field: &str, value: &str) -> Result<(), JsValue> {
        if value.trim().is_empty() {
            return Err(JsValue::from_str(&format!("{field} is required")));
        }
        Ok(())
    }
}

fn js_err(err: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&err.to_string())
}

async fn read_http_error(resp: gloo_net::http::Response) -> String {
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    format!("HTTP {status}: {text}")
}

fn storage_get(key: &str) -> Option<String> {
    window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(key).ok().flatten())
}

fn storage_set(key: &str, value: &str) {
    if let Some(w) = window() {
        if let Ok(Some(s)) = w.local_storage() {
            let _ = s.set_item(key, value);
        }
    }
}

fn storage_remove(key: &str) {
    if let Some(w) = window() {
        if let Ok(Some(s)) = w.local_storage() {
            let _ = s.remove_item(key);
        }
    }
}
