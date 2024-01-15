
create table if not exists users
(
    id serial  primary key,
    phone_number text collate pg_catalog."default" not null,
    email text collate pg_catalog."default" not null,
    pwd_hash text collate pg_catalog."default" not null,
    state smallint not null,
    multi_sign_strategy text not null,
    verified boolean not null,
    invite_code text collate pg_catalog."default" not null,
    direct_invited_number integer not null,
    ancestors text[] not null,
    points integer not null,
    grade smallint not null,
    fans_num integer not null,
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
comment on column users.state is '状态，0=正常，1=已冻结，2=继承后产生的子账户';
comment on column users.multi_sign_strategy is '转账时多签要求';
comment on column users.verified is 'true=已实名';
comment on column users.invite_code is '邀请码';
comment on column users.direct_invited_number is '已（直推）邀请人数';
comment on column users.ancestors is '所有的上级用户 id（从直接上级开始）';
comment on column users.points is '积分';
comment on column users.grade is '等级';
comment on column users.fans_num is '粉丝数';

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
     tx_id text  primary key,
     sender integer,
     receiver integer,
     coin_type text,
     amount text,
     status  text,
     raw_data  text,
     signatures text[],
     updated_at  timestamp with time zone default current_timestamp,
     created_at  timestamp with time zone default current_timestamp
);
create index coin_transaction_tx_id on coin_transaction (tx_id);
create index coin_transaction_user on coin_transaction (sender,receiver);

-- tokens table
create table wallet
(
    user_id integer primary key,
    account_id text,
    sub_pubkeys text[],
    sign_strategies text[],
    participate_device_ids text[],
    updated_at  timestamp with time zone default current_timestamp,
    created_at  timestamp with time zone default current_timestamp
);