alter table "group" rename to user_group;
alter table user_group rename column created to created_date;

alter table group_member rename column group_id to user_group;
alter table group_member rename column joined to created_date;
alter table group_member add created_by integer null references user_account(id);

create table if not exists good (
    id           serial       primary key,
    user_group   integer          null references user_group(id) on delete cascade,
    name         varchar(100) not null,
    description  text             null,
    created_by   integer      not null references user_account(id) on delete set null,
    created_date timestamp    not null default current_timestamp
);