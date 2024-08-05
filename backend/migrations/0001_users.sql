create table users (
    id serial primary key,
    username varchar not null,
    password varchar not null,
    clearance int not null
)
