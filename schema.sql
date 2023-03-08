CREATE TABLE IF NOT EXISTS user
(
    id       INTEGER PRIMARY KEY NOT NULL,
    username TEXT UNIQUE         NOT NULL,
    password TEXT                NOT NULL
);

CREATE TABLE IF NOT EXISTS account
(
    id   INTEGER PRIMARY KEY NOT NULL,
    name TEXT                NOT NULL,
    user INTEGER,

    CONSTRAINT fk_user
        FOREIGN KEY (user) REFERENCES User (id)
);

CREATE TABLE IF NOT EXISTS expense
(
    id          INTEGER PRIMARY KEY NOT NULL,
    amount      REAL                NOT NULL,
    title       TEXT                NOT NULL,
    description TEXT,
    time        INTEGER DEFAULT (unixepoch()) NOT NULL,
    account     INTEGER NOT NULL ,

    CONSTRAINT fk_account
        FOREIGN KEY (account) REFERENCES Account (id)
);