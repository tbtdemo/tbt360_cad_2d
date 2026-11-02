-- Verify revocation is effective even after prior successful changes.
BEGIN;
DO $$
DECLARE p uuid:=gen_random_uuid(); owner uuid:=gen_random_uuid(); user_id uuid:=gen_random_uuid(); fl uuid:=gen_random_uuid(); ob uuid:=gen_random_uuid(); rejected boolean:=false; rev bigint;
BEGIN
 INSERT INTO bim_projects(id,name,owner_account_id) VALUES(p,'revocation',owner);
 INSERT INTO bim_project_members(project_id,account_id,role,can_edit) VALUES(p,owner,'OWNER',true),(p,user_id,'DESIGNER',true);
 INSERT INTO bim_floors(id,project_id,code,name,sort_order,elevation_m,height_m) VALUES(fl,p,'R1','Revocation',1,0,3);
 rev:=bim_create_design_object(p,ob,user_id,fl,'ARCH','REVOKE-1','LINE','{}'::jsonb);
 -- Downgrade a member to view-only: change feed remains readable, writes must fail.
 UPDATE bim_project_members SET can_edit=false WHERE project_id=p AND account_id=user_id;
 BEGIN PERFORM bim_update_design_object(p,ob,user_id,rev,'{"readonly":false}'::jsonb);
 EXCEPTION WHEN insufficient_privilege THEN rejected:=true; END;
 IF NOT rejected THEN RAISE EXCEPTION 'read-only member updated an object'; END IF;
 IF (SELECT count(*) FROM bim_pull_project_changes(p,user_id,0,100))<>1 THEN RAISE EXCEPTION 'read-only member cannot read authorized change feed'; END IF;
 IF (SELECT revision FROM bim_design_objects WHERE id=ob)<>rev THEN RAISE EXCEPTION 'read-only denial mutated object'; END IF;
 -- Re-enable editing and verify the original revision remains usable for a legitimate write.
 UPDATE bim_project_members SET can_edit=true WHERE project_id=p AND account_id=user_id;
 rev:=bim_update_design_object(p,ob,user_id,rev,'{"restored":true}'::jsonb);
 IF rev<>2 THEN RAISE EXCEPTION 'restored edit permission did not advance revision'; END IF;
 rejected:=false;
 DELETE FROM bim_project_members WHERE project_id=p AND account_id=user_id;
 BEGIN PERFORM bim_update_design_object(p,ob,user_id,rev,'{"new":true}'::jsonb);
 EXCEPTION WHEN insufficient_privilege THEN rejected:=true; END;
 IF NOT rejected THEN RAISE EXCEPTION 'revoked user updated an object'; END IF;
 rejected:=false;
 BEGIN PERFORM bim_pull_project_changes(p,user_id,0,100);
 EXCEPTION WHEN insufficient_privilege THEN rejected:=true; END;
 IF NOT rejected THEN RAISE EXCEPTION 'revoked user accessed change feed'; END IF;
 IF (SELECT revision FROM bim_design_objects WHERE id=ob)<>rev THEN RAISE EXCEPTION 'revocation failure mutated object'; END IF;
 RAISE NOTICE 'revocation gates PASS';
END $$;
ROLLBACK;
