use anyhow::{Context, Result};
use rayon::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use strum::{EnumIter, IntoEnumIterator};
use uuid::Uuid;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let output = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("automate_v3_project.json"));

    let project = ProjectBlueprint::seed("AutoMate v3 Greenfield");
    project
        .save_json(&output)
        .with_context(|| format!("failed to save project at {}", output.display()))?;

    let loaded = ProjectBlueprint::load_json(&output)
        .with_context(|| format!("failed to load project at {}", output.display()))?;

    let health = loaded.health_report();
    let estimate = loaded.estimate();
    let csv = loaded.object_csv();

    println!("== AutoMate v3 ==");
    println!("Project: {} ({})", loaded.name, loaded.project_uuid);
    println!("Objects: {}", loaded.objects.len());
    println!("Health: {}", health.join(" | "));
    println!(
        "Estimate hours => engineering: {:.2}, graphics: {:.2}, commissioning: {:.2}, total: {:.2}",
        estimate.engineering, estimate.graphics, estimate.commissioning, estimate.total
    );
    println!(
        "CSV preview:\n{}",
        csv.lines().take(4).collect::<Vec<_>>().join("\n")
    );

    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, EnumIter, JsonSchema)]
enum ObjectType {
    Building,
    Controller,
    Equipment,
    Point,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, EnumIter, JsonSchema)]
enum PointKind {
    AI,
    DI,
    AO,
    DO,
    NetworkX,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
enum HourMode {
    StaticByEquipment,
    PointsBased,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct BasObject {
    id: u64,
    parent_id: Option<u64>,
    object_type: ObjectType,
    name: String,
    equipment_type: Option<String>,
    template_name: Option<String>,
    point_kind: Option<PointKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct EquipmentTemplate {
    name: String,
    equipment_type: String,
    points: Vec<(String, PointKind)>,
    hour_mode: HourMode,
    engineering_hours: f32,
    engineering_hours_per_point: f32,
    graphics_hours: f32,
    graphics_hours_per_point: f32,
    commissioning_hours: f32,
    commissioning_hours_per_point: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct EstimatorSettings {
    complexity_factor: f32,
    renovation_factor: f32,
    integration_factor: f32,
    qa_percent: f32,
    pm_percent: f32,
    risk_percent: f32,
}

impl Default for EstimatorSettings {
    fn default() -> Self {
        Self {
            complexity_factor: 1.0,
            renovation_factor: 1.0,
            integration_factor: 1.0,
            qa_percent: 8.0,
            pm_percent: 12.0,
            risk_percent: 5.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ProjectBlueprint {
    schema_version: u32,
    project_uuid: Uuid,
    name: String,
    objects: Vec<BasObject>,
    templates: Vec<EquipmentTemplate>,
    overlay_nodes: BTreeMap<u64, [f32; 2]>,
    routed_lines: Vec<([f32; 2], [f32; 2])>,
    included_equipment_ids: BTreeSet<u64>,
    estimator: EstimatorSettings,
}

#[derive(Debug, Clone, Copy)]
struct EstimateTotals {
    engineering: f32,
    graphics: f32,
    commissioning: f32,
    total: f32,
}

impl ProjectBlueprint {
    fn seed(name: &str) -> Self {
        let templates = vec![EquipmentTemplate {
            name: "VAV Typical".to_string(),
            equipment_type: "VAV".to_string(),
            points: vec![
                ("Space Temp".to_string(), PointKind::AI),
                ("Damper Cmd".to_string(), PointKind::AO),
                ("Airflow".to_string(), PointKind::AI),
                ("Occ Sensor".to_string(), PointKind::DI),
            ],
            hour_mode: HourMode::PointsBased,
            engineering_hours: 2.0,
            engineering_hours_per_point: 0.30,
            graphics_hours: 1.0,
            graphics_hours_per_point: 0.12,
            commissioning_hours: 1.5,
            commissioning_hours_per_point: 0.18,
        }];

        let mut objects = vec![BasObject {
            id: 1,
            parent_id: None,
            object_type: ObjectType::Building,
            name: "Building A".to_string(),
            equipment_type: None,
            template_name: None,
            point_kind: None,
        }];

        objects.push(BasObject {
            id: 2,
            parent_id: Some(1),
            object_type: ObjectType::Controller,
            name: "Controller-001".to_string(),
            equipment_type: None,
            template_name: None,
            point_kind: None,
        });

        objects.push(BasObject {
            id: 3,
            parent_id: Some(2),
            object_type: ObjectType::Equipment,
            name: "VAV-01".to_string(),
            equipment_type: Some("VAV".to_string()),
            template_name: Some("VAV Typical".to_string()),
            point_kind: None,
        });

        for (idx, (label, kind)) in templates[0].points.iter().enumerate() {
            objects.push(BasObject {
                id: 4 + idx as u64,
                parent_id: Some(3),
                object_type: ObjectType::Point,
                name: label.clone(),
                equipment_type: None,
                template_name: None,
                point_kind: Some(*kind),
            });
        }

        Self {
            schema_version: 3,
            project_uuid: Uuid::new_v4(),
            name: name.to_string(),
            objects,
            templates,
            overlay_nodes: BTreeMap::from([(2, [120.0, 100.0]), (3, [340.0, 220.0])]),
            routed_lines: vec![([120.0, 100.0], [340.0, 220.0])],
            included_equipment_ids: BTreeSet::from([3]),
            estimator: EstimatorSettings::default(),
        }
    }

    fn save_json(&self, path: &Path) -> Result<()> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }

    fn load_json(path: &Path) -> Result<Self> {
        let data = fs::read_to_string(path)?;
        let project = serde_json::from_str::<Self>(&data)?;
        Ok(project)
    }

    fn estimate(&self) -> EstimateTotals {
        let template_by_name: BTreeMap<&str, &EquipmentTemplate> = self
            .templates
            .iter()
            .map(|t| (t.name.as_str(), t))
            .collect();

        let (eng, gfx, cx) = self
            .objects
            .par_iter()
            .filter(|o| o.object_type == ObjectType::Equipment)
            .filter_map(|o| {
                o.template_name
                    .as_deref()
                    .and_then(|n| template_by_name.get(n).copied())
            })
            .map(|t| {
                let point_count = t.points.len() as f32;
                match t.hour_mode {
                    HourMode::StaticByEquipment => {
                        (t.engineering_hours, t.graphics_hours, t.commissioning_hours)
                    }
                    HourMode::PointsBased => (
                        t.engineering_hours + (point_count * t.engineering_hours_per_point),
                        t.graphics_hours + (point_count * t.graphics_hours_per_point),
                        t.commissioning_hours + (point_count * t.commissioning_hours_per_point),
                    ),
                }
            })
            .reduce(|| (0.0, 0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1, a.2 + b.2));

        let multiplier = self.estimator.complexity_factor
            * self.estimator.renovation_factor
            * self.estimator.integration_factor;
        let subtotal = (eng + gfx + cx) * multiplier;
        let overhead_pct =
            (self.estimator.qa_percent + self.estimator.pm_percent + self.estimator.risk_percent)
                / 100.0;
        let total = subtotal * (1.0 + overhead_pct);

        EstimateTotals {
            engineering: eng,
            graphics: gfx,
            commissioning: cx,
            total,
        }
    }

    fn health_report(&self) -> Vec<String> {
        let known_ids: BTreeSet<u64> = self.objects.iter().map(|o| o.id).collect();
        let mut issues = Vec::new();

        for obj in &self.objects {
            if let Some(parent) = obj.parent_id {
                if !known_ids.contains(&parent) {
                    issues.push(format!("dangling parent ref: {} -> {}", obj.id, parent));
                }
            }
        }

        let duplicates = self
            .objects
            .iter()
            .map(|o| o.id)
            .fold(BTreeMap::<u64, usize>::new(), |mut map, id| {
                *map.entry(id).or_insert(0) += 1;
                map
            })
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .count();

        if duplicates > 0 {
            issues.push(format!("duplicate ids detected: {}", duplicates));
        }

        if issues.is_empty() {
            issues.push("healthy graph".to_string());
        }

        issues
    }

    fn object_csv(&self) -> String {
        let mut lines = vec!["id,parent_id,type,name,point_kind".to_string()];
        for obj in &self.objects {
            let point = obj
                .point_kind
                .map(|k| format!("{k:?}"))
                .unwrap_or_else(|| "".to_string());
            lines.push(format!(
                "{},{},{:?},{},{}",
                obj.id,
                obj.parent_id.map(|v| v.to_string()).unwrap_or_default(),
                obj.object_type,
                obj.name,
                point
            ));
        }
        lines.join("\n")
    }
}

impl PointKind {
    #[allow(dead_code)]
    fn labels() -> Vec<&'static str> {
        PointKind::iter()
            .map(|k| match k {
                PointKind::AI => "AI",
                PointKind::DI => "DI",
                PointKind::AO => "AO",
                PointKind::DO => "DO",
                PointKind::NetworkX => "Network(X)",
            })
            .collect()
    }
}
