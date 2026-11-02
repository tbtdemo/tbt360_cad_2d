-- HTTP batch contract at database level: any failed item must roll back
-- all earlier edits and the idempotency receipt together.
BEGIN;
DO $$
DECLARE
  p uuid := gen_random_uuid();
  a uuid := gen_random_uuid();
  fl uuid := gen_random_uuid();
  ob uuid := gen_random_uuid();
  batch uuid := gen_random_uuid();
  blocked boolean := false;
BEGIN
  INSERT INTO bim_projects(id,name,owner_account_id) VALUES(p,'batch atomicity',a);
  INSERT INTO bim_project_members(project_id,account_id,role,can_edit) VALUES(p,a,'OWNER',true);
  INSERT INTO bim_floors(id,project_id,code,name,sort_order,elevation_m,height_m)
    VALUES(fl,p,'B1','Batch Floor',1,0,3);
  BEGIN
    PERFORM bim_create_design_object(p,ob,a,fl,'ARCH','BAT-1','LINE','{}'::jsonb);
    PERFORM bim_update_design_object(p,ob,a,999,'{}'::jsonb);
    INSERT INTO bim_change_batch_receipts(project_id,account_id,client_batch_id,request_sha256,response_json)
      VALUES(p,a,batch,repeat('a',64),'{"accepted":[]}'::jsonb);
  EXCEPTION WHEN OTHERS THEN blocked := true;
  END;
  IF NOT blocked THEN RAISE EXCEPTION 'stale revision did not fail batch'; END IF;
  IF EXISTS(SELECT 1 FROM bim_design_objects WHERE project_id=p)
    OR EXISTS(SELECT 1 FROM bim_audit_log WHERE project_id=p)
    OR EXISTS(SELECT 1 FROM bim_project_change_feed WHERE project_id=p)
    OR EXISTS(SELECT 1 FROM bim_change_batch_receipts WHERE project_id=p)
    OR (SELECT revision FROM bim_projects WHERE id=p) <> 0 THEN
      RAISE EXCEPTION 'failed batch leaked an object, audit, feed, revision or receipt';
  END IF;
  RAISE NOTICE 'CAD360 atomic batch rollback PASS';
END $$;
ROLLBACK;
