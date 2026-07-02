use blog_dto::User;
use gloo_storage::{LocalStorage, Storage};

pub fn load_token() -> Option<String> {
    LocalStorage::get::<String>("blog_token").ok()
}

pub fn save_token(token: &str) {
    let _ = LocalStorage::set("blog_token", token);
}

pub fn clear_token() {
    LocalStorage::delete("blog_token");
}

pub fn load_user() -> Option<User> {
    LocalStorage::get::<User>("blog_user").ok()
}

pub fn save_user(user: &User) {
    let _ = LocalStorage::set("blog_user", user);
}

pub fn clear_user() {
    LocalStorage::delete("blog_user");
}
