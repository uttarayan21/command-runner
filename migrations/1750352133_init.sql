CREATE TABLE IF NOT EXISTS "users" (
    "id" text NOT NULL PRIMARY KEY,
    "username" text NOT NULL UNIQUE,
    "password" text NOT NULL,
    "created_at" datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" datetime NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "commands" (
    "id" text NOT NULL PRIMARY KEY,
    "name" text NOT NULL UNIQUE,
    "command" text NOT NULL,
    "args" json,
    "created_at" datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" datetime NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "history" (
    "id" text NOT NULL PRIMARY KEY,
    "command_id" text,
    "stdout" text,
    "stderr" text,
    "success" boolean,
    "exit_code" integer,
    "created_at" datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY ("command_id") REFERENCES "commands" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE trigger IF NOT EXISTS "commands_updated_at"
AFTER
UPDATE
    ON "commands" FOR each ROW
BEGIN
UPDATE
    "commands"
SET
    "updated_at" = CURRENT_TIMESTAMP
WHERE
    "id" = old."id";

END;

CREATE trigger IF NOT EXISTS "users_updated_at"
AFTER
UPDATE
    ON "users" FOR each ROW
BEGIN
UPDATE
    "users"
SET
    "updated_at" = CURRENT_TIMESTAMP
WHERE
    "id" = old."id";

END;
