-- Run in a disposable CI PostgreSQL database AFTER base schema, 001, 002, 007.
-- Every fixture is rolled back. psql -v ON_ERROR_STOP=1 -f backend/postgres/tests/007_write_gate_regression.sql
BEGIN;
DO $$
DECLARE
    v_project uuid := gen_random_uuid();
    floor_id uuid := gen_random_uuid();
    object_id uuid := gen_random_uuid();
    owner_id uuid := gen_random_uuid();
    viewer_id uuid := gen_random_uuid();
    second_id uuid := gen_random_uuid();
    new_rev bigint;
    project_rev bigint;
    denied boolean := false;
    conflict boolean := false;
    locked boolean := false;
BEGIN
    INSERT INTO bim_projects(id,name,owner_account_id) VALUES(v_project,'write gate test',owner_id);
    INSERT INTO bim_project_members(project_id,account_id,role,can_edit)
      VALUES(v_project,owner_id,'OWNER',true),(v_project,viewer_id,'VIEWER',false),(v_project,second_id,'DESIGNER',true);
    INSERT INTO bim_floors(id,project_id,code,name,sort_order,elevation_m,height_m)
      VALUES(floor_id,v_project,'L1','Level 1',1,0,3);
    INSERT INTO bim_design_objects(id,project_id,source_handle,floor_id,discipline,owner_account_id,entity_type,revision)
      VALUES(object_id,v_project,'HX1',floor_id,'ARCH',owner_id,'LINE',1);

    -- Unauthorized viewer cannot change anything.
    BEGIN
        PERFORM bim_update_design_object(v_project,object_id,viewer_id,1,'{"points":[]}'::jsonb);
    EXCEPTION WHEN insufficient_privilege THEN denied := true;
    END;
    IF NOT denied THEN RAISE EXCEPTION 'viewer was allowed to write'; END IF;

    new_rev := bim_update_design_object(v_project,object_id,owner_id,1,'{"points":[1]}'::jsonb);
    IF new_rev <> 2 THEN RAISE EXCEPTION 'first update revision incorrect'; END IF;
    SELECT p.revision INTO project_rev FROM bim_projects p WHERE p.id=v_project;
    IF project_rev <> 1 THEN RAISE EXCEPTION 'project revision increment incorrect'; END IF;

    -- Stale writer cannot overwrite a newer revision.
    BEGIN
        PERFORM bim_update_design_object(v_project,object_id,second_id,1,'{"points":[2]}'::jsonb);
    EXCEPTION WHEN serialization_failure THEN conflict := true;
    END;
    IF NOT conflict THEN RAISE EXCEPTION 'stale write was accepted'; END IF;

    -- Active lease by another editor locks the object.
    INSERT INTO bim_object_leases(project_id,design_object_id,account_id,device_id,acquired_revision,expires_at)
      VALUES(v_project,object_id,owner_id,'device-1',2,now()+interval '10 minutes');
    BEGIN
        PERFORM bim_update_design_object(v_project,object_id,second_id,2,'{"points":[3]}'::jsonb);
    EXCEPTION WHEN lock_not_available THEN locked := true;
    END;
    IF NOT locked THEN RAISE EXCEPTION 'leased object was updated by another editor'; END IF;

    IF (SELECT count(*) FROM bim_audit_log a WHERE a.project_id=v_project AND a.action='UPDATE') <> 1 THEN
        RAISE EXCEPTION 'failed writes modified audit log';
    END IF;
    IF (SELECT revision FROM bim_design_objects d WHERE d.id=object_id) <> 2 THEN
        RAISE EXCEPTION 'failed writes modified object revision';
    END IF;

    -- Explicit deny overrides an otherwise permitted project role.
    INSERT INTO bim_access_grants(project_id,viewer_account_id,floor_id,discipline,deny)
      VALUES(v_project,second_id,floor_id,'ARCH',true);
    IF bim_can_edit_object(v_project,second_id,floor_id,'ARCH',owner_id) THEN
        RAISE EXCEPTION 'explicit deny failed to override role';
    END IF;

    -- A different project cannot address this object's revision or geometry.
    DECLARE
        other_project uuid := gen_random_uuid();
        cross_project_rejected boolean := false;
    BEGIN
        INSERT INTO bim_projects(id,name,owner_account_id) VALUES(other_project,'isolation',owner_id);
        INSERT INTO bim_project_members(project_id,account_id,role,can_edit) VALUES(other_project,owner_id,'OWNER',true);
        BEGIN
            PERFORM bim_update_design_object(other_project,object_id,owner_id,2,'{"foreign":true}'::jsonb);
        EXCEPTION WHEN no_data_found THEN cross_project_rejected := true;
        END;
        IF NOT cross_project_rejected THEN RAISE EXCEPTION 'cross-project object update was accepted'; END IF;
    END;

    -- Membership revocation closes the gate.
    DELETE FROM bim_project_members m WHERE m.project_id=v_project AND m.account_id=second_id;
    IF bim_can_edit_object(v_project,second_id,floor_id,'ARCH',owner_id) THEN
        RAISE EXCEPTION 'revoked member can still edit';
    END IF;
    RAISE NOTICE 'CAD360 write gate PostgreSQL regression passed';
END $$;
ROLLBACK;
