use sea_orm::{entity::prelude::*, DeleteMany};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Deserialize, Serialize)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub text: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// impl Entity {
//     pub fn find_by_id(id: i32) -> Select<Entity> {
//         Self::find().filter(Column::Id.eq(id))
//     }

//     pub fn find_by_title(title: &str) -> Select<Entity> {
//         Self::find().filter(Column::Title.eq(title))
//     }

//     pub fn delete_by_id(id: i32) -> DeleteMany<Entity> {
//         Self::delete_many().filter(Column::Id.eq(id))
//     }
// }
