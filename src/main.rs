use eframe::{
    egui::{self, Color32, RichText, Ui},
    App, CreationContext, Frame, NativeOptions,
};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true)
            .with_decorations(false)
            .with_transparent(true)
            .with_inner_size([1600.0, 900.0]),
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
}

impl ToolView {
    fn label(self) -> &'static str {
        match self {
            ToolView::ProjectSettings => "Project Settings",
            ToolView::HoursEstimator => "Hours Estimator",
            ToolView::DrawingsOverlay => "Drawings Overlay",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PropertyGroup {
    name: String,
    items: Vec<PropertyItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    property_groups: Vec<PropertyGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OverlayNode {
    id: u64,
    object_id: u64,
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OverlayLine {
    from: [f32; 2],
    to: [f32; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppSettings {
    accent_color: [u8; 4],
    company_name: String,
    estimator_rate: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            accent_color: [98, 189, 255, 255],
            company_name: "AutoMate Controls".to_string(),
            estimator_rate: 145.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Project {
    name: String,
    notes: String,
    objects: Vec<BasObject>,
    overlay_pdf: Option<String>,
    overlay_nodes: Vec<OverlayNode>,
    overlay_lines: Vec<OverlayLine>,
    next_id: u64,
    settings: AppSettings,
}

impl Default for Project {
    fn default() -> Self {
        let building = BasObject {
            id: 1,
            parent_id: None,
            object_type: ObjectType::Building,
            name: "HQ Building".to_string(),
            property_groups: vec![
                PropertyGroup {
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
                },
                PropertyGroup {
                    name: "Scheduling".to_string(),
                    items: vec![PropertyItem {
                        key: "Occupied Hours".to_string(),
                        value: "06:00-18:00".to_string(),
                    }],
                },
            ],
        };

        Self {
            name: "New BAS Project".to_string(),
            notes: "Capture assumptions, scope notes, and exclusions here.".to_string(),
            objects: vec![building],
            overlay_pdf: None,
            overlay_nodes: vec![],
            overlay_lines: vec![],
            next_id: 2,
            settings: AppSettings::default(),
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
        }
    }

    fn accent(&self) -> Color32 {
        let [r, g, b, a] = self.project.settings.accent_color;
        Color32::from_rgba_unmultiplied(r, g, b, a)
    }

    fn panel_bg(&self) -> Color32 {
        let accent = self.accent();
        Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 28)
    }

    fn add_object(&mut self, object_type: ObjectType, parent: Option<u64>) {
        let name = format!("{} {}", object_type.label(), self.project.next_id);
        let id = self.project.next_id;
        self.project.next_id += 1;
        self.project.objects.push(BasObject {
            id,
            parent_id: parent,
            object_type,
            name,
            property_groups: vec![PropertyGroup {
                name: "General".to_string(),
                items: vec![
                    PropertyItem {
                        key: "Tag".to_string(),
                        value: format!("{}-{}", object_type.label(), id),
                    },
                    PropertyItem {
                        key: "Description".to_string(),
                        value: "".to_string(),
                    },
                ],
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
                        self.status = format!("Saved project to {}", path.display());
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
                        self.status = format!("Loaded project {}", path.display());
                        self.selected_object = self.project.objects.first().map(|o| o.id);
                    }
                    Err(e) => self.status = format!("Parse failed: {e}"),
                },
                Err(e) => self.status = format!("Load failed: {e}"),
            }
        }
    }

    fn titlebar(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("titlebar")
            .frame(
                egui::Frame::default()
                    .fill(Color32::from_rgba_unmultiplied(12, 16, 24, 190))
                    .inner_margin(egui::Margin::symmetric(12.0, 8.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("AutoMate BAS Studio")
                            .size(18.0)
                            .strong()
                            .color(self.accent()),
                    );
                    ui.separator();
                    ui.label(format!("Project: {}", self.project.name));
                    if let Some(path) = &self.project_path {
                        ui.label(format!("({})", path.display()));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("▢").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                        }
                        if ui.button("—").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                    });
                });

                let drag_area = ui.max_rect();
                let id = ui.id().with("titlebar_drag");
                let response = ui.interact(drag_area, id, egui::Sense::click_and_drag());
                if response.dragged() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
            });
    }

    fn toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            for view in [
                ToolView::ProjectSettings,
                ToolView::HoursEstimator,
                ToolView::DrawingsOverlay,
            ] {
                let selected = self.current_view == view;
                let button = egui::Button::new(view.label())
                    .fill(if selected {
                        Color32::from_rgba_unmultiplied(
                            self.accent().r(),
                            self.accent().g(),
                            self.accent().b(),
                            140,
                        )
                    } else {
                        Color32::from_rgba_unmultiplied(255, 255, 255, 25)
                    })
                    .stroke(egui::Stroke::new(1.0, self.accent()));
                if ui.add(button).clicked() {
                    self.current_view = view;
                }
            }

            ui.separator();

            if ui.button("New").clicked() {
                self.project = Project::default();
                self.selected_object = Some(1);
                self.project_path = None;
                self.status = "Created new project".to_string();
            }
            if ui.button("Save").clicked() {
                self.save_project();
            }
            if ui.button("Load").clicked() {
                self.load_project();
            }
            if ui.button("Settings").clicked() {
                self.show_software_settings = true;
            }
            if ui.button("About").clicked() {
                self.show_about = true;
            }
        });
    }

    fn left_sidebar(&mut self, ui: &mut Ui) {
        ui.heading("BAS Object Tree");
        ui.small("Buildings > Controllers > Equipment > Points");

        if ui.button("+ Building").clicked() {
            self.add_object(ObjectType::Building, None);
        }

        let roots: Vec<u64> = self
            .project
            .objects
            .iter()
            .filter(|o| o.parent_id.is_none())
            .map(|o| o.id)
            .collect();
        for root in roots {
            self.object_node(ui, root);
        }
    }

    fn object_node(&mut self, ui: &mut Ui, id: u64) {
        let obj = self.project.objects.iter().find(|o| o.id == id).cloned();
        let Some(obj) = obj else { return };

        let children: Vec<u64> = self
            .project
            .objects
            .iter()
            .filter(|child| child.parent_id == Some(id))
            .map(|child| child.id)
            .collect();

        egui::CollapsingHeader::new(format!("{}: {}", obj.object_type.label(), obj.name))
            .id_source(("tree", id))
            .default_open(true)
            .show(ui, |ui| {
                if ui
                    .selectable_label(self.selected_object == Some(id), "Select")
                    .clicked()
                {
                    self.selected_object = Some(id);
                }

                if obj.object_type == ObjectType::Building && ui.button("+ Controller").clicked() {
                    self.add_object(ObjectType::Controller, Some(id));
                }
                if obj.object_type == ObjectType::Controller && ui.button("+ Equipment").clicked() {
                    self.add_object(ObjectType::Equipment, Some(id));
                }
                if obj.object_type == ObjectType::Equipment && ui.button("+ Point").clicked() {
                    self.add_object(ObjectType::Point, Some(id));
                }

                for child in children {
                    self.object_node(ui, child);
                }
            });
    }

    fn right_properties(&mut self, ui: &mut Ui) {
        ui.heading("Properties");
        if let Some(id) = self.selected_object {
            if let Some(obj) = self.project.objects.iter_mut().find(|o| o.id == id) {
                ui.label(format!("{}", obj.object_type.label()));
                ui.text_edit_singleline(&mut obj.name);
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
                                    value: "".to_string(),
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
        } else {
            ui.label("Select an object to edit properties.");
        }
    }

    fn project_settings_view(&mut self, ui: &mut Ui) {
        ui.heading("Project Settings");
        ui.horizontal(|ui| {
            ui.label("Project Name");
            ui.text_edit_singleline(&mut self.project.name);
        });
        ui.label("Project Notes");
        ui.text_edit_multiline(&mut self.project.notes);

        ui.separator();
        ui.heading("Workflow Boosters");
        ui.label("• Data-first schema for faster takeoff + submittal generation");
        ui.label("• Save reusable templates by duplicating fully configured branches");
        ui.label("• Attach assumptions directly to objects in the tree");
    }

    fn hours_estimator_view(&mut self, ui: &mut Ui) {
        ui.heading("Hours Estimator");
        let controllers = self
            .project
            .objects
            .iter()
            .filter(|o| o.object_type == ObjectType::Controller)
            .count() as f32;
        let equipment = self
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

        let engineering_hours = controllers * 8.0 + equipment * 2.0 + points * 0.2;
        let graphics_hours = equipment * 1.5;
        let commissioning_hours = controllers * 6.0 + points * 0.15;
        let total = engineering_hours + graphics_hours + commissioning_hours;

        egui::Grid::new("hours_grid").show(ui, |ui| {
            ui.label("Engineering");
            ui.label(format!("{engineering_hours:.1} h"));
            ui.end_row();
            ui.label("Graphics & Submittals");
            ui.label(format!("{graphics_hours:.1} h"));
            ui.end_row();
            ui.label("Commissioning");
            ui.label(format!("{commissioning_hours:.1} h"));
            ui.end_row();
            ui.separator();
            ui.separator();
            ui.end_row();
            ui.label(RichText::new("Total").strong());
            ui.label(RichText::new(format!("{total:.1} h")).strong());
            ui.end_row();
        });

        let budget = total * self.project.settings.estimator_rate;
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Loaded Rate");
            ui.add(egui::DragValue::new(&mut self.project.settings.estimator_rate).speed(1.0));
        });
        ui.label(RichText::new(format!("Estimated Cost: ${budget:.2}")));
        ui.label(RichText::new(format!("Estimated Cost: ${budget:,.2}")));
    }

    fn drawings_overlay_view(&mut self, ui: &mut Ui) {
        ui.heading("Drawings Overlay");
        ui.horizontal(|ui| {
            if ui.button("Load PDF").clicked() {
                if let Some(pdf) = FileDialog::new().add_filter("PDF", &["pdf"]).pick_file() {
                    self.project.overlay_pdf = Some(pdf.display().to_string());
                }
            }
            if let Some(pdf) = &self.project.overlay_pdf {
                ui.label(format!("Loaded: {pdf}"));
            } else {
                ui.label("No PDF selected");
            }
            if ui.button("Draw Line").clicked() {
                self.active_line_start = None;
            }
        });

        ui.separator();
        ui.label("Drag controllers/equipment into the overlay. Click two points to create a line.");

        ui.horizontal(|ui| {
            if ui.button("Controller token").drag_started() {
                self.dragging_palette = Some(ObjectType::Controller);
            }
            if ui.button("Equipment token").drag_started() {
                self.dragging_palette = Some(ObjectType::Equipment);
            }
        });

        let desired = egui::vec2(ui.available_width(), ui.available_height() - 20.0);
        let (resp, painter) = ui.allocate_painter(desired, egui::Sense::click_and_drag());

        painter.rect_filled(
            resp.rect,
            8.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 20),
        );
        painter.rect_stroke(resp.rect, 8.0, egui::Stroke::new(1.0, self.accent()));
        painter.rect_stroke(
            resp.rect,
            8.0,
            egui::Stroke::new(1.0, self.accent()),
            egui::StrokeKind::Outside,
        );

        for line in &self.project.overlay_lines {
            let a = egui::pos2(
                resp.rect.left() + line.from[0],
                resp.rect.top() + line.from[1],
            );
            let b = egui::pos2(resp.rect.left() + line.to[0], resp.rect.top() + line.to[1]);
            painter.line_segment([a, b], egui::Stroke::new(2.0, self.accent()));
        }

        for node in &self.project.overlay_nodes {
            if let Some(obj) = self.project.objects.iter().find(|o| o.id == node.object_id) {
                let p = egui::pos2(resp.rect.left() + node.x, resp.rect.top() + node.y);
                painter.circle_filled(p, 10.0, self.accent());
                painter.text(
                    p + egui::vec2(12.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    obj.name.as_str(),
                    egui::TextStyle::Body.resolve(ui.style()),
                    Color32::WHITE,
                );
            }
        }

        if resp.hovered() {
            if let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) {
                if ui.input(|i| i.pointer.any_released()) {
                    if let Some(kind) = self.dragging_palette.take() {
                        if let Some(object) = self
                            .project
                            .objects
                            .iter()
                            .find(|o| o.object_type == kind)
                            .map(|o| o.id)
                        {
                            self.project.overlay_nodes.push(OverlayNode {
                                id: self.project.next_id,
                                object_id: object,
                                x: pointer.x - resp.rect.left(),
                                y: pointer.y - resp.rect.top(),
                            });
                            self.project.next_id += 1;
                        } else {
                            self.status = format!(
                                "Create at least one {} before dropping on drawing",
                                kind.label()
                            );
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
            egui::Window::new("About AutoMate BAS Studio")
                .open(&mut self.show_about)
                .show(ctx, |ui| {
                    ui.label("Version 0.1.0");
                    ui.label("Data-driven BAS estimating + submittal workspace");
                    ui.label("Built in Rust + egui with glass-inspired UI.");
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
}

impl App for AutoMateApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        let mut style = (*ctx.style()).clone();
        style.visuals.panel_fill = self.panel_bg();
        style.visuals.widgets.active.bg_fill = self.accent();
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgba_unmultiplied(
            self.accent().r(),
            self.accent().g(),
            self.accent().b(),
            100,
        );
        ctx.set_style(style);

        self.titlebar(ctx, frame);

        egui::TopBottomPanel::top("toolbar")
            .frame(egui::Frame::default().fill(Color32::from_rgba_unmultiplied(15, 20, 30, 150)))
            .show(ctx, |ui| self.toolbar(ui));

        egui::TopBottomPanel::bottom("status")
            .frame(egui::Frame::default().fill(Color32::from_rgba_unmultiplied(14, 18, 28, 140)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(self.status.as_str());
                    ui.separator();
                    ui.label("Tip: Keep object data complete to auto-generate better takeoffs.");
                });
            });

        egui::SidePanel::left("objects")
            .resizable(true)
            .frame(egui::Frame::default().fill(Color32::from_rgba_unmultiplied(20, 25, 35, 120)))
            .default_width(300.0)
            .show(ctx, |ui| self.left_sidebar(ui));

        egui::SidePanel::right("properties")
            .resizable(true)
            .frame(egui::Frame::default().fill(Color32::from_rgba_unmultiplied(20, 25, 35, 120)))
            .default_width(360.0)
            .show(ctx, |ui| self.right_properties(ui));

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(Color32::from_rgba_unmultiplied(25, 30, 42, 100)))
            .show(ctx, |ui| match self.current_view {
                ToolView::ProjectSettings => self.project_settings_view(ui),
                ToolView::HoursEstimator => self.hours_estimator_view(ui),
                ToolView::DrawingsOverlay => self.drawings_overlay_view(ui),
            });

        self.dialogs(ctx);
    }
}
