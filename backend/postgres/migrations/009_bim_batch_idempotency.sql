-- Atomic idempotency receipts for project-scoped HTTP change batches.
CREATE TABLE IF NOT EXISTS bim_change_batch_receipts (
 project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
 account_id uuid NOT NULL,
 client_batch_id uuid NOT NULL,
 request_sha256 text NOT NULL,
 response_json jsonb NOT NULL,
 created_at timestamptz NOT NULL DEFAULT now(),
 PRIMARY KEY(project_id,account_id,client_batch_id)
);
CREATE INDEX IF NOT EXISTS ix_bim_batch_receipts_created ON bim_change_batch_receipts(created_at);
