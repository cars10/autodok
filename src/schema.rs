// @generated automatically by Diesel CLI.

diesel::table! {
    images (id) {
        id -> Uuid,
        image -> Varchar,
        tag -> Varchar,
    }
}
