
create table if not exists users
(
    --用户id,目前是直接自增
    id bigserial  primary key, 
    phone_number text unique,
    email text unique,
    --登陆密码
    login_pwd_hash text not null,
    -- 安全问答序列号
    anwser_indexes text not null,
    -- 是否冻结，暂时没有对应的详细需求
    is_frozen bool,
    -- kyc的预留字段，不确定是否需要
    kyc_is_verified bool,
    -- 最近三次创建子账户的时间戳，对应每天只能创建三个子账户的需求
    create_subacc_time bigint[],
    -- 无链钱包id
    main_account text unique,
    -- 令牌版本
    token_version bigint not null,
    updated_at  timestamp with time zone default current_timestamp,
    created_at  timestamp with time zone default current_timestamp,
);


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
--ALTER TABLE users ALTER COLUMN invite_code SET DEFAULT currval('users_id_seq');


create table coin_transaction(
     -- 订单id
     order_id  text primary key,
     -- 链上tx_id
     tx_id text unique,
     --  发送方
     sender text,
     -- 接收方
     receiver text,
     -- 币种类型
     coin_type text,
     -- 交易数量(u256)
     amount text,
     -- 过期时间
     expire_at text,
     -- 备注
     memo  text,
     -- 交易进度
     stage  text,
     -- 转账的信息
     coin_tx_raw  text,
     -- 链交易组装原始数据
     chain_tx_raw  text,
     -- 从设备签名信息
     signatures text[],
     -- 交易类型，Forced,ToSub,FromSub
     tx_type text not null,
     -- 交易的链上状态
     chain_status text not null,
     -- 接收者的联系方式，对应要显示用户是通过哪个联系方式发起的需求
     receiver_contact text,
     updated_at  timestamp with time zone default current_timestamp,
     created_at  timestamp with time zone default current_timestamp
);
create index coin_transaction_tx_id on coin_transaction (tx_id);
create index coin_transaction_user on coin_transaction (sender,receiver);

--密钥备份
create table secret_store
(
    -- 主密钥的公钥
    pubkey text primary key,
    -- 密钥的使用状态，Sitting,Deprecated,
    state text,
    -- 用户id
    user_id bigserial,
    -- 被安全密码加密的密钥
    encrypted_prikey_by_password text,
    -- 被安全问答加密的密钥
    encrypted_prikey_by_answer text,
    updated_at  timestamp with time zone default current_timestamp,
    created_at  timestamp with time zone default current_timestamp
);

--储蓄账户的主pubkey和从pubkey，子钱包的key不存
create table device_info
(
    -- 设备id
    id text,
    -- 用户id
    user_id bigserial,
    -- 待废弃
    state text,
    -- 用户+设备对应持有的公钥
    hold_pubkey text,
    -- 设备品牌
    brand text, 
    -- 设备是否本地保存好，但是没有客户端没有用到
    holder_confirm_saved bool,
    -- 待废弃
    key_role text,
    updated_at  timestamp with time zone default current_timestamp,
    created_at  timestamp with time zone default current_timestamp,
     --一台设备登陆多个账号
    CONSTRAINT device_user PRIMARY KEY (id, user_id)
);


create table wallet_manage_record
(
    --记录id
    record_id text primary key,
    --用户id
    user_id bigserial,
    --操作类型
    operation_type text,
    --使用了哪个key操作的
    operator_pubkey text,
    -- 操作时的设备id
    operator_device_id text,
    -- 操作时的设备品牌
    operator_device_brand text,
    -- 操作产生的链上txid
    tx_ids text[],
    -- 操作结果
    status text,
    updated_at  timestamp with time zone default current_timestamp,
    created_at  timestamp with time zone default current_timestamp
);

create table ethereum_bridge_order
(
    -- 订单id
    id text,
    -- 订单类型
    order_type text,
    -- 跨链操作的无链侧account_id
    chainless_acc text,
    -- 跨链操作的eth侧address
    eth_addr text,
    -- 跨链币种
    coin text,
    -- 跨链数量（u256）
    amount text,
    -- 订单状态
    status text,
    -- 相应业务的eth高度
    height bigint,
    updated_at  timestamp with time zone default current_timestamp,
    created_at  timestamp with time zone default current_timestamp,
    --考虑到回滚的情况，唯一限制的时候加上状态
    CONSTRAINT bridge_order_type_status_id PRIMARY KEY (order_type,status,id)
);


create table airdrop
(
    -- 用户id
    user_id bigserial primary key,
    -- 用户钱包id
    account_id text unique,
    -- 用户邀请码
    invite_code text not null unique,
    -- 上级用户id
    predecessor_user_id bigserial not null,
    -- 上级钱包id
    predecessor_account_id text not null,
    -- btc地址
    btc_address text unique,
    -- btc登记
    btc_level smallint,
    -- 地址评级状态
    btc_grade_status text not null,
    updated_at  timestamp with time zone default current_timestamp,
    created_at  timestamp with time zone default current_timestamp
);

