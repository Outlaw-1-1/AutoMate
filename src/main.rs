use eframe::{
    egui::{self, menu, Align2, Color32, FontFamily, FontId, RichText, Sense, TextureHandle, Ui},
    epaint::{Mesh, Shadow, Vertex},
    App, CreationContext, Frame, NativeOptions,
};
use pdfium_render::prelude::*;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    time::Instant,
};
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(false)
            .with_decorations(false)
            .with_resizable(true)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppScreen {
    Splash,
    Login,
    Studio,
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
    autosave_minutes: u32,
    ui_scale: f32,
    show_overlay_grid: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            accent_color: [168, 196, 84, 255],
            company_name: "AutoMate Controls".to_string(),
            autosave_minutes: 10,
            ui_scale: 1.0,
            show_overlay_grid: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ProposalData {
    project_number: String,
    client_name: String,
    owner: String,
    engineer_of_record: String,
    project_location: String,
    proposal_number: String,
    revision: String,
    contract_type: String,
    design_stage: String,
    bid_date: String,
    target_start_date: String,
    target_completion_date: String,
    prepared_by: String,
    project_manager: String,
    estimator: String,
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
struct EstimatorSettings {
    complexity_factor: f32,
    renovation_factor: f32,
    integration_factor: f32,
    qa_percent: f32,
    project_management_percent: f32,
    risk_percent: f32,
}

impl Default for EstimatorSettings {
    fn default() -> Self {
        Self {
            complexity_factor: 1.0,
            renovation_factor: 1.0,
            integration_factor: 1.0,
            qa_percent: 8.0,
            project_management_percent: 12.0,
            risk_percent: 5.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
enum HourCalculationMode {
    StaticByEquipment,
    PointsBased,
}

impl Default for HourCalculationMode {
    fn default() -> Self {
        Self::StaticByEquipment
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EquipmentTemplate {
    name: String,
    equipment_type: String,
    points: Vec<String>,
    hour_mode: HourCalculationMode,
    engineering_hours: f32,
    engineering_hours_per_point: f32,
    graphics_hours: f32,
    graphics_hours_per_point: f32,
    commissioning_hours: f32,
    commissioning_hours_per_point: f32,
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
            hour_mode: HourCalculationMode::StaticByEquipment,
            engineering_hours: 2.0,
            engineering_hours_per_point: 0.25,
            graphics_hours: 1.0,
            graphics_hours_per_point: 0.12,
            commissioning_hours: 1.5,
            commissioning_hours_per_point: 0.18,
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
    #[serde(default)]
    estimator: EstimatorSettings,
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
            property_groups: vec![],
        };

        Self {
            name: "New BAS Project".to_string(),
            notes: "Capture assumptions, scope notes, and exclusions here.".to_string(),
            proposal: ProposalData::default(),
            objects: vec![building],
            overlay_pdf: None,
            overlay_nodes: vec![],
            overlay_lines: vec![],
            templates: vec![
                EquipmentTemplate::default(),
                EquipmentTemplate {
                    name: "AHU Typical".to_string(),
                    equipment_type: "AHU".to_string(),
                    points: vec![
                        "Space Temp".to_string(),
                        "Supply Temp".to_string(),
                        "Return Temp".to_string(),
                        "Static Pressure".to_string(),
                        "Fan Cmd".to_string(),
                        "Filter DP".to_string(),
                    ],
                    hour_mode: HourCalculationMode::StaticByEquipment,
                    engineering_hours: 5.0,
                    engineering_hours_per_point: 0.3,
                    graphics_hours: 2.0,
                    graphics_hours_per_point: 0.15,
                    commissioning_hours: 3.0,
                    commissioning_hours_per_point: 0.2,
                },
                EquipmentTemplate {
                    name: "Boiler Plant".to_string(),
                    equipment_type: "Boiler".to_string(),
                    points: vec![
                        "Space Temp".to_string(),
                        "Enable".to_string(),
                        "Water Temp".to_string(),
                        "Status".to_string(),
                        "Alarm".to_string(),
                    ],
                    hour_mode: HourCalculationMode::StaticByEquipment,
                    engineering_hours: 4.0,
                    engineering_hours_per_point: 0.3,
                    graphics_hours: 1.5,
                    graphics_hours_per_point: 0.15,
                    commissioning_hours: 2.5,
                    commissioning_hours_per_point: 0.2,
                },
                EquipmentTemplate {
                    name: "Chiller".to_string(),
                    equipment_type: "Chiller".to_string(),
                    points: vec![
                        "Space Temp".to_string(),
                        "CHWS Temp".to_string(),
                        "CHWR Temp".to_string(),
                        "Run Cmd".to_string(),
                        "kW".to_string(),
                        "Fault".to_string(),
                    ],
                    hour_mode: HourCalculationMode::StaticByEquipment,
                    engineering_hours: 5.0,
                    engineering_hours_per_point: 0.3,
                    graphics_hours: 2.0,
                    graphics_hours_per_point: 0.15,
                    commissioning_hours: 3.0,
                    commissioning_hours_per_point: 0.2,
                },
            ],
            custom_hour_lines: vec![],
            estimator: EstimatorSettings::default(),
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
    app_screen: AppScreen,
    splash_started_at: Instant,
    login_username: String,
    login_password: String,
    login_error: Option<String>,
    overview_image_bytes: Option<Vec<u8>>,
    overview_texture: Option<TextureHandle>,
    overlay_pdf_bytes: Option<Vec<u8>>,
    overlay_texture: Option<TextureHandle>,
    last_autosave_at: Instant,
    overlay_undo_stack: Vec<(Vec<OverlayNode>, Vec<OverlayLine>)>,
    overlay_redo_stack: Vec<(Vec<OverlayNode>, Vec<OverlayLine>)>,
    pending_overlay_drop: Option<(ObjectType, [f32; 2])>,
    show_adjustment_popup: bool,
    left_sidebar_collapsed: bool,
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
            app_screen: AppScreen::Splash,
            splash_started_at: Instant::now(),
            login_username: String::new(),
            login_password: String::new(),
            login_error: None,
            overview_image_bytes: None,
            overview_texture: None,
            overlay_pdf_bytes: None,
            overlay_texture: None,
            last_autosave_at: Instant::now(),
            overlay_undo_stack: vec![],
            overlay_redo_stack: vec![],
            pending_overlay_drop: None,
            show_adjustment_popup: false,
            left_sidebar_collapsed: false,
        }
    }

    fn estimate_hours(&self) -> (f32, f32, f32, f32, f32, f32) {
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
                let eq_points = self
                    .project
                    .objects
                    .iter()
                    .filter(|o| o.parent_id == Some(eq.id) && o.object_type == ObjectType::Point)
                    .count() as f32;
                match t.hour_mode {
                    HourCalculationMode::StaticByEquipment => {
                        eng += t.engineering_hours;
                        gfx += t.graphics_hours;
                        cx += t.commissioning_hours;
                    }
                    HourCalculationMode::PointsBased => {
                        eng += eq_points * t.engineering_hours_per_point;
                        gfx += eq_points * t.graphics_hours_per_point;
                        cx += eq_points * t.commissioning_hours_per_point;
                    }
                }
            }
        }

        let custom_total = self
            .project
            .custom_hour_lines
            .iter()
            .map(|line| line.quantity.max(0.0) * line.hours_per_unit.max(0.0))
            .sum::<f32>();

        let base = eng + gfx + cx + custom_total;
        let factors = self.project.estimator.complexity_factor
            * self.project.estimator.renovation_factor
            * self.project.estimator.integration_factor;
        let adjusted = base * factors;
        let overhead_pct = (self.project.estimator.qa_percent
            + self.project.estimator.project_management_percent
            + self.project.estimator.risk_percent)
            .max(0.0);
        let overhead_hours = adjusted * (overhead_pct / 100.0);
        let grand_total = adjusted + overhead_hours;

        (eng, gfx, cx, custom_total, overhead_hours, grand_total)
    }

    fn apply_recommended_settings(&mut self) {
        self.project.settings.autosave_minutes = self.project.settings.autosave_minutes.min(15);
        self.project.settings.ui_scale = self.project.settings.ui_scale.clamp(0.95, 1.25);
        if self.project.settings.company_name.trim().is_empty() {
            self.project.settings.company_name = "AutoMate Controls".to_string();
        }

        self.project.estimator.complexity_factor =
            self.project.estimator.complexity_factor.clamp(0.8, 1.4);
        self.project.estimator.renovation_factor =
            self.project.estimator.renovation_factor.clamp(1.0, 1.35);
        self.project.estimator.integration_factor =
            self.project.estimator.integration_factor.clamp(0.9, 1.3);
        self.project.estimator.qa_percent = self.project.estimator.qa_percent.clamp(5.0, 12.0);
        self.project.estimator.project_management_percent = self
            .project
            .estimator
            .project_management_percent
            .clamp(8.0, 16.0);
        self.project.estimator.risk_percent = self.project.estimator.risk_percent.clamp(3.0, 12.0);
    }

    fn accent(&self) -> Color32 {
        let [r, g, b, a] = self.project.settings.accent_color;
        Color32::from_rgba_unmultiplied(r, g, b, a)
    }

    fn surface_panel() -> egui::Frame {
        egui::Frame::default()
            .fill(Color32::from_rgba_unmultiplied(18, 23, 34, 236))
            .stroke(egui::Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 20),
            ))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(14.0))
            .outer_margin(egui::Margin::same(2.0))
            .shadow(Shadow {
                offset: egui::vec2(0.0, 6.0),
                blur: 24.0,
                spread: 0.0,
                color: Color32::from_rgba_unmultiplied(0, 0, 0, 130),
            })
    }

    fn card_frame() -> egui::Frame {
        egui::Frame::default()
            .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 7))
            .stroke(egui::Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 20),
            ))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(8.0))
    }

    fn draw_studio_background(&self, ctx: &egui::Context) {
        let rect = ctx.screen_rect();
        let accent = self.accent();
        let top = Color32::from_rgba_unmultiplied(15, 20, 31, 255);
        let bottom = Color32::from_rgba_unmultiplied(10, 13, 21, 255);

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

        let painter = ctx.layer_painter(egui::LayerId::background());
        let primary = egui::pos2(rect.right() - 240.0, rect.top() + 220.0);
        painter.text(
            primary,
            Align2::CENTER_CENTER,
            "M8",
            FontId::proportional(280.0),
            Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 18),
        );
        let secondary = egui::pos2(rect.left() + 200.0, rect.bottom() - 160.0);
        painter.text(
            secondary,
            Align2::CENTER_CENTER,
            "M8",
            FontId::proportional(220.0),
            Color32::from_rgba_unmultiplied(76, 129, 255, 14),
        );
    }

    fn card_frame_with_alpha(alpha: u8) -> egui::Frame {
        egui::Frame::default()
            .fill(Color32::from_rgba_unmultiplied(255, 255, 255, alpha))
            .stroke(egui::Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 30),
            ))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(8.0))
    }

    fn draw_mark(&self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("M8")
                    .family(FontFamily::Proportional)
                    .size(84.0)
                    .strong()
                    .color(Color32::from_rgba_unmultiplied(
                        self.accent().r(),
                        self.accent().g(),
                        self.accent().b(),
                        150,
                    )),
            );
        });
    }

    fn splash_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::TRANSPARENT))
            .show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    self.draw_mark(ui);
                });
            });

        if self.splash_started_at.elapsed().as_secs_f32() > 2.2 {
            self.app_screen = AppScreen::Login;
        }
    }

    fn login_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let max_h = ui.available_height() * 0.30;
            ui.vertical_centered(|ui| {
                ui.add_space((ui.available_height() - max_h).max(0.0) * 0.45);
                Self::surface_panel().show(ui, |ui| {
                    ui.set_width(760.0);
                    ui.set_height(max_h);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            self.draw_mark(ui);
                            ui.label(
                                RichText::new("Technical Application Login")
                                    .size(18.0)
                                    .strong(),
                            );
                            ui.label(
                                RichText::new(
                                    "Secure sign-in for BAS estimating, drawings, and controls engineering.",
                                )
                                .size(13.0)
                                .color(Color32::from_gray(190)),
                            );
                        });

                        ui.separator();

                        ui.vertical(|ui| {
                            ui.set_min_width(340.0);
                            Self::card_frame_with_alpha(18).show(ui, |ui| {
                                ui.label(RichText::new("Operator ID").strong());
                                ui.text_edit_singleline(&mut self.login_username);
                                ui.label(RichText::new("Passphrase").strong());
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.login_password)
                                        .password(true),
                                );
                                ui.add_space(8.0);
                                if ui
                                    .add_sized(
                                        [ui.available_width(), 30.0],
                                        egui::Button::new(RichText::new("Authenticate").strong()),
                                    )
                                    .clicked()
                                {
                                    if self.login_username.trim().is_empty()
                                        || self.login_password.trim().is_empty()
                                    {
                                        self.login_error =
                                            Some("Enter operator ID and passphrase.".to_string());
                                    } else {
                                        self.login_error = None;
                                        self.status = format!(
                                            "Authenticated as {}",
                                            self.login_username.trim()
                                        );
                                        self.app_screen = AppScreen::Studio;
                                    }
                                }
                                if let Some(err) = &self.login_error {
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(err)
                                            .color(Color32::from_rgb(255, 130, 130))
                                            .size(12.0),
                                    );
                                }
                            });
                        });
                    });
                });
            });
        });
    }

    fn obfuscate(buffer: &mut [u8]) {
        for byte in buffer {
            *byte ^= 0xA5;
        }
    }

    fn sanitize_asset_name(path: &Path) -> String {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.replace(' ', "_"))
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "asset.bin".to_string())
    }

    fn local_pdf_path() -> Option<PathBuf> {
        std::env::var("AUTOMATE_PDFIUM_LIB")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                let mut candidate = std::env::current_exe().ok()?;
                candidate.pop();
                let file = if cfg!(target_os = "windows") {
                    "pdfium.dll"
                } else if cfg!(target_os = "macos") {
                    "libpdfium.dylib"
                } else {
                    "libpdfium.so"
                };
                candidate.push(file);
                candidate.exists().then_some(candidate)
            })
    }

    fn to_template(line: &HourLine) -> EquipmentTemplate {
        let name = line.name.trim();
        EquipmentTemplate {
            name: if name.is_empty() {
                "Custom Hour Template".to_string()
            } else {
                format!("{} (Hours)", name)
            },
            equipment_type: "Custom".to_string(),
            points: vec!["Space Temp".to_string(), "Template Point".to_string()],
            hour_mode: HourCalculationMode::StaticByEquipment,
            engineering_hours: if line.category == "Engineering" {
                line.quantity.max(0.0) * line.hours_per_unit.max(0.0)
            } else {
                0.0
            },
            engineering_hours_per_point: 0.0,
            graphics_hours: if line.category == "Graphics" {
                line.quantity.max(0.0) * line.hours_per_unit.max(0.0)
            } else {
                0.0
            },
            graphics_hours_per_point: 0.0,
            commissioning_hours: if line.category == "Commissioning" {
                line.quantity.max(0.0) * line.hours_per_unit.max(0.0)
            } else {
                0.0
            },
            commissioning_hours_per_point: 0.0,
        }
    }

    fn template_seed_data() -> Vec<EquipmentTemplate> {
        vec![
            EquipmentTemplate::default(),
            EquipmentTemplate {
                name: "AHU Typical".to_string(),
                equipment_type: "AHU".to_string(),
                points: vec![
                    "Space Temp".to_string(),
                    "Supply Temp".into(),
                    "Return Temp".into(),
                    "Static Pressure".into(),
                    "Fan Cmd".into(),
                    "Filter DP".into(),
                ],
                hour_mode: HourCalculationMode::StaticByEquipment,
                engineering_hours: 5.0,
                engineering_hours_per_point: 0.3,
                graphics_hours: 2.0,
                graphics_hours_per_point: 0.15,
                commissioning_hours: 3.0,
                commissioning_hours_per_point: 0.2,
            },
            EquipmentTemplate {
                name: "Boiler Plant".to_string(),
                equipment_type: "Boiler".to_string(),
                points: vec![
                    "Space Temp".to_string(),
                    "Enable".into(),
                    "Water Temp".into(),
                    "Status".into(),
                    "Alarm".into(),
                ],
                hour_mode: HourCalculationMode::StaticByEquipment,
                engineering_hours: 4.0,
                engineering_hours_per_point: 0.3,
                graphics_hours: 1.5,
                graphics_hours_per_point: 0.15,
                commissioning_hours: 2.5,
                commissioning_hours_per_point: 0.2,
            },
            EquipmentTemplate {
                name: "Chiller".to_string(),
                equipment_type: "Chiller".to_string(),
                points: vec![
                    "Space Temp".to_string(),
                    "CHWS Temp".into(),
                    "CHWR Temp".into(),
                    "Run Cmd".into(),
                    "kW".into(),
                    "Fault".into(),
                ],
                hour_mode: HourCalculationMode::StaticByEquipment,
                engineering_hours: 5.0,
                engineering_hours_per_point: 0.3,
                graphics_hours: 2.0,
                graphics_hours_per_point: 0.15,
                commissioning_hours: 3.0,
                commissioning_hours_per_point: 0.2,
            },
            EquipmentTemplate {
                name: "Fan Coil Unit".to_string(),
                equipment_type: "FCU".to_string(),
                points: vec![
                    "Space Temp".to_string(),
                    "Room Temp".into(),
                    "Fan Speed".into(),
                    "Valve Cmd".into(),
                    "Occupancy".into(),
                ],
                hour_mode: HourCalculationMode::StaticByEquipment,
                engineering_hours: 2.5,
                engineering_hours_per_point: 0.25,
                graphics_hours: 1.0,
                graphics_hours_per_point: 0.12,
                commissioning_hours: 1.5,
                commissioning_hours_per_point: 0.18,
            },
        ]
    }

    fn ensure_template_seeded(&mut self) {
        if self.project.templates.is_empty() {
            self.project.templates = Self::template_seed_data();
        }

        let mut names = BTreeSet::new();
        self.project
            .templates
            .retain(|t| names.insert(t.name.clone()));

        for line in &self.project.custom_hour_lines {
            let candidate = Self::to_template(line);
            if let Some(existing) = self
                .project
                .templates
                .iter_mut()
                .find(|t| t.name == candidate.name)
            {
                *existing = candidate;
            } else {
                self.project.templates.push(candidate);
            }
        }
    }

    fn refresh_overview_texture(&mut self, ctx: &egui::Context) {
        let Some(bytes) = &self.overview_image_bytes else {
            self.overview_texture = None;
            return;
        };
        if let Ok(img) = image::load_from_memory(bytes) {
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
            self.overview_texture =
                Some(ctx.load_texture("overview_image", color_image, egui::TextureOptions::LINEAR));
        }
    }

    fn refresh_overlay_texture(&mut self, ctx: &egui::Context, target_width: u16) {
        let Some(bytes) = &self.overlay_pdf_bytes else {
            self.overlay_texture = None;
            return;
        };

        let bindings = match Self::local_pdf_path() {
            Some(path) => Pdfium::bind_to_library(path).map_err(|err| err.to_string()),
            None => Err("local PDFium binary not found. Place PDFium next to the app or set AUTOMATE_PDFIUM_LIB.".to_string()),
        };
        let bindings = match bindings {
            Ok(bindings) => bindings,
            Err(err) => {
                self.status = format!("PDF renderer unavailable ({err})");
                self.overlay_texture = None;
                return;
            }
        };

        let pdfium = Pdfium::new(bindings);
        let document = match pdfium.load_pdf_from_byte_vec(bytes.clone(), None) {
            Ok(doc) => doc,
            Err(err) => {
                self.status = format!("PDF load failed: {err}");
                self.overlay_texture = None;
                return;
            }
        };

        let page = match document.pages().get(0) {
            Ok(page) => page,
            Err(err) => {
                self.status = format!("PDF page read failed: {err}");
                self.overlay_texture = None;
                return;
            }
        };

        let render = match page.render_with_config(
            &PdfRenderConfig::new()
                .set_target_width(target_width.max(400) as i32)
                .render_form_data(true),
        ) {
            Ok(render) => render,
            Err(err) => {
                self.status = format!("PDF render failed: {err}");
                self.overlay_texture = None;
                return;
            }
        };

        let image = render.as_image();
        let rgba = image.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
        self.overlay_texture = Some(ctx.load_texture(
            "overlay_pdf_page",
            color_image,
            egui::TextureOptions::LINEAR,
        ));
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
            ui.set_width(ui.available_width());
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
        if let Some(parent_id) = parent {
            let parent_obj = self.project.objects.iter().find(|o| o.id == parent_id);
            let is_valid_parent = matches!(
                (object_type, parent_obj.map(|o| o.object_type)),
                (ObjectType::Controller, Some(ObjectType::Building))
                    | (ObjectType::Equipment, Some(ObjectType::Controller))
                    | (ObjectType::Point, Some(ObjectType::Equipment))
            );

            if !is_valid_parent {
                self.status = format!("Cannot add {} to selected parent", object_type.label());
                return;
            }
        }

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
            property_groups: if object_type == ObjectType::Equipment {
                vec![PropertyGroup {
                    name: "General".to_string(),
                    items: vec![],
                }]
            } else {
                vec![]
            },
        });
        self.selected_object = Some(id);
    }

    fn remove_object_subtree(&mut self, id: u64) {
        let mut to_remove = BTreeSet::new();
        let mut stack = vec![id];

        while let Some(current) = stack.pop() {
            if !to_remove.insert(current) {
                continue;
            }
            for child in self
                .project
                .objects
                .iter()
                .filter(|obj| obj.parent_id == Some(current))
            {
                stack.push(child.id);
            }
        }

        if to_remove.is_empty() {
            return;
        }

        self.project
            .objects
            .retain(|obj| !to_remove.contains(&obj.id));
        self.project
            .overlay_nodes
            .retain(|node| !to_remove.contains(&node.object_id));

        if self
            .selected_object
            .is_some_and(|selected| to_remove.contains(&selected))
        {
            self.selected_object = self.project.objects.first().map(|o| o.id);
        }

        self.status = "Object deleted".to_string();
    }

    fn normalize_loaded_project(&mut self) {
        let valid_ids: BTreeSet<u64> = self.project.objects.iter().map(|o| o.id).collect();

        self.project.objects.retain(|obj| {
            obj.parent_id
                .is_none_or(|parent| valid_ids.contains(&parent))
        });

        let valid_ids: BTreeSet<u64> = self.project.objects.iter().map(|o| o.id).collect();
        self.project
            .overlay_nodes
            .retain(|node| valid_ids.contains(&node.object_id));

        let max_id = self.project.objects.iter().map(|o| o.id).max().unwrap_or(0);
        self.project.next_id = self.project.next_id.max(max_id + 1);

        if self
            .selected_object
            .is_some_and(|selected| !valid_ids.contains(&selected))
        {
            self.selected_object = self.project.objects.first().map(|o| o.id);
        }
    }

    fn save_project_to_path(&mut self, path: &Path) -> Result<(), String> {
        match serde_json::to_vec_pretty(&self.project) {
            Ok(project_payload) => {
                let mut archive_data = Vec::new();
                let mut zip = ZipWriter::new(Cursor::new(&mut archive_data));
                let options = SimpleFileOptions::default();

                if zip.start_file("project.json", options).is_err()
                    || zip.write_all(&project_payload).is_err()
                {
                    return Err("Save failed: unable to write project.json".to_string());
                }

                if let (Some(name), Some(bytes)) =
                    (&self.project.overview_image, &self.overview_image_bytes)
                {
                    if zip.start_file(format!("assets/{name}"), options).is_ok() {
                        let _ = zip.write_all(bytes);
                    }
                }

                if let (Some(name), Some(bytes)) =
                    (&self.project.overlay_pdf, &self.overlay_pdf_bytes)
                {
                    if zip.start_file(format!("assets/{name}"), options).is_ok() {
                        let _ = zip.write_all(bytes);
                    }
                }

                if zip.finish().is_err() {
                    return Err("Save failed: unable to finish archive".to_string());
                }

                Self::obfuscate(&mut archive_data);
                fs::write(path, archive_data).map_err(|e| format!("Save failed: {e}"))?;
                self.project_path = Some(path.to_path_buf());
                self.last_autosave_at = Instant::now();
                Ok(())
            }
            Err(e) => Err(format!("Serialization failed: {e}")),
        }
    }

    fn save_project(&mut self) {
        let path = self.project_path.clone().or_else(|| {
            FileDialog::new()
                .add_filter("M8 Project", &["m8"])
                .set_file_name("project.m8")
                .save_file()
        });
        if let Some(path) = path {
            match self.save_project_to_path(&path) {
                Ok(_) => self.status = format!("Saved {}", path.display()),
                Err(e) => self.status = e,
            }
        }
    }

    fn autosave_project(&mut self) {
        let interval = self.project.settings.autosave_minutes.max(1) as u64 * 60;
        if self.last_autosave_at.elapsed().as_secs() < interval {
            return;
        }
        let Some(path) = self.project_path.clone() else {
            self.last_autosave_at = Instant::now();
            return;
        };
        match self.save_project_to_path(&path) {
            Ok(_) => self.status = format!("Autosaved {}", path.display()),
            Err(e) => self.status = format!("Autosave failed: {e}"),
        }
    }

    fn export_proposal_markdown(&mut self) {
        let Some(path) = FileDialog::new()
            .add_filter("Markdown", &["md"])
            .set_file_name("proposal-summary.md")
            .save_file()
        else {
            return;
        };
        let p = &self.project.proposal;
        let (eng, gfx, cx, custom, overhead, total) = self.estimate_hours();
        let body = format!(
            "# Proposal Summary\n\nProject: {}\n\n## Metadata\n- Client: {}\n- Location: {}\n- Proposal #: {}\n- Revision: {}\n- Bid Date: {}\n- Prepared By: {}\n\n## Scope\n{}\n\n## Assumptions\n{}\n\n## Exclusions\n{}\n\n## Estimated Hours\n- Engineering: {:.1} h\n- Graphics/Submittals: {:.1} h\n- Commissioning: {:.1} h\n- Custom Lines: {:.1} h\n- Overhead / Risk: {:.1} h\n- **Total: {:.1} h**\n",
            self.project.name,
            p.client_name,
            p.project_location,
            p.proposal_number,
            p.revision,
            p.bid_date,
            p.prepared_by,
            p.scope_summary,
            p.assumptions,
            p.exclusions,
            eng,
            gfx,
            cx,
            custom,
            overhead,
            total
        );

        match fs::write(&path, body) {
            Ok(_) => self.status = format!("Exported proposal {}", path.display()),
            Err(e) => self.status = format!("Proposal export failed: {e}"),
        }
    }

    fn push_overlay_history(&mut self) {
        self.overlay_undo_stack.push((
            self.project.overlay_nodes.clone(),
            self.project.overlay_lines.clone(),
        ));
        if self.overlay_undo_stack.len() > 50 {
            self.overlay_undo_stack.remove(0);
        }
        self.overlay_redo_stack.clear();
    }

    fn overlay_undo(&mut self) {
        if let Some((nodes, lines)) = self.overlay_undo_stack.pop() {
            self.overlay_redo_stack.push((
                self.project.overlay_nodes.clone(),
                self.project.overlay_lines.clone(),
            ));
            self.project.overlay_nodes = nodes;
            self.project.overlay_lines = lines;
            self.active_line_start = None;
            self.status = "Overlay undo applied".to_string();
        }
    }

    fn overlay_redo(&mut self) {
        if let Some((nodes, lines)) = self.overlay_redo_stack.pop() {
            self.overlay_undo_stack.push((
                self.project.overlay_nodes.clone(),
                self.project.overlay_lines.clone(),
            ));
            self.project.overlay_nodes = nodes;
            self.project.overlay_lines = lines;
            self.active_line_start = None;
            self.status = "Overlay redo applied".to_string();
        }
    }

    fn load_project(&mut self, ctx: &egui::Context) {
        if let Some(path) = FileDialog::new()
            .add_filter("M8 Project", &["m8"])
            .pick_file()
        {
            match fs::read(&path) {
                Ok(mut content) => {
                    Self::obfuscate(&mut content);
                    let reader = Cursor::new(content);
                    match ZipArchive::new(reader) {
                        Ok(mut archive) => {
                            let mut project_json = String::new();
                            if let Ok(mut file) = archive.by_name("project.json") {
                                let _ = file.read_to_string(&mut project_json);
                            }

                            match serde_json::from_str::<Project>(&project_json) {
                                Ok(project) => {
                                    self.project = project;
                                    self.overview_image_bytes = None;
                                    self.overlay_pdf_bytes = None;
                                    self.overview_texture = None;
                                    self.overlay_texture = None;

                                    if let Some(name) = &self.project.overview_image {
                                        if let Ok(mut file) =
                                            archive.by_name(&format!("assets/{name}"))
                                        {
                                            let mut bytes = Vec::new();
                                            let _ = file.read_to_end(&mut bytes);
                                            self.overview_image_bytes = Some(bytes);
                                            self.refresh_overview_texture(ctx);
                                        }
                                    }

                                    if let Some(name) = &self.project.overlay_pdf {
                                        if let Ok(mut file) =
                                            archive.by_name(&format!("assets/{name}"))
                                        {
                                            let mut bytes = Vec::new();
                                            let _ = file.read_to_end(&mut bytes);
                                            self.overlay_pdf_bytes = Some(bytes);
                                        }
                                    }

                                    self.project_path = Some(path.clone());
                                    self.normalize_loaded_project();
                                    self.status = format!("Loaded {}", path.display());
                                    self.selected_object =
                                        self.project.objects.first().map(|o| o.id);
                                    self.last_autosave_at = Instant::now();
                                    self.overlay_undo_stack.clear();
                                    self.overlay_redo_stack.clear();
                                    self.pending_overlay_drop = None;
                                }
                                Err(e) => self.status = format!("Parse failed: {e}"),
                            }
                        }
                        Err(e) => self.status = format!("Load failed: {e}"),
                    }
                }
                Err(e) => self.status = format!("Load failed: {e}"),
            }
        }
    }

    fn titlebar(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("titlebar")
            .frame(Self::surface_panel())
            .show(ctx, |ui| {
                let title_rect = ui.max_rect();
                let drag = ui.interact(title_rect, ui.id().with("titlebar_drag"), Sense::drag());
                if drag.drag_started() || drag.dragged() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
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
                        if ui.add_sized([28.0, 22.0], egui::Button::new("✕")).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui
                            .add_sized(
                                [28.0, 22.0],
                                egui::Button::new(if self.is_fullscreen { "🗗" } else { "🗖" }),
                            )
                            .clicked()
                        {
                            self.is_fullscreen = !self.is_fullscreen;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                                self.is_fullscreen,
                            ));
                        }
                        if ui.add_sized([28.0, 22.0], egui::Button::new("—")).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                    });
                });
            });
    }

    fn toolbar_dropdowns(&mut self, ui: &mut Ui) {
        menu::bar(ui, |ui| {
            ui.menu_button("📂 Project", |ui| {
                if ui.button("New").clicked() {
                    self.project = Project::default();
                    self.selected_object = Some(1);
                    self.project_path = None;
                    self.overview_image_bytes = None;
                    self.overview_texture = None;
                    self.overlay_pdf_bytes = None;
                    self.overlay_texture = None;
                    ui.close_menu();
                }
                if ui.button("Save").clicked() {
                    self.save_project();
                    ui.close_menu();
                }
                if ui.button("Load").clicked() {
                    self.load_project(ui.ctx());
                    ui.close_menu();
                }
                if ui.button("Export Proposal (Markdown)").clicked() {
                    self.export_proposal_markdown();
                    ui.close_menu();
                }
            });
            ui.menu_button("⚙ Settings", |ui| {
                if ui.button("Open Settings").clicked() {
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

    fn labeled_singleline(ui: &mut Ui, label: &str, value: &mut String) {
        ui.horizontal(|ui| {
            ui.set_width(ui.available_width());
            ui.label(label);
            ui.add_sized(
                [ui.available_width(), 24.0],
                egui::TextEdit::singleline(value),
            );
        });
    }

    fn sync_template_points(&mut self) {
        let equipment: Vec<(u64, String)> = self
            .project
            .objects
            .iter()
            .filter(|o| o.object_type == ObjectType::Equipment && !o.template_name.is_empty())
            .map(|o| (o.id, o.template_name.clone()))
            .collect();
        for (eq_id, template_name) in equipment {
            let Some(template) = self
                .project
                .templates
                .iter()
                .find(|t| t.name == template_name)
                .cloned()
            else {
                continue;
            };
            let existing: BTreeSet<String> = self
                .project
                .objects
                .iter()
                .filter(|o| o.parent_id == Some(eq_id) && o.object_type == ObjectType::Point)
                .map(|o| o.name.clone())
                .collect();
            for point_name in template.points {
                if existing.contains(&point_name) {
                    continue;
                }
                self.add_object(ObjectType::Point, Some(eq_id));
                if let Some(new_obj) = self.project.objects.last_mut() {
                    new_obj.name = point_name;
                    new_obj.property_groups.clear();
                }
            }
        }
    }

    fn duplicate_object(&mut self, id: u64) {
        let Some(obj) = self.project.objects.iter().find(|o| o.id == id).cloned() else {
            return;
        };
        let mut copy = obj;
        copy.id = self.project.next_id;
        self.project.next_id += 1;
        copy.name = format!("{} Copy", copy.name);
        self.project.objects.push(copy);
        self.selected_object = Some(self.project.next_id - 1);
    }

    fn project_overview(&mut self, ui: &mut Ui) {
        Self::card_frame().show(ui, |ui| {
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
        ui.horizontal(|ui| {
            if ui
                .button(if self.left_sidebar_collapsed {
                    "⮞"
                } else {
                    "⮜"
                })
                .clicked()
            {
                self.left_sidebar_collapsed = !self.left_sidebar_collapsed;
            }
            ui.label(RichText::new("Object Tree").strong());
        });
        if self.left_sidebar_collapsed {
            return;
        }
        self.project_overview(ui);
        ui.add_space(8.0);
        if ui.button("➕ Building").clicked() {
            self.add_object(ObjectType::Building, None);
        }

        egui::ScrollArea::both().show(ui, |ui| {
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

        let mut add_child = None;
        let mut delete_clicked = false;
        let mut duplicate_clicked = false;

        let selected = self.selected_object == Some(id);
        let title = format!("{} {}", obj.object_type.icon(), obj.name);
        let text = if selected {
            RichText::new(title).color(Color32::WHITE)
        } else {
            RichText::new(title).color(Color32::from_rgb(230, 235, 245))
        };

        let row = ui.selectable_label(selected, text);
        if row.clicked() {
            self.selected_object = Some(id);
        }
        row.context_menu(|ui| {
            if ui.button("Duplicate").clicked() {
                duplicate_clicked = true;
                ui.close_menu();
            }
            if ui.button("Delete").clicked() {
                delete_clicked = true;
                ui.close_menu();
            }
            match obj.object_type {
                ObjectType::Building => {
                    if ui.button("Add Controller").clicked() {
                        add_child = Some(ObjectType::Controller);
                        ui.close_menu();
                    }
                }
                ObjectType::Controller => {
                    if ui.button("Add Equipment").clicked() {
                        add_child = Some(ObjectType::Equipment);
                        ui.close_menu();
                    }
                }
                ObjectType::Equipment => {
                    if ui.button("Add Point").clicked() {
                        add_child = Some(ObjectType::Point);
                        ui.close_menu();
                    }
                }
                ObjectType::Point => {}
            }
        });

        if obj.object_type != ObjectType::Point {
            ui.horizontal_wrapped(|ui| match obj.object_type {
                ObjectType::Building => {
                    if ui.button("+ Controller").clicked() {
                        add_child = Some(ObjectType::Controller);
                    }
                }
                ObjectType::Controller => {
                    if ui.button("+ Equipment").clicked() {
                        add_child = Some(ObjectType::Equipment);
                    }
                }
                ObjectType::Equipment => {
                    if ui.button("+ Point").clicked() {
                        add_child = Some(ObjectType::Point);
                    }
                }
                ObjectType::Point => {}
            });
        }

        if let Some(kind) = add_child {
            self.add_object(kind, Some(id));
        }
        if duplicate_clicked {
            self.duplicate_object(id);
        }
        if delete_clicked && obj.object_type != ObjectType::Building {
            self.remove_object_subtree(id);
        }

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
            if let Some(eq_obj) = self.project.objects.iter_mut().find(|o| o.id == obj_id) {
                if eq_obj.equipment_type.trim().is_empty() {
                    eq_obj.equipment_type = template.equipment_type.clone();
                }
                if eq_obj.equipment_tag.trim().is_empty() {
                    eq_obj.equipment_tag = format!("{}-{}", template.equipment_type, obj_id);
                }
            }
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
                    new_obj.property_groups.clear();
                }
            }
        }
    }

    fn right_properties(&mut self, ui: &mut Ui) {
        ui.heading("Properties");
        if let Some(id) = self.selected_object {
            if let Some(index) = self.project.objects.iter().position(|o| o.id == id) {
                let mut apply_template = false;
                let mut delete_clicked = false;
                let obj = &mut self.project.objects[index];
                Self::card_frame().show(ui, |ui| {
                    ui.label(format!(
                        "{} {}",
                        obj.object_type.icon(),
                        obj.object_type.label()
                    ));
                    ui.text_edit_singleline(&mut obj.name);

                    if ui
                        .button(RichText::new("Delete Object").color(Color32::LIGHT_RED))
                        .clicked()
                    {
                        delete_clicked = true;
                    }

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

                    if obj.object_type == ObjectType::Equipment {
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
                    }
                });

                if delete_clicked {
                    if self.project.objects[index].object_type == ObjectType::Building {
                        self.status = "Delete blocked: building is required at root".to_string();
                    } else {
                        self.remove_object_subtree(id);
                    }
                }

                if apply_template {
                    self.apply_template_to_selected_equipment();
                }
            }
        }
    }

    fn project_settings_view(&mut self, ui: &mut Ui) {
        ui.heading("Project Settings & Proposal Inputs");
        egui::ScrollArea::both().show(ui, |ui| {
            ui.columns(3, |columns| {
                Self::card_frame().show(&mut columns[0], |ui| {
                    ui.label(RichText::new("Project Core").strong());
                    Self::labeled_singleline(ui, "Project Name", &mut self.project.name);
                    Self::labeled_singleline(
                        ui,
                        "Project #",
                        &mut self.project.proposal.project_number,
                    );
                    ui.horizontal(|ui| {
                        if ui.button("🖼 Upload Overview Image").clicked() {
                            if let Some(path) = FileDialog::new()
                                .add_filter("Images", &["png", "jpg", "jpeg", "bmp"])
                                .pick_file()
                            {
                                match fs::read(&path) {
                                    Ok(bytes) => {
                                        self.project.overview_image =
                                            Some(Self::sanitize_asset_name(&path));
                                        self.overview_image_bytes = Some(bytes);
                                        self.refresh_overview_texture(ui.ctx());
                                        self.status = "Loaded overview image".to_string();
                                    }
                                    Err(err) => self.status = format!("Image load failed: {err}"),
                                }
                            }
                        }
                        if let Some(path) = &self.project.overview_image {
                            ui.small(path);
                        }
                    });
                    if let Some(texture) = &self.overview_texture {
                        let w = ui.available_width().max(120.0);
                        let h = (w * 0.42).clamp(90.0, 180.0);
                        ui.add(egui::Image::new(texture).fit_to_exact_size(egui::vec2(w, h)));
                    }
                    ui.label("Project Notes");
                    ui.text_edit_multiline(&mut self.project.notes);
                });

                Self::card_frame().show(&mut columns[1], |ui| {
                    ui.label(RichText::new("Stakeholders").strong());
                    let p = &mut self.project.proposal;
                    Self::labeled_singleline(ui, "Client", &mut p.client_name);
                    Self::labeled_singleline(ui, "Owner", &mut p.owner);
                    Self::labeled_singleline(ui, "Engineer", &mut p.engineer_of_record);
                    Self::labeled_singleline(ui, "PM", &mut p.project_manager);
                    Self::labeled_singleline(ui, "Estimator", &mut p.estimator);
                });

                Self::card_frame().show(&mut columns[2], |ui| {
                    ui.label(RichText::new("Commercial & Schedule").strong());
                    let p = &mut self.project.proposal;
                    Self::labeled_singleline(ui, "Location", &mut p.project_location);
                    Self::labeled_singleline(ui, "Proposal #", &mut p.proposal_number);
                    Self::labeled_singleline(ui, "Revision", &mut p.revision);
                    Self::labeled_singleline(ui, "Contract", &mut p.contract_type);
                    Self::labeled_singleline(ui, "Design Stage", &mut p.design_stage);
                    Self::labeled_singleline(ui, "Bid Date", &mut p.bid_date);
                    Self::labeled_singleline(ui, "Start", &mut p.target_start_date);
                    Self::labeled_singleline(ui, "Completion", &mut p.target_completion_date);
                    Self::labeled_singleline(ui, "Prepared By", &mut p.prepared_by);
                });
            });

            ui.add_space(8.0);
            ui.columns(3, |columns| {
                Self::card_frame().show(&mut columns[0], |ui| {
                    ui.label(RichText::new("Scope Summary").strong());
                    ui.text_edit_multiline(&mut self.project.proposal.scope_summary);
                });
                Self::card_frame().show(&mut columns[1], |ui| {
                    ui.label(RichText::new("Assumptions").strong());
                    ui.text_edit_multiline(&mut self.project.proposal.assumptions);
                });
                Self::card_frame().show(&mut columns[2], |ui| {
                    ui.label(RichText::new("Exclusions").strong());
                    ui.text_edit_multiline(&mut self.project.proposal.exclusions);
                });
            });
        });
    }

    fn hours_estimator_view(&mut self, ui: &mut Ui) {
        ui.heading("Hours Estimator");
        let (eng, gfx, cx, mut custom_total, overhead, grand_total) = self.estimate_hours();

        Self::card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Hours Summation").strong());
                if ui.button("⏰").on_hover_text("Adjustments").clicked() {
                    self.show_adjustment_popup = true;
                }
            });
            egui::Grid::new("est_grid").num_columns(2).show(ui, |ui| {
                ui.label("Engineering");
                ui.label(format!("{eng:.1} h"));
                ui.end_row();
                ui.label("Graphics/Submittals");
                ui.label(format!("{gfx:.1} h"));
                ui.end_row();
                ui.label("Commissioning");
                ui.label(format!("{cx:.1} h"));
                ui.end_row();
                ui.label("Custom Lines");
                ui.label(format!("{custom_total:.1} h"));
                ui.end_row();
            });
        });

        if self.show_adjustment_popup {
            egui::Window::new("Hours Adjustments")
                .open(&mut self.show_adjustment_popup)
                .collapsible(false)
                .resizable(true)
                .show(ui.ctx(), |ui| {
                    ui.columns(3, |columns| {
                        columns[0].add(
                            egui::Slider::new(
                                &mut self.project.estimator.complexity_factor,
                                0.8..=1.6,
                            )
                            .text("Complexity"),
                        );
                        columns[1].add(
                            egui::Slider::new(
                                &mut self.project.estimator.renovation_factor,
                                0.9..=1.5,
                            )
                            .text("Renovation"),
                        );
                        columns[2].add(
                            egui::Slider::new(
                                &mut self.project.estimator.integration_factor,
                                0.8..=1.4,
                            )
                            .text("Integrations"),
                        );
                    });
                    ui.columns(3, |columns| {
                        columns[0].add(
                            egui::Slider::new(&mut self.project.estimator.qa_percent, 0.0..=20.0)
                                .text("QA %"),
                        );
                        columns[1].add(
                            egui::Slider::new(
                                &mut self.project.estimator.project_management_percent,
                                0.0..=25.0,
                            )
                            .text("PM %"),
                        );
                        columns[2].add(
                            egui::Slider::new(&mut self.project.estimator.risk_percent, 0.0..=20.0)
                                .text("Risk %"),
                        );
                    });
                });
        }

        ui.columns(3, |columns| {
            Self::card_frame().show(&mut columns[0], |ui| {
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

            Self::card_frame().show(&mut columns[1], |ui| {
                custom_total = self
                    .project
                    .custom_hour_lines
                    .iter()
                    .map(|line| line.quantity.max(0.0) * line.hours_per_unit.max(0.0))
                    .sum::<f32>();
                let base_total = eng + gfx + cx + custom_total;
                ui.label(RichText::new(format!("Base Total: {base_total:.1} h")).strong());
                ui.label(format!("Overhead & Risk: {overhead:.1} h"));
                ui.label(RichText::new(format!("Final Estimated Hours: {grand_total:.1} h")).strong());
                ui.small("Calibrated model: direct effort + complexity factors + QA/PM/risk overhead.");
            });

            Self::card_frame().show(&mut columns[2], |ui| {
                ui.label(RichText::new("Estimator Notes").strong());
                ui.label("Use the ⏰ button in Hours Summation to tune adjustment modifiers.");
                ui.label("Template hour mode can be configured in Template Tool (static or points-based).");
            });
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
                hour_mode: HourCalculationMode::StaticByEquipment,
                engineering_hours: 0.0,
                engineering_hours_per_point: 0.25,
                graphics_hours: 0.0,
                graphics_hours_per_point: 0.12,
                commissioning_hours: 0.0,
                commissioning_hours_per_point: 0.18,
            });
        }

        egui::ScrollArea::both().show(ui, |ui| {
            let mut remove_template = None;
            for (idx, template) in self.project.templates.iter_mut().enumerate() {
                Self::card_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.columns(3, |columns| {
                        columns[0].label("Template");
                        columns[0].add_sized(
                            [columns[0].available_width(), 22.0],
                            egui::TextEdit::singleline(&mut template.name),
                        );
                        columns[1].label("Type");
                        columns[1].add_sized(
                            [columns[1].available_width(), 22.0],
                            egui::TextEdit::singleline(&mut template.equipment_type),
                        );
                        if columns[2].button("Delete Template").clicked() {
                            remove_template = Some(idx);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Hours method");
                        ui.selectable_value(
                            &mut template.hour_mode,
                            HourCalculationMode::StaticByEquipment,
                            "Static by equipment",
                        );
                        ui.selectable_value(
                            &mut template.hour_mode,
                            HourCalculationMode::PointsBased,
                            "Points based",
                        );
                    });

                    ui.columns(3, |columns| {
                        columns[0].add(
                            egui::DragValue::new(
                                if template.hour_mode == HourCalculationMode::PointsBased {
                                    &mut template.engineering_hours_per_point
                                } else {
                                    &mut template.engineering_hours
                                },
                            )
                            .speed(0.1)
                            .prefix("Eng "),
                        );
                        columns[1].add(
                            egui::DragValue::new(
                                if template.hour_mode == HourCalculationMode::PointsBased {
                                    &mut template.graphics_hours_per_point
                                } else {
                                    &mut template.graphics_hours
                                },
                            )
                            .speed(0.1)
                            .prefix("Graphics "),
                        );
                        columns[2].add(
                            egui::DragValue::new(
                                if template.hour_mode == HourCalculationMode::PointsBased {
                                    &mut template.commissioning_hours_per_point
                                } else {
                                    &mut template.commissioning_hours
                                },
                            )
                            .speed(0.1)
                            .prefix("Cx "),
                        );
                    });

                    ui.separator();
                    ui.label(RichText::new("Point List").strong());
                    let mut remove_point = None;
                    for (pidx, point) in template.points.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [ui.available_width() - 34.0, 22.0],
                                egui::TextEdit::singleline(point),
                            );
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
                ui.add_space(6.0);
            }
            if let Some(idx) = remove_template {
                self.project.templates.remove(idx);
            }
        });
    }

    fn drawings_overlay_view(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.heading("Takeoff Workspace");
            ui.label(
                RichText::new(format!(
                    "{} • Drawing Rev {}",
                    self.project.name, self.project.proposal.revision
                ))
                .color(Color32::from_gray(180)),
            );
        });
        ui.horizontal(|ui| {
            if ui.button("Load PDF").clicked() {
                if let Some(path) = FileDialog::new().add_filter("PDF", &["pdf"]).pick_file() {
                    match fs::read(&path) {
                        Ok(bytes) => {
                            self.project.overlay_pdf = Some(Self::sanitize_asset_name(&path));
                            self.overlay_pdf_bytes = Some(bytes);
                            self.overlay_texture = None;
                            self.status = "Loaded overlay PDF".to_string();
                        }
                        Err(err) => self.status = format!("PDF load failed: {err}"),
                    }
                }
            }
            ui.label(
                self.project
                    .overlay_pdf
                    .clone()
                    .unwrap_or_else(|| "No PDF selected".to_string()),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new("Needs Clarification").color(Color32::from_rgb(224, 182, 86)),
                );
                ui.label(RichText::new("Assumed").color(Color32::from_rgb(221, 113, 113)));
                ui.label(RichText::new("Specified").color(Color32::from_rgb(122, 202, 137)));
            });
        });

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Controller token").drag_started() {
                self.dragging_palette = Some(ObjectType::Controller);
            }
            if ui.button("Equipment token").drag_started() {
                self.dragging_palette = Some(ObjectType::Equipment);
            }
            if ui.button("↶ Undo").clicked() {
                self.overlay_undo();
            }
            if ui.button("↷ Redo").clicked() {
                self.overlay_redo();
            }
        });

        let desired = egui::vec2(ui.available_width(), ui.available_height() - 16.0);
        let (resp, painter) = ui.allocate_painter(desired, egui::Sense::click_and_drag());
        if self.overlay_texture.is_none() && self.overlay_pdf_bytes.is_some() {
            self.refresh_overlay_texture(ui.ctx(), desired.x as u16);
        }
        painter.rect_filled(
            resp.rect,
            10.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 16),
        );
        painter.rect_stroke(resp.rect, 10.0, egui::Stroke::new(1.0, self.accent()));

        if let Some(texture) = &self.overlay_texture {
            painter.image(
                texture.id(),
                resp.rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::from_rgba_unmultiplied(255, 255, 255, 210),
            );
        }

        if self.project.settings.show_overlay_grid {
            let step = 36.0;
            let mut x = resp.rect.left();
            while x < resp.rect.right() {
                painter.line_segment(
                    [
                        egui::pos2(x, resp.rect.top()),
                        egui::pos2(x, resp.rect.bottom()),
                    ],
                    egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 16)),
                );
                x += step;
            }
            let mut y = resp.rect.top();
            while y < resp.rect.bottom() {
                painter.line_segment(
                    [
                        egui::pos2(resp.rect.left(), y),
                        egui::pos2(resp.rect.right(), y),
                    ],
                    egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 16)),
                );
                y += step;
            }
        }

        for (idx, node) in self.project.overlay_nodes.iter().enumerate() {
            let center = egui::pos2(resp.rect.left() + node.x, resp.rect.top() + node.y);
            let status_color = match idx % 3 {
                0 => Color32::from_rgba_unmultiplied(189, 86, 92, 220),
                1 => Color32::from_rgba_unmultiplied(193, 162, 78, 220),
                _ => Color32::from_rgba_unmultiplied(91, 156, 103, 220),
            };
            let obj_name = self
                .project
                .objects
                .iter()
                .find(|o| o.id == node.object_id)
                .map(|o| o.name.as_str())
                .unwrap_or("Token");

            let label_rect = egui::Rect::from_center_size(center, egui::vec2(138.0, 28.0));
            painter.rect_filled(label_rect, 6.0, status_color);
            painter.rect_stroke(
                label_rect,
                6.0,
                egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 90)),
            );
            painter.text(
                label_rect.center(),
                egui::Align2::CENTER_CENTER,
                obj_name,
                FontId::new(15.0, FontFamily::Proportional),
                Color32::WHITE,
            );
        }

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
                        self.pending_overlay_drop = Some((
                            kind,
                            [pointer.x - resp.rect.left(), pointer.y - resp.rect.top()],
                        ));
                    } else if resp.clicked() {
                        let local = [pointer.x - resp.rect.left(), pointer.y - resp.rect.top()];
                        if let Some(start) = self.active_line_start.take() {
                            self.push_overlay_history();
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

        if let Some((kind, pos)) = self.pending_overlay_drop.clone() {
            let mut open = true;
            egui::Window::new("Bind Token to Object")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Choose which object to place on the overlay.");
                    let candidates: Vec<(u64, String)> = self
                        .project
                        .objects
                        .iter()
                        .filter(|o| o.object_type == kind)
                        .map(|o| (o.id, o.name.clone()))
                        .collect();

                    if candidates.is_empty() {
                        ui.label("No matching objects found.");
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(220.0)
                            .show(ui, |ui| {
                                for (id, name) in candidates {
                                    if ui.button(name).clicked() {
                                        self.push_overlay_history();
                                        self.project.overlay_nodes.push(OverlayNode {
                                            id: self.project.next_id,
                                            object_id: id,
                                            x: pos[0],
                                            y: pos[1],
                                        });
                                        self.project.next_id += 1;
                                        self.pending_overlay_drop = None;
                                        self.status = "Placed overlay token".to_string();
                                    }
                                }
                            });
                    }

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.pending_overlay_drop = None;
                        }
                    });
                });

            if !open {
                self.pending_overlay_drop = None;
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
                    ui.separator();
                    ui.label(RichText::new("Signature: Built for M8 by ChatGPT").italics());
                });
        }

        if self.show_software_settings {
            let mut apply_recommended = false;
            egui::Window::new("Settings")
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
                    ui.add(
                        egui::Slider::new(&mut self.project.settings.autosave_minutes, 1..=60)
                            .text("Autosave (minutes)"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.project.settings.ui_scale, 0.8..=1.5)
                            .text("UI Scale"),
                    );
                    ui.checkbox(
                        &mut self.project.settings.show_overlay_grid,
                        "Show overlay grid",
                    );
                    ui.separator();
                    ui.label(RichText::new("Recommendations").strong());
                    if self.project.settings.autosave_minutes > 15 {
                        ui.colored_label(Color32::YELLOW, "• Consider autosave ≤ 15 minutes.");
                    }
                    if self.project.settings.ui_scale < 0.95
                        || self.project.settings.ui_scale > 1.25
                    {
                        ui.colored_label(
                            Color32::YELLOW,
                            "• UI scale between 0.95 and 1.25 is recommended for readability.",
                        );
                    }
                    if self.project.settings.company_name.trim().is_empty() {
                        ui.colored_label(
                            Color32::YELLOW,
                            "• Add a company name for exports and title metadata.",
                        );
                    }
                    if ui.button("Apply Recommended Defaults").clicked() {
                        apply_recommended = true;
                    }
                });
            if apply_recommended {
                self.apply_recommended_settings();
            }
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
        if self.app_screen != AppScreen::Splash {
            self.draw_studio_background(ctx);
        }
        ctx.set_pixels_per_point(self.project.settings.ui_scale);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(6.0, 6.0);
        if self.app_screen == AppScreen::Splash {
            style.visuals.window_fill = Color32::TRANSPARENT;
            style.visuals.panel_fill = Color32::TRANSPARENT;
        } else {
            style.visuals.window_fill = Color32::from_rgb(18, 23, 34);
            style.visuals.panel_fill = Color32::from_rgb(18, 23, 34);
        }
        style.visuals.widgets.noninteractive.bg_fill =
            Color32::from_rgba_unmultiplied(255, 255, 255, 10);
        style.visuals.override_text_color = Some(Color32::from_rgb(226, 233, 242));
        style.visuals.extreme_bg_color = Color32::from_rgb(9, 12, 20);
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgba_unmultiplied(28, 36, 49, 230);
        style.visuals.widgets.inactive.fg_stroke.color = Color32::from_rgb(225, 231, 240);
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

        match self.app_screen {
            AppScreen::Splash => self.splash_screen(ctx),
            AppScreen::Login => self.login_screen(ctx),
            AppScreen::Studio => {
                self.ensure_template_seeded();
                self.sync_template_points();
                self.autosave_project();
                self.titlebar(ctx, _frame);
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
                        ui.set_width(ui.available_width());
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
            }
        }
        ctx.request_repaint();
    }
}
