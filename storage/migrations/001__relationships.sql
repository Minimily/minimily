create unique index if not exists user_account_email_uidx on user_account (email);

create table if not exists profile (
    id           serial       primary key,
    name         varchar(100) not null,
    type         varchar(20)  not null,
    user_account integer          null references user_account(id),
    created_by   integer      not null references user_account(id)
);

create table if not exists relationship (
    id           serial      primary key,
    profile_from integer     not null,
    profile_to   integer     not null,
    type         varchar(20) not null
);

create unique index if not exists relationship_uidx on relationship(profile_from, profile_to, type);