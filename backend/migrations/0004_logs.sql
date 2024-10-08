create table logs (
    id serial primary key,
    uid uuid not null,
    key varchar not null,
    value varchar not null,
    time timestamp with time zone default CURRENT_TIMESTAMP
);

alter table logs
add constraint commands_uid_fkey
foreign key (uid) references zombies (id)
on delete cascade
