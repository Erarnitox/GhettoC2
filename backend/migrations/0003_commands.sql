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
on delete cascade
