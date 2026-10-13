create table if not exists "group" (
    id         serial       primary key,
    name       varchar(100) not null,
    type       varchar(20)  not null,
    created_by integer      not null references user_account(id),
    created    timestamp    not null default current_timestamp
);

create table if not exists group_member (
    id           serial    primary key,
    group_id     integer   not null references "group"(id) on delete cascade,
    user_account integer   not null references user_account(id),
    joined       timestamp not null default current_timestamp
);

create unique index if not exists group_member_uidx on group_member(group_id, user_account);

-- Move families stored as profiles into groups, preserving their ids, then drop the old tables.
do $$
begin
    if to_regclass('public.profile') is not null and to_regclass('public.relationship') is not null then
        insert into "group" (id, name, type, created_by)
            select p.id, p.name, 'family', p.created_by
            from profile p
            where p.type = 'family'
        on conflict (id) do nothing;

        perform setval(pg_get_serial_sequence('"group"', 'id'), coalesce((select max(id) from "group"), 0) + 1, false);

        insert into group_member (group_id, user_account)
            select r.profile_to, p.user_account
            from relationship r
                join profile p on p.id = r.profile_from
            where r.type = 'familymember'
                and p.user_account is not null
        on conflict (group_id, user_account) do nothing;
    end if;
end $$;