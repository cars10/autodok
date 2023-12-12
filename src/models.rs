use diesel::prelude::*;
use uuid::Uuid;

#[derive(serde::Deserialize, Insertable, Debug)]
#[diesel(table_name = crate::schema::images)]
pub struct NewImage {
    tag: String,
    image: String,
}

#[derive(serde::Serialize, Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::images)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Image {
    pub id: Uuid,
    pub tag: String,
    pub image: String,
}
