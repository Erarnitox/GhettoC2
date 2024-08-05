create table zombies (
    id uuid primary key default gen_random_uuid(),
    internal_ip inet,
    external_ip inet,
    hostname varchar,
    username varchar,
    operating_system varchar,
    last_update timestamp default now(),
    status int default 0
)
