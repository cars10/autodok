use diesel::prelude::*;
use uuid::Uuid;

#[derive(serde::Deserialize, Insertable, Debug)]
#[diesel(table_name = crate::schema::projects)]
pub struct NewProject {
    name: String,
}

#[derive(serde::Serialize, Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::projects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Project {
    pub id: Uuid,
    pub name: String,
}
