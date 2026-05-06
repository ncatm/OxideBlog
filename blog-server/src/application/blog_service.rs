use std::sync::Arc;

use crate::{
    data::post_repository::PostStore,
    domain::{error::BlogError, post::Post},
};

pub struct BlogService {
    post_repo: Arc<dyn PostStore>,
}

impl BlogService {
    pub fn new(post_repo: Arc<dyn PostStore>) -> Self {
        Self { post_repo }
    }

    pub async fn create_post(&self, title: &str, content: &str, author_id: i64) -> Result<Post, BlogError> {
        self.post_repo.create(title, content, author_id).await
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, BlogError> {
        self.post_repo.get(id).await
    }

    pub async fn update_post(
        &self,
        id: i64,
        author_id: i64,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<Post, BlogError> {
        let post = self.post_repo.get(id).await?;
        if post.author_id != author_id {
            return Err(BlogError::Forbidden);
        }
        self.post_repo.update(id, title, content).await
    }

    pub async fn delete_post(&self, id: i64, author_id: i64) -> Result<(), BlogError> {
        let post = self.post_repo.get(id).await?;
        if post.author_id != author_id {
            return Err(BlogError::Forbidden);
        }
        self.post_repo.delete(id).await
    }

    pub async fn list_posts(&self, limit: i64, offset: i64) -> Result<(Vec<Post>, i64), BlogError> {
        self.post_repo.list(limit, offset).await
    }
}
