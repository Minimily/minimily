create table if not exists good_location (
    id           serial       primary key,
    name         varchar(100) not null,
    description  text             null,
    created_by   integer          null references user_account(id) on delete set null,
    created_date timestamp    not null default current_timestamp
);

alter table good add column if not exists location integer null references good_location(id) on delete set null;