
create table if not exists users
(
    id serial  primary key,
    phone_number text collate pg_catalog."default" not null,
    email text collate pg_catalog."default" not null,
    login_pwd_hash text collate pg_catalog."default" not null,
    anwser_indexes text collate pg_catalog."default" not null,
    is_frozen bool,--冻结的账户
    predecessor int,
    laste_predecessor_replace_time text,
    invite_code text collate pg_catalog."default" not null,
    kyc_is_verified bool,
    secruity_is_seted bool,
    create_subacc_time text[],
    main_account text not null,
    op_status text not null,
    reserved_field1 text not null,
    reserved_field2 text not null,
    reserved_field3 text not null,
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
     order_id  text primary key,
     tx_id text,
     sender text,
     receiver text,
     coin_type text,
     amount text,
     expire_at text,
     memo  text,
     stage  text,
     coin_tx_raw  text,
     chain_tx_raw  text,
     signatures text[],
     tx_type text not null, --Forced,ToSub,FromSub
     chain_status text not null,
     reserved_field2 text not null,
     reserved_field3 text not null,
     updated_at  timestamp with time zone default current_timestamp,
     created_at  timestamp with time zone default current_timestamp
);
create index coin_transaction_tx_id on coin_transaction (tx_id);
create index coin_transaction_user on coin_transaction (sender,receiver);

--密钥备份
create table secret_store
(
    pubkey text primary key,
    state text,--Sitting,Deprecated
    user_id int,
    encrypted_prikey_by_password text,
    encrypted_prikey_by_answer text,
    updated_at  timestamp with time zone default current_timestamp,
    created_at  timestamp with time zone default current_timestamp
);

--储蓄账户的主pubkey和从pubkey，子钱包的key不存
create table device_info
(
    id text,
    user_id int,
    state text,--Active,Inactive
    hold_pubkey text,
    brand text, --huawei,apple
    holder_confirm_saved bool,
    key_role text,
    updated_at  timestamp with time zone default current_timestamp,
    created_at  timestamp with time zone default current_timestamp,
     --一台设备登陆多个账号
    CONSTRAINT device_user PRIMARY KEY (id, user_id)
);


create table wallet_manage_record
(
    record_id text primary key,
    user_id text,
    operation_type text,
    operator_pubkey text,
    operator_device_id text,
    operator_device_brand text,
    tx_ids text[],
    status text,
    updated_at  timestamp with time zone default current_timestamp,
    created_at  timestamp with time zone default current_timestamp
);

create table ethereum_bridge_order
(
    id text,
    order_type text,
    chainless_acc text,
    eth_addr text,
    coin text,
    amount text,
    reserved_field1 text,
    reserved_field2 text,
    reserved_field3 text,
    updated_at  timestamp with time zone default current_timestamp,
    created_at  timestamp with time zone default current_timestamp,
    CONSTRAINT bridge_order_type_and_id PRIMARY KEY (id, order_type)
);
