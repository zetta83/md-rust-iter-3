use blog_dto::{AuthResponse, ListPostsResponse, PostResponse};

const API: &str = "http://localhost:8080/api";

async fn check(res: reqwest::Response) -> Result<reqwest::Response, String> {
    if res.status().is_success() {
        return Ok(res);
    }
    #[derive(serde::Deserialize)]
    struct Err { error: String }
    let text = res.text().await.unwrap_or_default();
    let msg = serde_json::from_str::<Err>(&text).map(|e| e.error).unwrap_or(text);
    Err(msg)
}

pub async fn login(username: &str, password: &str) -> Result<AuthResponse, String> {
    let res = reqwest::Client::new()
        .post(format!("{API}/auth/login"))
        .json(&serde_json::json!({ "username": username, "password": password }))
        .send().await.map_err(|e| e.to_string())?;
    check(res).await?.json::<AuthResponse>().await.map_err(|e| e.to_string())
}

pub async fn register(username: &str, email: &str, password: &str) -> Result<AuthResponse, String> {
    let res = reqwest::Client::new()
        .post(format!("{API}/auth/register"))
        .json(&serde_json::json!({ "username": username, "email": email, "password": password }))
        .send().await.map_err(|e| e.to_string())?;
    check(res).await?.json::<AuthResponse>().await.map_err(|e| e.to_string())
}

pub async fn load_posts(limit: u32, offset: u32) -> Result<ListPostsResponse, String> {
    let res = reqwest::Client::new()
        .get(format!("{API}/posts?page={offset}&limit={limit}"))
        .send().await.map_err(|e| e.to_string())?;
    check(res).await?.json::<ListPostsResponse>().await.map_err(|e| e.to_string())
}

pub async fn create_post(token: &str, title: &str, content: &str) -> Result<PostResponse, String> {
    let res = reqwest::Client::new()
        .post(format!("{API}/posts"))
        .bearer_auth(token)
        .json(&serde_json::json!({ "title": title, "content": content }))
        .send().await.map_err(|e| e.to_string())?;
    check(res).await?.json::<PostResponse>().await.map_err(|e| e.to_string())
}

pub async fn update_post(token: &str, id: i64, title: &str, content: &str) -> Result<PostResponse, String> {
    let res = reqwest::Client::new()
        .put(format!("{API}/posts/{id}"))
        .bearer_auth(token)
        .json(&serde_json::json!({ "title": title, "content": content }))
        .send().await.map_err(|e| e.to_string())?;
    check(res).await?.json::<PostResponse>().await.map_err(|e| e.to_string())
}

pub async fn delete_post(token: &str, id: i64) -> Result<(), String> {
    let res = reqwest::Client::new()
        .delete(format!("{API}/posts/{id}"))
        .bearer_auth(token)
        .send().await.map_err(|e| e.to_string())?;
    check(res).await?;
    Ok(())
}
