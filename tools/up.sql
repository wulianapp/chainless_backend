CREATE TABLE IF NOT EXISTS users
(
    id SERIAL  primary key,
    phone_number text COLLATE pg_catalog."default" NOT NULL,
    email text COLLATE pg_catalog."default" NOT NULL,
    pwd_hash text COLLATE pg_catalog."default" NOT NULL,
                          state smallint NOT NULL,
    multi_sign_strategy text NOT NULL,
                          verified boolean NOT NULL,
                          invite_code text COLLATE pg_catalog."default" NOT NULL,
    direct_invited_number integer NOT NULL,
                          ancestors text[] NOT NULL,
                          points integer NOT NULL,
                          grade smallint NOT NULL,
    fans_num integer NOT NULL,
                          --CONSTRAINT users_pkey PRIMARY KEY (id),
    CONSTRAINT users_invite_code_key UNIQUE (invite_code),
    updated_at  TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at  TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
--TABLESPACE pg_default;
--COMMENT ON TABLE users IS '用户';
COMMENT ON COLUMN users.id IS '用户 ID';
COMMENT ON COLUMN users.phone_number IS '手机号';
COMMENT ON COLUMN users.email IS '邮箱';
COMMENT ON COLUMN users.pwd_hash IS '密码 hash';
COMMENT ON COLUMN users.created_at IS '创建时间';
COMMENT ON COLUMN users.state IS '状态，0=正常，1=已冻结，2=继承后产生的子账户';
COMMENT ON COLUMN users.multi_sign_strategy IS '转账时多签要求';
COMMENT ON COLUMN users.verified IS 'true=已实名';
COMMENT ON COLUMN users.invite_code IS '邀请码';
COMMENT ON COLUMN users.direct_invited_number IS '已（直推）邀请人数';
COMMENT ON COLUMN users.ancestors IS '所有的上级用户 ID（从直接上级开始）';
COMMENT ON COLUMN users.points IS '积分';
COMMENT ON COLUMN users.grade IS '等级';
COMMENT ON COLUMN users.fans_num IS '粉丝数';

-- Index: ix_users_email
CREATE INDEX IF NOT EXISTS ix_users_email
    ON users USING btree
    (email COLLATE pg_catalog."default" ASC NULLS LAST)
    TABLESPACE pg_default;
-- Index: ix_users_phone
CREATE INDEX IF NOT EXISTS ix_users_phone
    ON users USING btree
    (phone_number COLLATE pg_catalog."default" ASC NULLS LAST)
    TABLESPACE pg_default;



-- tokens table
CREATE TABLE IF NOT EXISTS accounts
(
    id SERIAL  primary key,
    uid bigint NOT NULL,
    token_id integer NOT NULL,
    unlocked numeric(40,0) NOT NULL,
    locked numeric(40,0) NOT NULL,
    CONSTRAINT accounts_uid_token_id_key UNIQUE (uid, token_id),
    updated_at  TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at  TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE accounts IS '资金账户';
COMMENT ON COLUMN accounts.uid IS '用户 ID';
COMMENT ON COLUMN accounts.token_id IS '币种 ID';
COMMENT ON COLUMN accounts.unlocked IS '已解锁余额';
COMMENT ON COLUMN accounts.locked IS '已锁定余额';