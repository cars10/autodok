-- Your SQL goes here
CREATE TABLE projects (
    id uuid DEFAULT gen_random_uuid (),
    name VARCHAR NOT NULL,
    PRIMARY KEY (id)
);