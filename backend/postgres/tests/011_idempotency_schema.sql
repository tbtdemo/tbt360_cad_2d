BEGIN;
DO $$
DECLARE
  v_project uuid := gen_random_uuid();
  v_actor uuid := gen_random_uuid();
  v_batch uuid := gen_random_uuid();
  v_blocked boolean := false;
BEGIN
  INSERT INTO bim_projects(id,name,owner_account_id) VALUES(v_project,'receipt test',v_actor);
  INSERT INTO bim_change_batch_receipts(project_id,account_id,client_batch_id,request_sha256,response_json)
    VALUES(v_project,v_actor,v_batch,repeat('a',64),'{"accepted":[]}'::jsonb);
  BEGIN
    INSERT INTO bim_change_batch_receipts(project_id,account_id,client_batch_id,request_sha256,response_json)
      VALUES(v_project,v_actor,v_batch,repeat('b',64),'{"accepted":[]}'::jsonb);
  EXCEPTION WHEN unique_violation THEN v_blocked := true;
  END;
  IF NOT v_blocked THEN RAISE EXCEPTION 'duplicate receipt was inserted'; END IF;
  IF (SELECT count(*) FROM bim_change_batch_receipts r WHERE r.project_id=v_project) <> 1 THEN
    RAISE EXCEPTION 'duplicate receipt altered state';
  END IF;
  RAISE NOTICE 'CAD360 batch idempotency schema PASS';
END $$;
ROLLBACK;
