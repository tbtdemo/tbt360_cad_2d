-- CAD360 server-side write gate, migration 007.
-- Apply after BIM_COLLABORATION_SCHEMA.sql and 002_bim_hierarchical_permissions.sql.
-- API must authenticate TBT.Auth token and bind account identity from validated claims,
-- never from client JSON. DB role used by API must not be exposed to desktop clients.

CREATE OR REPLACE FUNCTION bim_can_edit_object(p_project uuid, p_account uuid, p_floor uuid, p_discipline text, p_owner uuid)
RETURNS boolean LANGUAGE sql STABLE AS $$
  SELECT p_project IS NOT NULL AND p_account IS NOT NULL AND p_floor IS NOT NULL
    AND EXISTS (SELECT 1 FROM bim_projects p WHERE p.id=p_project)
    AND EXISTS (SELECT 1 FROM bim_floors f WHERE f.project_id=p_project AND f.id=p_floor)
    AND EXISTS (SELECT 1 FROM bim_project_members m WHERE m.project_id=p_project AND m.account_id=p_account)
    AND p_discipline IS NOT NULL AND length(btrim(p_discipline)) > 0
    AND (
      EXISTS (SELECT 1 FROM bim_project_members m WHERE m.project_id=p_project
              AND m.account_id=p_account AND m.can_edit)
      OR EXISTS (SELECT 1 FROM bim_projects p WHERE p.id=p_project AND p.owner_account_id=p_account)
      OR EXISTS (SELECT 1 FROM bim_access_grants g WHERE g.project_id=p_project
        AND g.viewer_account_id=p_account AND g.can_edit AND NOT g.deny
        AND (g.floor_id IS NULL OR g.floor_id=p_floor)
        AND (g.discipline IS NULL OR g.discipline=p_discipline)
        AND (g.source_owner_account_id IS NULL OR g.source_owner_account_id=p_owner)
        AND g.building_id IS NULL AND g.zone_id IS NULL AND g.workset_id IS NULL)
    )
    AND NOT EXISTS (SELECT 1 FROM bim_access_grants d WHERE d.project_id=p_project
      AND d.viewer_account_id=p_account AND d.deny
      AND (d.floor_id IS NULL OR d.floor_id=p_floor)
      AND (d.discipline IS NULL OR d.discipline=p_discipline)
      AND (d.source_owner_account_id IS NULL OR d.source_owner_account_id=p_owner)
      AND d.building_id IS NULL AND d.zone_id IS NULL AND d.workset_id IS NULL);
$$;

-- Atomic revision CAS update. Single SQL call is a transaction-scoped update.
-- Caller supplies authenticated account from validated JWT claims at API boundary.
-- A failed authorization or stale revision never modifies geometry.
CREATE OR REPLACE FUNCTION bim_update_design_object(
  p_project uuid, p_object uuid, p_authenticated_account uuid,
  p_expected_revision bigint, p_payload jsonb)
RETURNS bigint LANGUAGE plpgsql AS $$
DECLARE
  v_object bim_design_objects%ROWTYPE;
  v_new_revision bigint;
BEGIN
  IF p_project IS NULL OR p_object IS NULL OR p_authenticated_account IS NULL
     OR p_payload IS NULL OR jsonb_typeof(p_payload) <> 'object'
     OR p_expected_revision IS NULL OR p_expected_revision < 1 THEN
    RAISE EXCEPTION 'invalid geometry payload or revision' USING ERRCODE='22023';
  END IF;

  SELECT * INTO v_object FROM bim_design_objects
    WHERE id=p_object AND project_id=p_project AND NOT deleted FOR UPDATE;
  IF NOT FOUND THEN
    RAISE EXCEPTION 'design object not found' USING ERRCODE='P0002';
  END IF;
  IF NOT bim_can_edit_object(p_project,p_authenticated_account,v_object.floor_id,v_object.discipline,v_object.owner_account_id) THEN
    RAISE EXCEPTION 'write permission denied' USING ERRCODE='42501';
  END IF;
  IF v_object.revision <> p_expected_revision THEN
    RAISE EXCEPTION 'stale object revision' USING ERRCODE='40001';
  END IF;
  IF EXISTS (SELECT 1 FROM bim_object_leases l WHERE l.project_id=p_project
    AND l.design_object_id=p_object AND l.expires_at > now()
    AND l.account_id <> p_authenticated_account) THEN
    RAISE EXCEPTION 'object is locked by another editor' USING ERRCODE='55P03';
  END IF;

  UPDATE bim_design_objects SET geometry_payload=p_payload,revision=revision+1,updated_at=now()
    WHERE id=p_object AND project_id=p_project RETURNING revision INTO v_new_revision;
  UPDATE bim_projects SET revision=revision+1,updated_at=now() WHERE id=p_project;
  INSERT INTO bim_audit_log(project_id,account_id,action,design_object_id,source_handle,base_revision,result_revision)
    VALUES(p_project,p_authenticated_account,'UPDATE',p_object,v_object.source_handle,p_expected_revision,v_new_revision);
  RETURN v_new_revision;
END;
$$;

COMMENT ON FUNCTION bim_update_design_object(uuid,uuid,uuid,bigint,jsonb) IS
'Server-only write gate. The account argument MUST come from validated TBT.Auth token claims. Restrict EXECUTE privileges to trusted API DB role.';

-- Soft-delete with identical authorization, lease and stale-revision checks.
-- Preserve entity and audit history for recovery instead of physically deleting geometry.
CREATE OR REPLACE FUNCTION bim_delete_design_object(
  p_project uuid, p_object uuid, p_authenticated_account uuid, p_expected_revision bigint)
RETURNS bigint LANGUAGE plpgsql AS $$
DECLARE
  v_object bim_design_objects%ROWTYPE;
  v_new_revision bigint;
BEGIN
  IF p_project IS NULL OR p_object IS NULL OR p_authenticated_account IS NULL
     OR p_expected_revision IS NULL OR p_expected_revision < 1 THEN
    RAISE EXCEPTION 'invalid delete request' USING ERRCODE='22023';
  END IF;
  SELECT * INTO v_object FROM bim_design_objects
    WHERE id=p_object AND project_id=p_project AND NOT deleted FOR UPDATE;
  IF NOT FOUND THEN RAISE EXCEPTION 'design object not found' USING ERRCODE='P0002'; END IF;
  IF NOT bim_can_edit_object(p_project,p_authenticated_account,v_object.floor_id,v_object.discipline,v_object.owner_account_id) THEN
    RAISE EXCEPTION 'write permission denied' USING ERRCODE='42501';
  END IF;
  IF v_object.revision <> p_expected_revision THEN
    RAISE EXCEPTION 'stale object revision' USING ERRCODE='40001';
  END IF;
  IF EXISTS (SELECT 1 FROM bim_object_leases l WHERE l.project_id=p_project
    AND l.design_object_id=p_object AND l.expires_at > now()
    AND l.account_id <> p_authenticated_account) THEN
    RAISE EXCEPTION 'object is locked by another editor' USING ERRCODE='55P03';
  END IF;
  UPDATE bim_design_objects SET deleted=true,revision=revision+1,updated_at=now()
    WHERE id=p_object AND project_id=p_project RETURNING revision INTO v_new_revision;
  UPDATE bim_projects SET revision=revision+1,updated_at=now() WHERE id=p_project;
  INSERT INTO bim_audit_log(project_id,account_id,action,design_object_id,source_handle,base_revision,result_revision)
    VALUES(p_project,p_authenticated_account,'DELETE',p_object,v_object.source_handle,p_expected_revision,v_new_revision);
  RETURN v_new_revision;
END;
$$;
COMMENT ON FUNCTION bim_delete_design_object(uuid,uuid,uuid,bigint) IS
'Server-only soft delete. Authenticated account must come from validated TBT.Auth token claims.';

-- Creation is serialized on the parent project to maintain a monotonic project revision.
CREATE OR REPLACE FUNCTION bim_create_design_object(
  p_project uuid, p_object uuid, p_authenticated_account uuid, p_floor uuid,
  p_discipline text, p_source_handle text, p_entity_type text, p_payload jsonb)
RETURNS bigint LANGUAGE plpgsql AS $$
DECLARE
  v_owner uuid;
  v_revision bigint;
BEGIN
  IF p_project IS NULL OR p_object IS NULL OR p_authenticated_account IS NULL OR p_floor IS NULL
     OR p_discipline IS NULL OR p_source_handle IS NULL OR btrim(p_source_handle)=''
     OR p_entity_type IS NULL OR btrim(p_entity_type)=''
     OR p_payload IS NULL OR jsonb_typeof(p_payload) <> 'object' THEN
    RAISE EXCEPTION 'invalid create request' USING ERRCODE='22023';
  END IF;
  PERFORM 1 FROM bim_projects WHERE id=p_project FOR UPDATE;
  IF NOT FOUND THEN RAISE EXCEPTION 'project not found' USING ERRCODE='P0002'; END IF;
  IF NOT bim_can_edit_object(p_project,p_authenticated_account,p_floor,p_discipline,p_authenticated_account) THEN
    RAISE EXCEPTION 'write permission denied' USING ERRCODE='42501';
  END IF;
  INSERT INTO bim_design_objects(id,project_id,source_handle,floor_id,discipline,owner_account_id,entity_type,geometry_payload)
    VALUES(p_object,p_project,p_source_handle,p_floor,p_discipline,p_authenticated_account,p_entity_type,p_payload);
  UPDATE bim_projects SET revision=revision+1,updated_at=now() WHERE id=p_project RETURNING revision INTO v_revision;
  INSERT INTO bim_audit_log(project_id,account_id,action,design_object_id,source_handle,base_revision,result_revision)
    VALUES(p_project,p_authenticated_account,'CREATE',p_object,p_source_handle,0,1);
  RETURN v_revision;
END;
$$;
COMMENT ON FUNCTION bim_create_design_object(uuid,uuid,uuid,uuid,text,text,text,jsonb) IS
'API-only creation. Authenticated account is supplied by verified TBT.Auth principal, never request body.';
