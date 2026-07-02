use crate::application::blog_service::BlogService;
use crate::domain::error::PostError;
use super::in_memory::InMemoryPostRepository;
use std::sync::Arc;

fn make_service() -> BlogService<InMemoryPostRepository> {
    BlogService::new(Arc::new(InMemoryPostRepository::new()))
}

#[tokio::test]
async fn create_post_assigns_id() {
    let svc = make_service();
    let post = svc.create_post("Title", "Content", 1).await.unwrap();
    assert_eq!(post.title, "Title");
    assert_eq!(post.content, "Content");
    assert_eq!(post.author_id, 1);
    assert!(post.id > 0);
}

#[tokio::test]
async fn get_post_by_id_returns_post() {
    let svc = make_service();
    let created = svc.create_post("Hello", "World", 1).await.unwrap();
    let found = svc.get_post_by_id(created.id).await.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.title, "Hello");
}

#[tokio::test]
async fn get_post_by_id_unknown_returns_not_found() {
    let svc = make_service();
    let err = svc.get_post_by_id(999).await.unwrap_err();
    assert!(matches!(err, PostError::NotFound));
}

#[tokio::test]
async fn list_posts_returns_all() {
    let svc = make_service();
    svc.create_post("A", "a", 1).await.unwrap();
    svc.create_post("B", "b", 1).await.unwrap();
    svc.create_post("C", "c", 1).await.unwrap();
    let posts = svc.get_list_posts(0, 10).await.unwrap();
    assert_eq!(posts.len(), 3);
}

#[tokio::test]
async fn list_posts_pagination_no_overlap() {
    let svc = make_service();
    for i in 0..5 {
        svc.create_post(&format!("Post {}", i), "content", 1).await.unwrap();
    }
    let page1 = svc.get_list_posts(0, 2).await.unwrap();
    let page2 = svc.get_list_posts(2, 2).await.unwrap();
    assert_eq!(page1.len(), 2);
    assert_eq!(page2.len(), 2);
    assert!(page1.iter().all(|p| page2.iter().all(|q| q.id != p.id)));
}

#[tokio::test]
async fn get_posts_count_reflects_creates() {
    let svc = make_service();
    assert_eq!(svc.get_posts_count().await.unwrap(), 0);
    svc.create_post("T", "C", 1).await.unwrap();
    svc.create_post("T2", "C2", 1).await.unwrap();
    assert_eq!(svc.get_posts_count().await.unwrap(), 2);
}

#[tokio::test]
async fn update_post_changes_title_and_content() {
    let svc = make_service();
    let post = svc.create_post("Old Title", "Old Content", 1).await.unwrap();
    let updated = svc.update_post(post.id, 1, "New Title", "New Content").await.unwrap();
    assert_eq!(updated.id, post.id);
    assert_eq!(updated.title, "New Title");
    assert_eq!(updated.content, "New Content");
}

#[tokio::test]
async fn update_post_wrong_user_returns_not_found() {
    let svc = make_service();
    let post = svc.create_post("Title", "Content", 1).await.unwrap();
    let err = svc.update_post(post.id, 2, "X", "Y").await.unwrap_err();
    assert!(matches!(err, PostError::NotFound));
}

#[tokio::test]
async fn delete_post_removes_it() {
    let svc = make_service();
    let post = svc.create_post("Delete Me", "...", 1).await.unwrap();
    svc.delete_post(post.id, 1).await.unwrap();
    let err = svc.get_post_by_id(post.id).await.unwrap_err();
    assert!(matches!(err, PostError::NotFound));
}

#[tokio::test]
async fn delete_post_wrong_user_returns_not_found() {
    let svc = make_service();
    let post = svc.create_post("Keep Me", "...", 1).await.unwrap();
    let err = svc.delete_post(post.id, 2).await.unwrap_err();
    assert!(matches!(err, PostError::NotFound));
    // пост должен остаться
    assert!(svc.get_post_by_id(post.id).await.is_ok());
}

#[tokio::test]
async fn list_posts_ordered_by_id_desc() {
    let svc = make_service();
    let p1 = svc.create_post("First", "c", 1).await.unwrap();
    let p2 = svc.create_post("Second", "c", 1).await.unwrap();
    let p3 = svc.create_post("Third", "c", 1).await.unwrap();
    let posts = svc.get_list_posts(0, 10).await.unwrap();
    // новые посты идут первыми
    assert_eq!(posts[0].id, p3.id);
    assert_eq!(posts[1].id, p2.id);
    assert_eq!(posts[2].id, p1.id);
}
