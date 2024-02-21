
create table if not exists users
(
    id serial  primary key,
    phone_number text collate pg_catalog."default" not null,
    email text collate pg_catalog."default" not null,
    pwd_hash text collate pg_catalog."default" not null,
    predecessor int,
    status smallint not null,
    verified boolean not null,
    invite_code text collate pg_catalog."default" not null,
    account_ids text[] not null,
    predecessor_replace_laste_time text,
    main_account_id text not null,
    sub_account_ids text[] not null,
    constraint users_invite_code_key unique (invite_code),
    updated_at  timestamp with time zone default current_timestamp,
    created_at  timestamp with time zone default current_timestamp
);

--tablespace pg_default;
--comment on table users is '用户';
comment on column users.id is '用户 id';
comment on column users.phone_number is '手机号';
comment on column users.email is '邮箱';
comment on column users.pwd_hash is '密码 hash';
comment on column users.created_at is '创建时间';
comment on column users.status is '状态，0=正常，1=已冻结';
comment on column users.predecessor is '邀请人';
comment on column users.verified is 'true=已实名';
comment on column users.invite_code is '邀请码';

-- index: ix_users_email
create index if not exists ix_users_email
    on users using btree
    (email collate pg_catalog."default" asc nulls last)
    tablespace pg_default;
-- index: ix_users_phone
create index if not exists ix_users_phone
    on users using btree
    (phone_number collate pg_catalog."default" asc nulls last)
    tablespace pg_default;


create table coin_transaction(
     tx_index serial  primary key,
     tx_id text,
     sender text,
     receiver text,
     coin_type text,
     amount text,
     expire_at text,
     memo  text,
     status  text,
     coin_tx_raw  text,
     chain_tx_raw  text,
     signatures text[],
     updated_at  timestamp with time zone default current_timestamp,
     created_at  timestamp with time zone default current_timestamp
);
create index coin_transaction_tx_id on coin_transaction (tx_id);
create index coin_transaction_user on coin_transaction (sender,receiver);

-- tokens table
create table wallet
(
    account_id text primary key,
    user_id int,
    master_pubkey text,
    servant_pubkeys text[],
    sign_strategy text, --json
    updated_at  timestamp with time zone default current_timestamp,
    created_at  timestamp with time zone default current_timestamp
);


create table device
(
    device_id text primary key,
    user_id int,
    type   text, --master,servant,readonly
    status text, --active,abandoned
    updated_at  timestamp with time zone default current_timestamp,
    created_at  timestamp with time zone default current_timestamp
);

create table secret_store
(
    account_id text primary key,
    user_id int,
    master_encrypted_prikey text,
    servant_encrypted_prikeys text[],
    updated_at  timestamp with time zone default current_timestamp,
    created_at  timestamp with time zone default current_timestamp
);

--储蓄账户的主pubkey和从pubkey，子账户的key不存
create table key_info
(
    pubkey text primary key,
    device_info text,
    updated_at  timestamp with time zone default current_timestamp,
    created_at  timestamp with time zone default current_timestamp
);