use eframe::{
    egui::{self, menu, Color32, FontFamily, FontId, RichText, Ui},
    epaint::{Mesh, Shadow, Vertex},
    App, CreationContext, Frame, NativeOptions,
};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::PathBuf};

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true)
            .with_decorations(false)
            .with_inner_size([1600.0, 920.0]),
        ..Default::default()
    };

    eframe::run_native(
        "AutoMate BAS Studio",
        options,
        Box::new(|cc| Ok(Box::new(AutoMateApp::new(cc)))),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToolView {
    ProjectSettings,
    HoursEstimator,
    DrawingsOverlay,
    Templates,
}

impl ToolView {
    fn label(self) -> &'static str {
        match self {
            ToolView::ProjectSettings => "Project Settings",
            ToolView::HoursEstimator => "Hours Estimator",
            ToolView::DrawingsOverlay => "Drawings Overlay",
            ToolView::Templates => "Template Tool",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
enum ObjectType {
    Building,
    Controller,
    Equipment,
    Point,
}

impl ObjectType {
    fn label(self) -> &'static str {
        match self {
            ObjectType::Building => "Building",
            ObjectType::Controller => "Controller",
            ObjectType::Equipment => "Equipment",
            ObjectType::Point => "Point",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            ObjectType::Building => "🏢",
            ObjectType::Controller => "🧠",
            ObjectType::Equipment => "🛠",
            ObjectType::Point => "📍",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PropertyGroup {
    name: String,
    items: Vec<PropertyItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PropertyItem {
    key: String,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BasObject {
    id: u64,
    parent_id: Option<u64>,
    object_type: ObjectType,
    name: String,
    #[serde(default)]
    equipment_type: String,
    #[serde(default)]
    equipment_tag: String,
    #[serde(default)]
    make: String,
    #[serde(default)]
    model: String,
    #[serde(default)]
    controller_type: String,
    #[serde(default)]
    controller_license: String,
    #[serde(default)]
    template_name: String,
    property_groups: Vec<PropertyGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct OverlayNode {
    id: u64,
    object_id: u64,
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct OverlayLine {
    from: [f32; 2],
    to: [f32; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppSettings {
    accent_color: [u8; 4],
    company_name: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            accent_color: [74, 154, 255, 255],
            company_name: "AutoMate Controls".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ProposalData {
    client_name: String,
    project_location: String,
    proposal_number: String,
    revision: String,
    bid_date: String,
    prepared_by: String,
    scope_summary: String,
    assumptions: String,
    exclusions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HourLine {
    name: String,
    category: String,
    quantity: f32,
    hours_per_unit: f32,
}

impl Default for HourLine {
    fn default() -> Self {
        Self {
            name: "Custom line".to_string(),
            category: "Engineering".to_string(),
            quantity: 1.0,
            hours_per_unit: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EquipmentTemplate {
    name: String,
    equipment_type: String,
    points: Vec<String>,
    engineering_hours: f32,
    graphics_hours: f32,
    commissioning_hours: f32,
}

impl Default for EquipmentTemplate {
    fn default() -> Self {
        Self {
            name: "VAV Typical".to_string(),
            equipment_type: "VAV".to_string(),
            points: vec![
                "Space Temp".to_string(),
                "Discharge Temp".to_string(),
                "Damper Cmd".to_string(),
                "Airflow".to_string(),
            ],
            engineering_hours: 2.0,
            graphics_hours: 1.0,
            commissioning_hours: 1.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Project {
    name: String,
    notes: String,
    proposal: ProposalData,
    objects: Vec<BasObject>,
    overlay_pdf: Option<String>,
    overlay_nodes: Vec<OverlayNode>,
    overlay_lines: Vec<OverlayLine>,
    #[serde(default)]
    templates: Vec<EquipmentTemplate>,
    #[serde(default)]
    custom_hour_lines: Vec<HourLine>,
    next_id: u64,
    settings: AppSettings,
    #[serde(default)]
    overview_image: Option<String>,
}

impl Default for Project {
    fn default() -> Self {
        let building = BasObject {
            id: 1,
            parent_id: None,
            object_type: ObjectType::Building,
            name: "HQ Building".to_string(),
            equipment_type: String::new(),
            equipment_tag: String::new(),
            make: String::new(),
            model: String::new(),
            controller_type: String::new(),
            controller_license: String::new(),
            template_name: String::new(),
            property_groups: vec![PropertyGroup {
                name: "General".to_string(),
                items: vec![
                    PropertyItem {
                        key: "Address".to_string(),
                        value: "100 Main St".to_string(),
                    },
                    PropertyItem {
                        key: "Square Footage".to_string(),
                        value: "125000".to_string(),
                    },
                ],
            }],
        };

        Self {
            name: "New BAS Project".to_string(),
            notes: "Capture assumptions, scope notes, and exclusions here.".to_string(),
            proposal: ProposalData::default(),
            objects: vec![building],
            overlay_pdf: None,
            overlay_nodes: vec![],
            overlay_lines: vec![],
            templates: vec![EquipmentTemplate::default()],
            custom_hour_lines: vec![],
            next_id: 2,
            settings: AppSettings::default(),
            overview_image: None,
        }
    }
}

struct AutoMateApp {
    project: Project,
    current_view: ToolView,
    selected_object: Option<u64>,
    status: String,
    project_path: Option<PathBuf>,
    show_about: bool,
    show_software_settings: bool,
    dragging_palette: Option<ObjectType>,
    active_line_start: Option<[f32; 2]>,
    is_fullscreen: bool,
}

impl AutoMateApp {
    fn new(cc: &CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        Self {
            project: Project::default(),
            current_view: ToolView::ProjectSettings,
            selected_object: Some(1),
            status: "Ready".to_string(),
            project_path: None,
            show_about: false,
            show_software_settings: false,
            dragging_palette: None,
            active_line_start: None,
            is_fullscreen: true,
        }
    }

    fn accent(&self) -> Color32 {
        let [r, g, b, a] = self.project.settings.accent_color;
        Color32::from_rgba_unmultiplied(r, g, b, a)
    }

    fn surface_panel() -> egui::Frame {
        egui::Frame::default()
            .fill(Color32::from_rgba_unmultiplied(27, 30, 35, 242))
            .stroke(egui::Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 28),
            ))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(14.0))
            .outer_margin(egui::Margin::same(2.0))
            .shadow(Shadow {
                offset: egui::vec2(0.0, 6.0),
                blur: 16.0,
                spread: 0.0,
                color: Color32::from_rgba_unmultiplied(0, 0, 0, 95),
            })
    }

    fn card_frame() -> egui::Frame {
        egui::Frame::default()
            .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 10))
            .stroke(egui::Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 26),
            ))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(8.0))
    }

    fn draw_studio_background(&self, ctx: &egui::Context) {
        let rect = ctx.screen_rect();
        let accent = self.accent();
        let top = Color32::from_rgba_unmultiplied(23, 25, 29, 255);
        let bottom = Color32::from_rgba_unmultiplied(16, 17, 20, 255);

        let mut mesh = Mesh::default();
        let i = mesh.vertices.len() as u32;
        mesh.vertices.push(Vertex {
            pos: rect.left_top(),
            uv: Default::default(),
            color: top,
        });
        mesh.vertices.push(Vertex {
            pos: rect.right_top(),
            uv: Default::default(),
            color: top,
        });
        mesh.vertices.push(Vertex {
            pos: rect.right_bottom(),
            uv: Default::default(),
            color: bottom,
        });
        mesh.vertices.push(Vertex {
            pos: rect.left_bottom(),
            uv: Default::default(),
            color: bottom,
        });
        mesh.indices
            .extend_from_slice(&[i, i + 1, i + 2, i, i + 2, i + 3]);
        ctx.layer_painter(egui::LayerId::background())
            .add(egui::Shape::mesh(mesh));

        let glow_radius = rect.width().min(rect.height()) * 0.55;
        let glow_center = egui::pos2(
            rect.right() - glow_radius * 0.32,
            rect.top() + glow_radius * 0.35,
        );
        ctx.layer_painter(egui::LayerId::background())
            .circle_filled(
                glow_center,
                glow_radius,
                Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 24),
            );
    }

    fn workspace_header(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Workspace").strong().size(16.0));
            ui.separator();
            for view in [
                ToolView::ProjectSettings,
                ToolView::HoursEstimator,
                ToolView::DrawingsOverlay,
                ToolView::Templates,
            ] {
                let is_selected = self.current_view == view;
                if ui.selectable_label(is_selected, view.label()).clicked() {
                    self.current_view = view;
                }
            }
        });

        ui.add_space(8.0);
        Self::card_frame().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                let buildings = self
                    .project
                    .objects
                    .iter()
                    .filter(|o| o.object_type == ObjectType::Building)
                    .count();
                let controllers = self
                    .project
                    .objects
                    .iter()
                    .filter(|o| o.object_type == ObjectType::Controller)
                    .count();
                let equipment = self
                    .project
                    .objects
                    .iter()
                    .filter(|o| o.object_type == ObjectType::Equipment)
                    .count();
                let points = self
                    .project
                    .objects
                    .iter()
                    .filter(|o| o.object_type == ObjectType::Point)
                    .count();

                ui.monospace(format!("Buildings: {buildings}"));
                ui.separator();
                ui.monospace(format!("Controllers: {controllers}"));
                ui.separator();
                ui.monospace(format!("Equipment: {equipment}"));
                ui.separator();
                ui.monospace(format!("Points: {points}"));
            });
        });
    }

    fn add_object(&mut self, object_type: ObjectType, parent: Option<u64>) {
        let id = self.project.next_id;
        self.project.next_id += 1;
        self.project.objects.push(BasObject {
            id,
            parent_id: parent,
            object_type,
            name: format!("{} {}", object_type.label(), id),
            equipment_type: String::new(),
            equipment_tag: String::new(),
            make: String::new(),
            model: String::new(),
            controller_type: "Lynxspring Edge".to_string(),
            controller_license: "None".to_string(),
            template_name: String::new(),
            property_groups: vec![PropertyGroup {
                name: "General".to_string(),
                items: vec![],
            }],
        });
        self.selected_object = Some(id);
    }

    fn save_project(&mut self) {
        let path = self.project_path.clone().or_else(|| {
            FileDialog::new()
                .add_filter("AutoMate Project", &["json"])
                .set_file_name("project.json")
                .save_file()
        });
        if let Some(path) = path {
            match serde_json::to_string_pretty(&self.project) {
                Ok(payload) => match fs::write(&path, payload) {
                    Ok(_) => {
                        self.status = format!("Saved {}", path.display());
                        self.project_path = Some(path);
                    }
                    Err(e) => self.status = format!("Save failed: {e}"),
                },
                Err(e) => self.status = format!("Serialization failed: {e}"),
            }
        }
    }

    fn load_project(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("AutoMate Project", &["json"])
            .pick_file()
        {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<Project>(&content) {
                    Ok(project) => {
                        self.project = project;
                        self.project_path = Some(path.clone());
                        self.status = format!("Loaded {}", path.display());
                        self.selected_object = self.project.objects.first().map(|o| o.id);
                    }
                    Err(e) => self.status = format!("Parse failed: {e}"),
                },
                Err(e) => self.status = format!("Load failed: {e}"),
            }
        }
    }

    fn titlebar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("titlebar")
            .frame(Self::surface_panel())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("AutoMate BAS Studio")
                            .font(FontId::new(22.0, FontFamily::Proportional))
                            .color(self.accent()),
                    );
                    ui.separator();
                    ui.label(
                        RichText::new(format!("PROJECT  {}", self.project.name.to_uppercase()))
                            .font(FontId::new(11.0, FontFamily::Monospace))
                            .color(Color32::from_rgba_unmultiplied(215, 215, 220, 190)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui
                            .button(if self.is_fullscreen { "🗗" } else { "🗖" })
                            .clicked()
                        {
                            self.is_fullscreen = !self.is_fullscreen;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                                self.is_fullscreen,
                            ));
                        }
                        if ui.button("—").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                    });
                });

                let drag_area = ui.max_rect();
                let response = ui.interact(
                    drag_area,
                    ui.id().with("titlebar_drag"),
                    egui::Sense::click_and_drag(),
                );
                if response.dragged() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
            });
    }

    fn toolbar_dropdowns(&mut self, ui: &mut Ui) {
        menu::bar(ui, |ui| {
            ui.menu_button("🧰 Tools", |ui| {
                for view in [
                    ToolView::ProjectSettings,
                    ToolView::HoursEstimator,
                    ToolView::DrawingsOverlay,
                    ToolView::Templates,
                ] {
                    if ui.button(view.label()).clicked() {
                        self.current_view = view;
                        ui.close_menu();
                    }
                }
            });
            ui.menu_button("📂 Project", |ui| {
                if ui.button("New").clicked() {
                    self.project = Project::default();
                    self.selected_object = Some(1);
                    self.project_path = None;
                    ui.close_menu();
                }
                if ui.button("Save").clicked() {
                    self.save_project();
                    ui.close_menu();
                }
                if ui.button("Load").clicked() {
                    self.load_project();
                    ui.close_menu();
                }
            });
            ui.menu_button("🎨 View", |ui| {
                if ui.button("Software Settings").clicked() {
                    self.show_software_settings = true;
                    ui.close_menu();
                }
            });
            ui.menu_button("ℹ Help", |ui| {
                if ui.button("About").clicked() {
                    self.show_about = true;
                    ui.close_menu();
                }
            });
        });
    }

    fn project_overview(&mut self, ui: &mut Ui) {
        Self::card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("🖼 Upload Overview Image").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg", "bmp"])
                        .pick_file()
                    {
                        self.project.overview_image = Some(path.display().to_string());
                    }
                }
                if let Some(path) = &self.project.overview_image {
                    ui.small(path);
                }
            });
            ui.separator();
            ui.label(RichText::new("Project Overview").strong());
            ui.label(format!("Client: {}", self.project.proposal.client_name));
            ui.label(format!(
                "Location: {}",
                self.project.proposal.project_location
            ));
            ui.label(format!(
                "Proposal #: {}",
                self.project.proposal.proposal_number
            ));
            ui.label(format!("Total Objects: {}", self.project.objects.len()));
        });
    }

    fn left_sidebar(&mut self, ui: &mut Ui) {
        self.project_overview(ui);
        ui.add_space(8.0);
        if ui.button("➕ Building").clicked() {
            self.add_object(ObjectType::Building, None);
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            let roots: Vec<u64> = self
                .project
                .objects
                .iter()
                .filter(|o| o.parent_id.is_none())
                .map(|o| o.id)
                .collect();
            for root in roots {
                self.object_node(ui, root);
                ui.add_space(6.0);
            }
        });
    }

    fn object_node(&mut self, ui: &mut Ui, id: u64) {
        let obj = self.project.objects.iter().find(|o| o.id == id).cloned();
        let Some(obj) = obj else { return };

        Self::card_frame().show(ui, |ui| {
            let selected = self.selected_object == Some(id);
            let title = format!("{} {}", obj.object_type.icon(), obj.name);
            if ui.selectable_label(selected, title).clicked() {
                self.selected_object = Some(id);
            }

            ui.horizontal_wrapped(|ui| match obj.object_type {
                ObjectType::Building => {
                    if ui.button("+ Controller").clicked() {
                        self.add_object(ObjectType::Controller, Some(id))
                    }
                }
                ObjectType::Controller => {
                    if ui.button("+ Equipment").clicked() {
                        self.add_object(ObjectType::Equipment, Some(id))
                    }
                }
                ObjectType::Equipment => {
                    if ui.button("+ Point").clicked() {
                        self.add_object(ObjectType::Point, Some(id))
                    }
                }
                ObjectType::Point => {}
            });

            let children: Vec<u64> = self
                .project
                .objects
                .iter()
                .filter(|child| child.parent_id == Some(id))
                .map(|child| child.id)
                .collect();

            for child in children {
                ui.indent(("child", child), |ui| self.object_node(ui, child));
            }
        });
    }

    fn apply_template_to_selected_equipment(&mut self) {
        let Some(obj_id) = self.selected_object else {
            return;
        };
        let Some(eq) = self
            .project
            .objects
            .iter()
            .find(|o| o.id == obj_id)
            .cloned()
        else {
            return;
        };
        if eq.object_type != ObjectType::Equipment || eq.template_name.is_empty() {
            return;
        }

        if let Some(template) = self
            .project
            .templates
            .iter()
            .find(|t| t.name == eq.template_name)
            .cloned()
        {
            let existing_points: Vec<String> = self
                .project
                .objects
                .iter()
                .filter(|o| o.parent_id == Some(obj_id) && o.object_type == ObjectType::Point)
                .map(|o| o.name.clone())
                .collect();

            for point_name in template.points {
                if existing_points.contains(&point_name) {
                    continue;
                }
                self.add_object(ObjectType::Point, Some(obj_id));
                if let Some(new_obj) = self.project.objects.last_mut() {
                    new_obj.name = point_name;
                }
            }
        }
    }

    fn right_properties(&mut self, ui: &mut Ui) {
        ui.heading("Properties");
        if let Some(id) = self.selected_object {
            if let Some(index) = self.project.objects.iter().position(|o| o.id == id) {
                let mut apply_template = false;
                let obj = &mut self.project.objects[index];
                Self::card_frame().show(ui, |ui| {
                    ui.label(format!(
                        "{} {}",
                        obj.object_type.icon(),
                        obj.object_type.label()
                    ));
                    ui.text_edit_singleline(&mut obj.name);

                    if obj.object_type == ObjectType::Controller {
                        ui.separator();
                        ui.label(RichText::new("Controller Data").strong());
                        egui::ComboBox::from_label("Controller Type")
                            .selected_text(if obj.controller_type.is_empty() {
                                "Select type"
                            } else {
                                &obj.controller_type
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut obj.controller_type,
                                    "Lynxspring Edge".to_string(),
                                    "Lynxspring Edge",
                                );
                                ui.selectable_value(
                                    &mut obj.controller_type,
                                    "JENEsys".to_string(),
                                    "JENEsys",
                                );
                            });

                        egui::ComboBox::from_label("License")
                            .selected_text(if obj.controller_license.is_empty() {
                                "Select license"
                            } else {
                                &obj.controller_license
                            })
                            .show_ui(ui, |ui| {
                                for lic in [
                                    "None",
                                    "JENEsys Supervisor",
                                    "JENEsys Edge",
                                    "Niagara 4 Supervisor",
                                    "Niagara 4 Edge 10",
                                    "Niagara 4 Edge 25",
                                    "Niagara 4 Edge 100",
                                    "Niagara 4 Edge Unlimited",
                                ] {
                                    ui.selectable_value(
                                        &mut obj.controller_license,
                                        lic.to_string(),
                                        lic,
                                    );
                                }
                            });
                    }

                    if obj.object_type == ObjectType::Equipment {
                        ui.separator();
                        ui.label(RichText::new("Equipment Data").strong());
                        ui.horizontal(|ui| {
                            ui.label("Equipment Type");
                            ui.text_edit_singleline(&mut obj.equipment_type);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Equipment Tag");
                            ui.text_edit_singleline(&mut obj.equipment_tag);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Make");
                            ui.text_edit_singleline(&mut obj.make);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Model");
                            ui.text_edit_singleline(&mut obj.model);
                        });

                        egui::ComboBox::from_label("Point Template")
                            .selected_text(if obj.template_name.is_empty() {
                                "Select template"
                            } else {
                                &obj.template_name
                            })
                            .show_ui(ui, |ui| {
                                for t in &self.project.templates {
                                    ui.selectable_value(
                                        &mut obj.template_name,
                                        t.name.clone(),
                                        t.name.as_str(),
                                    );
                                }
                            });
                        if ui.button("Generate Points from Template").clicked() {
                            apply_template = true;
                        }
                    }

                    ui.separator();
                    for group in &mut obj.property_groups {
                        egui::CollapsingHeader::new(group.name.as_str())
                            .default_open(true)
                            .show(ui, |ui| {
                                for item in &mut group.items {
                                    ui.horizontal(|ui| {
                                        ui.label(item.key.as_str());
                                        ui.text_edit_singleline(&mut item.value);
                                    });
                                }
                                if ui.button("+ Property").clicked() {
                                    group.items.push(PropertyItem {
                                        key: "New".to_string(),
                                        value: String::new(),
                                    });
                                }
                            });
                    }
                    if ui.button("+ Group").clicked() {
                        obj.property_groups.push(PropertyGroup {
                            name: format!("Group {}", obj.property_groups.len() + 1),
                            items: vec![],
                        });
                    }
                });

                if apply_template {
                    self.apply_template_to_selected_equipment();
                }
            }
        }
    }

    fn project_settings_view(&mut self, ui: &mut Ui) {
        ui.heading("Project Settings & Proposal Inputs");
        egui::ScrollArea::vertical().show(ui, |ui| {
            Self::card_frame().show(ui, |ui| {
                ui.label(RichText::new("Project Core").strong());
                ui.horizontal(|ui| {
                    ui.label("Project Name");
                    ui.text_edit_singleline(&mut self.project.name);
                });
                ui.horizontal(|ui| {
                    ui.label("Project Notes");
                    ui.text_edit_singleline(&mut self.project.notes);
                });
            });

            Self::card_frame().show(ui, |ui| {
                ui.label(RichText::new("Proposal Metadata").strong());
                let p = &mut self.project.proposal;
                ui.horizontal(|ui| {
                    ui.label("Client");
                    ui.text_edit_singleline(&mut p.client_name);
                });
                ui.horizontal(|ui| {
                    ui.label("Location");
                    ui.text_edit_singleline(&mut p.project_location);
                });
                ui.horizontal(|ui| {
                    ui.label("Proposal #");
                    ui.text_edit_singleline(&mut p.proposal_number);
                });
                ui.horizontal(|ui| {
                    ui.label("Revision");
                    ui.text_edit_singleline(&mut p.revision);
                });
                ui.horizontal(|ui| {
                    ui.label("Bid Date");
                    ui.text_edit_singleline(&mut p.bid_date);
                });
                ui.horizontal(|ui| {
                    ui.label("Prepared By");
                    ui.text_edit_singleline(&mut p.prepared_by);
                });
            });

            Self::card_frame().show(ui, |ui| {
                ui.label(RichText::new("Scope, Assumptions, Exclusions").strong());
                ui.label("Scope Summary");
                ui.text_edit_multiline(&mut self.project.proposal.scope_summary);
                ui.label("Assumptions");
                ui.text_edit_multiline(&mut self.project.proposal.assumptions);
                ui.label("Exclusions");
                ui.text_edit_multiline(&mut self.project.proposal.exclusions);
            });
        });
    }

    fn hours_estimator_view(&mut self, ui: &mut Ui) {
        ui.heading("Hours Estimator");

        let controllers = self
            .project
            .objects
            .iter()
            .filter(|o| o.object_type == ObjectType::Controller)
            .count() as f32;
        let equipment_count = self
            .project
            .objects
            .iter()
            .filter(|o| o.object_type == ObjectType::Equipment)
            .count() as f32;
        let points = self
            .project
            .objects
            .iter()
            .filter(|o| o.object_type == ObjectType::Point)
            .count() as f32;

        let mut eng = controllers * 7.0 + points * 0.25;
        let mut gfx = equipment_count * 1.0;
        let mut cx = controllers * 5.5 + points * 0.12;

        for eq in self
            .project
            .objects
            .iter()
            .filter(|o| o.object_type == ObjectType::Equipment)
        {
            if let Some(t) = self
                .project
                .templates
                .iter()
                .find(|t| t.name == eq.template_name)
            {
                eng += t.engineering_hours;
                gfx += t.graphics_hours;
                cx += t.commissioning_hours;
            }
        }

        let mut custom_total = 0.0;
        Self::card_frame().show(ui, |ui| {
            ui.label(RichText::new("System-derived hours").strong());
            egui::Grid::new("est_grid").show(ui, |ui| {
                ui.label("Engineering");
                ui.label(format!("{eng:.1} h"));
                ui.end_row();
                ui.label("Graphics/Submittals");
                ui.label(format!("{gfx:.1} h"));
                ui.end_row();
                ui.label("Commissioning");
                ui.label(format!("{cx:.1} h"));
                ui.end_row();
            });
        });

        Self::card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Custom hour lines").strong());
                if ui.button("+ Add line").clicked() {
                    self.project.custom_hour_lines.push(HourLine::default());
                }
            });

            let mut remove_idx = None;
            for (idx, line) in self.project.custom_hour_lines.iter_mut().enumerate() {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut line.name);
                    egui::ComboBox::from_id_source(("cat", idx))
                        .selected_text(&line.category)
                        .show_ui(ui, |ui| {
                            for c in ["Engineering", "Graphics", "Commissioning", "Other"] {
                                ui.selectable_value(&mut line.category, c.to_string(), c);
                            }
                        });
                    ui.add(
                        egui::DragValue::new(&mut line.quantity)
                            .speed(0.1)
                            .prefix("Qty "),
                    );
                    ui.add(
                        egui::DragValue::new(&mut line.hours_per_unit)
                            .speed(0.1)
                            .prefix("h/u "),
                    );
                    if ui.button("🗑").clicked() {
                        remove_idx = Some(idx);
                    }
                });
                custom_total += line.quantity.max(0.0) * line.hours_per_unit.max(0.0);
            }
            if let Some(idx) = remove_idx {
                self.project.custom_hour_lines.remove(idx);
            }
        });

        Self::card_frame().show(ui, |ui| {
            let total = eng + gfx + cx + custom_total;
            ui.label(RichText::new(format!("Total Estimated Hours: {total:.1} h")).strong());
            ui.small("No dollar estimates are shown by design.");
        });
    }

    fn templates_view(&mut self, ui: &mut Ui) {
        ui.heading("Template Tool");
        ui.label("Define typical equipment point lists and default hours per template.");
        if ui.button("+ New Template").clicked() {
            self.project.templates.push(EquipmentTemplate {
                name: format!("Template {}", self.project.templates.len() + 1),
                equipment_type: String::new(),
                points: vec!["New Point".to_string()],
                engineering_hours: 0.0,
                graphics_hours: 0.0,
                commissioning_hours: 0.0,
            });
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut remove_template = None;
            for (idx, template) in self.project.templates.iter_mut().enumerate() {
                egui::Frame::default()
                    .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 14))
                    .rounding(egui::Rounding::same(10.0))
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut template.name);
                            ui.label("Type");
                            ui.text_edit_singleline(&mut template.equipment_type);
                            if ui.button("Delete").clicked() {
                                remove_template = Some(idx);
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::DragValue::new(&mut template.engineering_hours)
                                    .speed(0.1)
                                    .prefix("Eng "),
                            );
                            ui.add(
                                egui::DragValue::new(&mut template.graphics_hours)
                                    .speed(0.1)
                                    .prefix("Graphics "),
                            );
                            ui.add(
                                egui::DragValue::new(&mut template.commissioning_hours)
                                    .speed(0.1)
                                    .prefix("Cx "),
                            );
                        });

                        ui.label(RichText::new("Point List").strong());
                        let mut remove_point = None;
                        for (pidx, point) in template.points.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(point);
                                if ui.button("x").clicked() {
                                    remove_point = Some(pidx);
                                }
                            });
                        }
                        if let Some(pidx) = remove_point {
                            template.points.remove(pidx);
                        }
                        if ui.button("+ Point").clicked() {
                            template.points.push("New Point".to_string());
                        }
                    });
                ui.add_space(8.0);
            }
            if let Some(idx) = remove_template {
                self.project.templates.remove(idx);
            }
        });
    }

    fn drawings_overlay_view(&mut self, ui: &mut Ui) {
        ui.heading("Drawings Overlay");
        ui.horizontal(|ui| {
            if ui.button("Load PDF").clicked() {
                if let Some(pdf) = FileDialog::new().add_filter("PDF", &["pdf"]).pick_file() {
                    self.project.overlay_pdf = Some(pdf.display().to_string());
                }
            }
            ui.label(
                self.project
                    .overlay_pdf
                    .clone()
                    .unwrap_or_else(|| "No PDF selected".to_string()),
            );
        });

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Controller token").drag_started() {
                self.dragging_palette = Some(ObjectType::Controller);
            }
            if ui.button("Equipment token").drag_started() {
                self.dragging_palette = Some(ObjectType::Equipment);
            }
        });

        let desired = egui::vec2(ui.available_width(), ui.available_height() - 16.0);
        let (resp, painter) = ui.allocate_painter(desired, egui::Sense::click_and_drag());
        painter.rect_filled(
            resp.rect,
            10.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 16),
        );
        painter.rect_stroke(resp.rect, 10.0, egui::Stroke::new(1.0, self.accent()));

        for line in &self.project.overlay_lines {
            let a = egui::pos2(
                resp.rect.left() + line.from[0],
                resp.rect.top() + line.from[1],
            );
            let b = egui::pos2(resp.rect.left() + line.to[0], resp.rect.top() + line.to[1]);
            painter.line_segment([a, b], egui::Stroke::new(2.0, self.accent()));
        }

        if resp.hovered() {
            if let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) {
                if ui.input(|i| i.pointer.any_released()) {
                    if let Some(kind) = self.dragging_palette.take() {
                        if let Some(object_id) = self
                            .project
                            .objects
                            .iter()
                            .find(|o| o.object_type == kind)
                            .map(|o| o.id)
                        {
                            self.project.overlay_nodes.push(OverlayNode {
                                id: self.project.next_id,
                                object_id,
                                x: pointer.x - resp.rect.left(),
                                y: pointer.y - resp.rect.top(),
                            });
                            self.project.next_id += 1;
                        }
                    } else if resp.clicked() {
                        let local = [pointer.x - resp.rect.left(), pointer.y - resp.rect.top()];
                        if let Some(start) = self.active_line_start.take() {
                            self.project.overlay_lines.push(OverlayLine {
                                from: start,
                                to: local,
                            });
                        } else {
                            self.active_line_start = Some(local);
                        }
                    }
                }
            }
        }
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        if self.show_about {
            egui::Window::new("About")
                .open(&mut self.show_about)
                .show(ctx, |ui| {
                    ui.label("AutoMate BAS Studio");
                    ui.label("Data-first takeoff, estimating, and proposal workflow.");
                });
        }

        if self.show_software_settings {
            egui::Window::new("Software Settings")
                .open(&mut self.show_software_settings)
                .show(ctx, |ui| {
                    ui.label("Accent Color");
                    ui.color_edit_button_srgba_unmultiplied(
                        &mut self.project.settings.accent_color,
                    );
                    ui.horizontal(|ui| {
                        ui.label("Company Name");
                        ui.text_edit_singleline(&mut self.project.settings.company_name);
                    });
                });
        }
    }

    fn object_counts(&self) -> BTreeMap<ObjectType, usize> {
        let mut map = BTreeMap::new();
        for obj in &self.project.objects {
            *map.entry(obj.object_type).or_insert(0) += 1;
        }
        map
    }
}

impl App for AutoMateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.draw_studio_background(ctx);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.visuals.window_fill = Color32::from_rgb(27, 30, 35);
        style.visuals.panel_fill = Color32::from_rgb(27, 30, 35);
        style.visuals.widgets.noninteractive.bg_fill =
            Color32::from_rgba_unmultiplied(255, 255, 255, 8);
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgba_unmultiplied(255, 255, 255, 12);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgba_unmultiplied(
            self.accent().r(),
            self.accent().g(),
            self.accent().b(),
            100,
        );
        style.visuals.widgets.active.bg_fill = self.accent();
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgba_unmultiplied(
            self.accent().r(),
            self.accent().g(),
            self.accent().b(),
            120,
        );
        style.visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(
            self.accent().r(),
            self.accent().g(),
            self.accent().b(),
            128,
        );
        ctx.set_style(style);

        self.titlebar(ctx);
        egui::TopBottomPanel::top("toolbar")
            .frame(Self::surface_panel())
            .show(ctx, |ui| self.toolbar_dropdowns(ui));

        egui::TopBottomPanel::bottom("status")
            .frame(Self::surface_panel())
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(self.status.as_str());
                    for (kind, count) in self.object_counts() {
                        ui.label(format!("{} {}", kind.icon(), count));
                    }
                });
            });

        egui::SidePanel::left("objects")
            .resizable(true)
            .default_width(330.0)
            .frame(Self::surface_panel())
            .show(ctx, |ui| self.left_sidebar(ui));

        egui::SidePanel::right("properties")
            .resizable(true)
            .default_width(360.0)
            .frame(Self::surface_panel())
            .show(ctx, |ui| self.right_properties(ui));

        egui::CentralPanel::default()
            .frame(Self::surface_panel().inner_margin(egui::Margin::same(18.0)))
            .show(ctx, |ui| {
                self.workspace_header(ui);
                ui.separator();
                match self.current_view {
                    ToolView::ProjectSettings => self.project_settings_view(ui),
                    ToolView::HoursEstimator => self.hours_estimator_view(ui),
                    ToolView::DrawingsOverlay => self.drawings_overlay_view(ui),
                    ToolView::Templates => self.templates_view(ui),
                }
            });

        self.dialogs(ctx);
        ctx.request_repaint();
    }
}
