BEGIN;
DO $$
DECLARE
  v_project uuid := gen_random_uuid();
  v_owner uuid := gen_random_uuid();
  v_outsider uuid := gen_random_uuid();
  v_floor uuid := gen_random_uuid();
  v_object uuid := gen_random_uuid();
  v_count integer;
  v_revision bigint;
  v_denied boolean := false;
BEGIN
  INSERT INTO bim_projects(id,name,owner_account_id) VALUES(v_project,'feed test',v_owner);
  INSERT INTO bim_project_members(project_id,account_id,role,can_edit) VALUES(v_project,v_owner,'OWNER',true);
  INSERT INTO bim_floors(id,project_id,code,name,sort_order,elevation_m,height_m) VALUES(v_floor,v_project,'F1','F1',1,0,3);
  v_revision := bim_create_design_object(v_project,v_object,v_owner,v_floor,'ARCH','HF1','LINE','{}'::jsonb);
  IF v_revision <> 1 THEN RAISE EXCEPTION 'create revision incorrect'; END IF;
  PERFORM bim_update_design_object(v_project,v_object,v_owner,1,'{"x":2}'::jsonb);
  PERFORM bim_delete_design_object(v_project,v_object,v_owner,2);
  SELECT count(*) INTO v_count FROM bim_pull_project_changes(v_project,v_owner,0,500);
  IF v_count <> 3 THEN RAISE EXCEPTION 'expected 3 change events, got %',v_count; END IF;
  SELECT count(*) INTO v_count FROM bim_pull_project_changes(v_project,v_owner,1,500);
  IF v_count <> 2 THEN RAISE EXCEPTION 'resume cursor lost events'; END IF;
  SELECT count(*) INTO v_count FROM bim_pull_project_changes(v_project,v_owner,0,2);
  IF v_count <> 2 THEN RAISE EXCEPTION 'page size not enforced'; END IF;
  IF (SELECT string_agg(f.operation,',' ORDER BY f.project_revision) FROM bim_project_change_feed f WHERE f.project_id=v_project) <> 'CREATE,UPDATE,DELETE' THEN
    RAISE EXCEPTION 'feed order incorrect';
  END IF;
  BEGIN
    PERFORM bim_pull_project_changes(v_project,v_outsider,0,500);
  EXCEPTION WHEN insufficient_privilege THEN v_denied := true;
  END;
  IF NOT v_denied THEN RAISE EXCEPTION 'outsider accessed change feed'; END IF;
  RAISE NOTICE 'CAD360 incremental change feed passed';
END $$;
ROLLBACK;
