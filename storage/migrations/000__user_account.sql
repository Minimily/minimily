create table if not exists user_account (
    id                serial       primary key,
    first_name        varchar(50)  not null,
    last_name         varchar(50)  not null,
    gender            varchar(10)      null,
    birth_date        date             null,
    email             varchar(100)     null,
    password          varchar(100)     null,
    created           timestamp    not null default current_timestamp,
    modified          timestamp    not null default current_timestamp
);