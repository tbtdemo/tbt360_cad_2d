//! BIM shared/project coordinate hierarchy.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CoordinateFrame {
    pub origin: Point3,
    pub rotation_deg: f64,
}

impl CoordinateFrame {
    pub fn local_to_parent(self, p: Point3) -> Point3 {
        let a = self.rotation_deg.to_radians();
        let (c, s) = (a.cos(), a.sin());
        Point3 {
            x: self.origin.x + p.x * c - p.y * s,
            y: self.origin.y + p.x * s + p.y * c,
            z: self.origin.z + p.z,
        }
    }

    pub fn parent_to_local(self, p: Point3) -> Point3 {
        let dx = p.x - self.origin.x;
        let dy = p.y - self.origin.y;
        let a = (-self.rotation_deg).to_radians();
        let (c, s) = (a.cos(), a.sin());
        Point3 { x: dx * c - dy * s, y: dx * s + dy * c, z: p.z - self.origin.z }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Building {
    pub id: String,
    pub code: String,
    pub name: String,
    pub frame: CoordinateFrame,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Zone {
    pub id: String,
    pub building_id: String,
    pub code: String,
    pub name: String,
    pub frame: CoordinateFrame,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Level {
    pub id: String,
    pub building_id: String,
    pub zone_id: Option<String>,
    pub code: String,
    pub name: String,
    pub elevation_m: f64,
    pub height_m: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SharedCoordinates {
    pub survey_point: Point3,
    pub project_base_point: Point3,
    pub true_north_deg: f64,
}

impl Default for SharedCoordinates {
    fn default() -> Self {
        Self { survey_point: Point3::default(), project_base_point: Point3::default(), true_north_deg: 0.0 }
    }
}

impl SharedCoordinates {
    pub fn project_to_shared(&self, p: Point3) -> Point3 {
        let frame = CoordinateFrame { origin: self.survey_point, rotation_deg: self.true_north_deg };
        frame.local_to_parent(Point3 {
            x: p.x - self.project_base_point.x,
            y: p.y - self.project_base_point.y,
            z: p.z - self.project_base_point.z,
        })
    }

    pub fn shared_to_project(&self, p: Point3) -> Point3 {
        let frame = CoordinateFrame { origin: self.survey_point, rotation_deg: self.true_north_deg };
        let local = frame.parent_to_local(p);
        Point3 {
            x: local.x + self.project_base_point.x,
            y: local.y + self.project_base_point.y,
            z: local.z + self.project_base_point.z,
        }
    }
}

pub fn validate_levels(levels: &[Level]) -> Result<(), String> {
    for l in levels {
        if l.building_id.trim().is_empty() || l.id.trim().is_empty() || l.code.trim().is_empty() {
            return Err("Level thiếu mã hoặc định danh tòa nhà".into());
        }
        if !l.elevation_m.is_finite() || !l.height_m.is_finite() || l.height_m <= 0.0 {
            return Err(format!("Level {} có cao độ/chiều cao không hợp lệ", l.code));
        }
    }
    // Group by building first: a level from another building must never hide an overlap.
    let mut sorted = levels.iter().collect::<Vec<_>>();
    sorted.sort_by(|a,b| a.building_id.cmp(&b.building_id).then_with(|| a.elevation_m.total_cmp(&b.elevation_m)));
    for pair in sorted.windows(2) {
        if pair[0].building_id == pair[1].building_id && pair[1].elevation_m < pair[0].elevation_m + pair[0].height_m - 1e-9 {
            return Err(format!("Level {} chồng cao độ với {}", pair[0].code, pair[1].code));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_base_and_survey_point_map_to_shared_coordinates() {
        let c = SharedCoordinates {
            survey_point: Point3 { x: 1000.0, y: 2000.0, z: 50.0 },
            project_base_point: Point3 { x: 10.0, y: 20.0, z: 5.0 },
            true_north_deg: 0.0,
        };
        assert_eq!(c.project_to_shared(Point3 { x: 12.0, y: 23.0, z: 6.0 }), Point3 { x: 1002.0, y: 2003.0, z: 51.0 });
    }

    #[test]
    fn shared_coordinate_roundtrip_is_stable() {
        let c = SharedCoordinates {
            survey_point: Point3 { x: 1000.0, y: 2000.0, z: 50.0 },
            project_base_point: Point3 { x: 10.0, y: 20.0, z: 5.0 },
            true_north_deg: 23.0,
        };
        let p = Point3 { x: 14.0, y: 35.0, z: 9.0 };
        let r = c.shared_to_project(c.project_to_shared(p));
        assert!((r.x-p.x).abs()<1e-9 && (r.y-p.y).abs()<1e-9 && (r.z-p.z).abs()<1e-9);
    }

    #[test]
    fn building_and_zone_frames_compose_without_losing_height() {
        let building = CoordinateFrame { origin: Point3 { x: 100.0, y: 0.0, z: 10.0 }, rotation_deg: 90.0 };
        let zone = CoordinateFrame { origin: Point3 { x: 5.0, y: 0.0, z: 2.0 }, rotation_deg: 0.0 };
        let p = building.local_to_parent(zone.local_to_parent(Point3 { x: 1.0, y: 0.0, z: 3.0 }));
        assert!((p.x - 100.0).abs() < 1e-10);
        assert!((p.y - 6.0).abs() < 1e-10);
        assert!((p.z - 15.0).abs() < 1e-10);
    }
    #[test]
    fn overlapping_levels_in_same_building_are_detected_even_with_interleaved_buildings() {
        let level = |building: &str, code: &str, elevation: f64, height: f64| Level {
            id: code.into(), building_id: building.into(), zone_id: None, code: code.into(),
            name: code.into(), elevation_m: elevation, height_m: height,
        };
        // In elevation-only order: A0, B1, A2. Previous adjacent-window logic missed A0/A2.
        let levels = vec![level("A", "A0", 0.0, 3.0), level("B", "B1", 1.0, 3.0), level("A", "A2", 2.0, 3.0)];
        assert!(validate_levels(&levels).is_err());
    }

    #[test]
    fn overlapping_elevations_in_different_buildings_are_allowed() {
        let levels = vec![
            Level { id: "1".into(), building_id: "A".into(), zone_id: None, code: "L1".into(), name: "L1".into(), elevation_m: 0.0, height_m: 3.0 },
            Level { id: "2".into(), building_id: "B".into(), zone_id: None, code: "L1".into(), name: "L1".into(), elevation_m: 0.0, height_m: 3.0 },
        ];
        assert!(validate_levels(&levels).is_ok());
    }

    #[test]
    fn touching_levels_in_one_building_are_allowed() {
        let levels = vec![
            Level { id: "1".into(), building_id: "A".into(), zone_id: None, code: "L1".into(), name: "L1".into(), elevation_m: 0.0, height_m: 3.0 },
            Level { id: "2".into(), building_id: "A".into(), zone_id: None, code: "L2".into(), name: "L2".into(), elevation_m: 3.0, height_m: 3.0 },
        ];
        assert!(validate_levels(&levels).is_ok());
    }

    #[test]
    fn levels_with_empty_building_identity_are_rejected() {
        let invalid = Level { id: "L1".into(), building_id: " ".into(), zone_id: None, code: "L1".into(), name: "Level 1".into(), elevation_m: 0.0, height_m: 3.0 };
        assert!(validate_levels(&[invalid]).is_err());
    }

    #[test]
    fn invalid_nan_and_negative_level_heights_are_rejected() {
        for height in [f64::NAN, f64::INFINITY, -1.0, 0.0] {
            let level = Level { id: "L1".into(), building_id: "A".into(), zone_id: None, code: "L1".into(), name: "L1".into(), elevation_m: 0.0, height_m: height };
            assert!(validate_levels(&[level]).is_err());
        }
    }

}
