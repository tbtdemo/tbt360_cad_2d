CREATE TABLE IF NOT EXISTS bim_access_grants (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
    viewer_account_id uuid NOT NULL,
    source_owner_account_id uuid NULL,
    building_id uuid NULL REFERENCES bim_buildings(id) ON DELETE CASCADE,
    zone_id uuid NULL REFERENCES bim_zones(id) ON DELETE CASCADE,
    floor_id uuid NULL REFERENCES bim_floors(id) ON DELETE CASCADE,
    discipline text NULL,
    workset_id uuid NULL,
    can_view boolean NOT NULL DEFAULT false,
    can_edit boolean NOT NULL DEFAULT false,
    can_publish boolean NOT NULL DEFAULT false,
    deny boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ix_bim_access_grants_lookup ON bim_access_grants(project_id, viewer_account_id, source_owner_account_id, floor_id, discipline);

-- Read-only policy helper; callers must supply the authenticated account identity.
-- Explicit deny takes precedence over grants. Default is no permission.
CREATE OR REPLACE FUNCTION bim_can_edit_project(p_project_id uuid, p_account_id uuid)
RETURNS boolean LANGUAGE sql STABLE AS $$
    SELECT EXISTS (
        SELECT 1 FROM bim_access_grants g
        WHERE p_project_id IS NOT NULL AND p_account_id IS NOT NULL AND g.project_id = p_project_id AND g.viewer_account_id = p_account_id
          AND g.can_edit AND NOT g.deny AND g.source_owner_account_id IS NULL
          AND g.building_id IS NULL AND g.zone_id IS NULL AND g.floor_id IS NULL
          AND g.discipline IS NULL AND g.workset_id IS NULL
    ) AND NOT EXISTS (
        SELECT 1 FROM bim_access_grants d
        WHERE p_project_id IS NOT NULL AND p_account_id IS NOT NULL AND d.project_id = p_project_id AND d.viewer_account_id = p_account_id
          AND d.deny AND d.source_owner_account_id IS NULL
          AND d.building_id IS NULL AND d.zone_id IS NULL AND d.floor_id IS NULL
          AND d.discipline IS NULL AND d.workset_id IS NULL
    );
$$;
