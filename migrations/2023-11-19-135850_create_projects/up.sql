-- Your SQL goes here
CREATE TABLE images (
    id uuid DEFAULT gen_random_uuid (),
    image VARCHAR NOT NULL,
    tag VARCHAR NOT NULL,
    PRIMARY KEY (id)
);