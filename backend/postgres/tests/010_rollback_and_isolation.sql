-- Integration test: failed/rolled back writes must not leak audit, feed or revisions.
BEGIN;
DO $$
DECLARE
  v_project uuid := gen_random_uuid();
  v_owner uuid := gen_random_uuid();
  v_floor uuid := gen_random_uuid();
  v_object uuid := gen_random_uuid();
  v_after bigint;
BEGIN
  INSERT INTO bim_projects(id,name,owner_account_id) VALUES(v_project,'rollback check',v_owner);
  INSERT INTO bim_project_members(project_id,account_id,role,can_edit) VALUES(v_project,v_owner,'OWNER',true);
  INSERT INTO bim_floors(id,project_id,code,name,sort_order,elevation_m,height_m)
    VALUES(v_floor,v_project,'RB1','Rollback Level',1,0,3);

  -- A nested exception block is a real PL/pgSQL savepoint/subtransaction.
  BEGIN
    PERFORM bim_create_design_object(v_project,v_object,v_owner,v_floor,'ARCH','RB-HANDLE','LINE','{}'::jsonb);
    RAISE EXCEPTION 'simulate failure after write';
  EXCEPTION WHEN raise_exception THEN
    NULL;
  END;
  IF EXISTS (SELECT 1 FROM bim_design_objects d WHERE d.id=v_object)
     OR EXISTS (SELECT 1 FROM bim_audit_log a WHERE a.project_id=v_project)
     OR EXISTS (SELECT 1 FROM bim_project_change_feed f WHERE f.project_id=v_project)
     OR (SELECT p.revision FROM bim_projects p WHERE p.id=v_project) <> 0 THEN
    RAISE EXCEPTION 'transaction rollback leaked partial state';
  END IF;

  v_after := bim_create_design_object(v_project,v_object,v_owner,v_floor,'ARCH','RB-HANDLE','LINE','{}'::jsonb);
  IF v_after <> 1 THEN RAISE EXCEPTION 'revision after rollback should equal 1'; END IF;
  BEGIN
    PERFORM bim_update_design_object(v_project,v_object,v_owner,1,'{"value": 1}'::jsonb);
    RAISE EXCEPTION 'simulate failure after update';
  EXCEPTION WHEN raise_exception THEN NULL;
  END;
  IF (SELECT d.revision FROM bim_design_objects d WHERE d.id=v_object) <> 1
    OR (SELECT p.revision FROM bim_projects p WHERE p.id=v_project) <> 1
    OR (SELECT count(*) FROM bim_audit_log a WHERE a.project_id=v_project) <> 1
    OR (SELECT count(*) FROM bim_project_change_feed f WHERE f.project_id=v_project) <> 1 THEN
    RAISE EXCEPTION 'rolled-back update changed state';
  END IF;
  RAISE NOTICE 'CAD360 rollback invariants passed';
END $$;
ROLLBACK;
