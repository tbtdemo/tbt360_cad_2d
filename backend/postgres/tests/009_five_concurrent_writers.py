-- PostgreSQL concurrent-writer test. Run with python3 and psycopg installed.
-- Environment: PGHOST PGPORT PGDATABASE PGUSER PGPASSWORD.
-- Apply migrations 001,002,007,008 before running.
import concurrent.futures
import os
import threading
import uuid

import psycopg

conninfo = "host={host} port={port} dbname={db} user={user} password={password}".format(
    host=os.environ.get("PGHOST", "localhost"), port=os.environ.get("PGPORT", "5432"),
    db=os.environ.get("PGDATABASE", "cad360_ci"), user=os.environ.get("PGUSER", "cad360"),
    password=os.environ.get("PGPASSWORD", ""))
project, floor, obj, owner = (uuid.uuid4() for _ in range(4))
accounts = [owner] + [uuid.uuid4() for _ in range(4)]

with psycopg.connect(conninfo) as conn:
    with conn.cursor() as cur:
        cur.execute("INSERT INTO bim_projects(id,name,owner_account_id) VALUES(%s,'concurrent regression',%s)", (project, owner))
        cur.execute("INSERT INTO bim_floors(id,project_id,code,name,sort_order,elevation_m,height_m) VALUES(%s,%s,'F1','Floor 1',1,0,3)", (floor, project))
        for i, user in enumerate(accounts):
            cur.execute("INSERT INTO bim_project_members(project_id,account_id,role,can_edit) VALUES(%s,%s,%s,true)", (project,user,"OWNER" if i == 0 else "DESIGNER"))
        cur.execute("INSERT INTO bim_design_objects(id,project_id,source_handle,floor_id,discipline,owner_account_id,entity_type,revision) VALUES(%s,%s,'shared-handle',%s,'ARCH',%s,'LINE',1)", (obj,project,floor,owner))

barrier = threading.Barrier(5)
def writer(idx):
    barrier.wait(timeout=15)
    try:
        with psycopg.connect(conninfo) as conn:
            with conn.cursor() as cur:
                cur.execute("SELECT bim_update_design_object(%s,%s,%s,1,%s::jsonb)", (project,obj,accounts[idx],'{"editor":%d}' % idx))
                return ("accepted", cur.fetchone()[0])
    except psycopg.Error as exc:
        return (exc.sqlstate, None)

try:
    with concurrent.futures.ThreadPoolExecutor(max_workers=5) as pool:
        outcomes = list(pool.map(writer, range(5)))
    successes = [r for r in outcomes if r[0] == "accepted"]
    conflicts = [r for r in outcomes if r[0] == "40001"]
    assert len(successes) == 1, outcomes
    assert len(conflicts) == 4, outcomes
    with psycopg.connect(conninfo) as conn:
        with conn.cursor() as cur:
            cur.execute("SELECT revision FROM bim_design_objects WHERE id=%s", (obj,))
            assert cur.fetchone()[0] == 2
            cur.execute("SELECT revision FROM bim_projects WHERE id=%s", (project,))
            assert cur.fetchone()[0] == 1
            cur.execute("SELECT count(*) FROM bim_project_change_feed WHERE project_id=%s", (project,))
            assert cur.fetchone()[0] == 1
            cur.execute("SELECT count(*) FROM bim_audit_log WHERE project_id=%s AND action='UPDATE'", (project,))
            assert cur.fetchone()[0] == 1
    print("PASS: 5 concurrent PostgreSQL writers, 1 commit, 4 CAS conflicts, 1 feed event")
finally:
    with psycopg.connect(conninfo) as conn:
        with conn.cursor() as cur:
            cur.execute("DELETE FROM bim_projects WHERE id=%s", (project,))
