-- CAD360 incremental project change feed, after migration 007.
-- The feed is committed atomically with the object change and audit.
CREATE TABLE IF NOT EXISTS bim_project_change_feed (
  project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
  project_revision bigint NOT NULL CHECK (project_revision > 0),
  design_object_id uuid NOT NULL REFERENCES bim_design_objects(id),
  source_handle text NOT NULL,
  operation text NOT NULL CHECK (operation IN ('CREATE','UPDATE','DELETE')),
  object_revision bigint NOT NULL,
  actor_account_id uuid NOT NULL,
  occurred_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY(project_id,project_revision)
);
CREATE INDEX IF NOT EXISTS ix_bim_change_feed_object ON bim_project_change_feed(project_id,design_object_id,project_revision DESC);

-- Audit insertion is the last operation of each SQL write gate.
-- A feed insert failure aborts the entire transaction, including the object edit.
CREATE OR REPLACE FUNCTION bim_append_project_change() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE v_revision bigint;
BEGIN
  IF NEW.action NOT IN ('CREATE','UPDATE','DELETE') OR NEW.design_object_id IS NULL THEN RETURN NEW; END IF;
  SELECT revision INTO v_revision FROM bim_projects WHERE id=NEW.project_id;
  IF v_revision IS NULL THEN RAISE EXCEPTION 'missing project for change feed' USING ERRCODE='23503'; END IF;
  INSERT INTO bim_project_change_feed(project_id,project_revision,design_object_id,source_handle,operation,object_revision,actor_account_id)
    VALUES(NEW.project_id,v_revision,NEW.design_object_id,NEW.source_handle,NEW.action,NEW.result_revision,NEW.account_id);
  RETURN NEW;
END;
$$;
DROP TRIGGER IF EXISTS trg_bim_append_project_change ON bim_audit_log;
CREATE TRIGGER trg_bim_append_project_change AFTER INSERT ON bim_audit_log
FOR EACH ROW EXECUTE FUNCTION bim_append_project_change();

-- Server API must derive p_account from verified auth claims. This function
-- enforces membership, avoids client-controlled actor identity, and bounds pages.
CREATE OR REPLACE FUNCTION bim_pull_project_changes(p_project uuid,p_account uuid,p_after_revision bigint,p_limit integer DEFAULT 500)
RETURNS TABLE(project_revision bigint,design_object_id uuid,source_handle text,operation text,object_revision bigint,actor_account_id uuid,occurred_at timestamptz)
LANGUAGE plpgsql STABLE AS $$
BEGIN
  IF p_project IS NULL OR p_account IS NULL OR p_after_revision IS NULL OR p_after_revision < 0
     OR p_limit IS NULL OR p_limit < 1 OR p_limit > 1000 THEN
    RAISE EXCEPTION 'invalid pull request' USING ERRCODE='22023';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM bim_project_members m WHERE m.project_id=p_project AND m.account_id=p_account) THEN
    RAISE EXCEPTION 'project access denied' USING ERRCODE='42501';
  END IF;
  IF p_after_revision > (SELECT p.revision FROM bim_projects p WHERE p.id=p_project) THEN
    RAISE EXCEPTION 'cursor ahead of server' USING ERRCODE='22023';
  END IF;
  RETURN QUERY SELECT f.project_revision,f.design_object_id,f.source_handle,f.operation,f.object_revision,f.actor_account_id,f.occurred_at
    FROM bim_project_change_feed f WHERE f.project_id=p_project AND f.project_revision > p_after_revision
    ORDER BY f.project_revision LIMIT p_limit;
END;
$$;
