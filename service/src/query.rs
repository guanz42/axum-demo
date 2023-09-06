use ::entity::{post, post::Entity as Post};
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn find_post_by_id(db: &DbConn, id: i32) -> Result<Option<post::Model>, DbErr> {
        Post::find_by_id(id).one(db).await
    }

    pub async fn find_posts_in_page(
        db: &DbConn,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<post::Model>, u64), DbErr> {
        let paginator = Post::find()
            .order_by_asc(post::Column::Id)
            .paginate(db, page_size);
        let num_pages = paginator.num_pages().await?;

        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
}
