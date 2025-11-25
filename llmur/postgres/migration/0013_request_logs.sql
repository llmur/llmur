CREATE TABLE request_logs (
  id                UUID     NOT NULL,
  attempt_number    SMALLINT NOT NULL,

  virtual_key_id    UUID NOT NULL,
  project_id        UUID NOT NULL,
  deployment_id     UUID NOT NULL,
  connection_id     UUID NOT NULL,

  input_tokens      INTEGER NOT NULL CHECK (input_tokens >= 0),
  output_tokens     INTEGER NOT NULL CHECK (output_tokens >= 0),
  total_tokens      INTEGER GENERATED ALWAYS AS (input_tokens + output_tokens) STORED,

  cost              FLOAT NOT NULL CHECK (cost >= 0),

  http_status_code  SMALLINT NOT NULL,
  error             TEXT,

  request_ts        TIMESTAMP NOT NULL,
  response_ts       TIMESTAMP NOT NULL,

  method            TEXT,
  path              TEXT,
  provider          TEXT,
  deployment_name   TEXT,
  project_name      TEXT,
  virtual_key_alias TEXT,

  created_at        TIMESTAMP NOT NULL DEFAULT (timezone('utc', now())),

  PRIMARY KEY(id, attempt_number)
);

CREATE INDEX idx_request_logs_vk_ts   ON request_logs (virtual_key_id, request_ts) INCLUDE (cost, total_tokens);
CREATE INDEX idx_request_logs_prj_ts  ON request_logs (project_id,     request_ts) INCLUDE (cost, total_tokens);
CREATE INDEX idx_request_logs_dep_ts  ON request_logs (deployment_id,  request_ts) INCLUDE (cost, total_tokens);
CREATE INDEX idx_request_logs_conn_ts ON request_logs (connection_id,  request_ts) INCLUDE (cost, total_tokens);

CREATE INDEX idx_request_logs_ts_brin ON request_logs USING BRIN (request_ts);