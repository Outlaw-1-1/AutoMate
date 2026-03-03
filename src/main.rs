use eframe::{egui, App, CreationContext, Frame, NativeOptions};

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1680.0, 980.0])
            .with_min_inner_size([1200.0, 760.0])
            .with_title("AutoMate BAS Studio 2.0"),
        ..Default::default()
    };

    eframe::run_native(
        "AutoMate BAS Studio 2.0",
        options,
        Box::new(|cc| Ok(Box::new(AutoMateApp::new(cc)))),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
enum ToolView {
    ProjectSettings,
    HoursEstimator,
    DrawingsOverlay,
    Templates,
    FeatureControlCenter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverlayTool {
    Route,
    PlaceController,
    PlaceEquipment,
}

impl OverlayTool {
    fn label(self) -> &'static str {
        match self {
            OverlayTool::Route => "Wire tool",
            OverlayTool::PlaceController => "Place controller",
            OverlayTool::PlaceEquipment => "Place equipment",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppScreen {
    Splash,
    Login,
    Studio,
}

const SPLASH_WINDOW_SIZE: f32 = 200.0;
const LOGIN_WINDOW_DEFAULT_SIZE: [f32; 2] = [1200.0, 760.0];
const LOGIN_WINDOW_MIN_SIZE: [f32; 2] = [960.0, 620.0];
const STUDIO_WINDOW_SIZE: [f32; 2] = [1600.0, 920.0];

#[derive(Debug, Error)]
enum AppIoError {
    #[error("Serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Filesystem error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

impl ToolView {
    fn label(self) -> &'static str {
        match self {
            ToolView::ProjectSettings => "Project Settings",
            ToolView::HoursEstimator => "Hours Estimator",
            ToolView::DrawingsOverlay => "Drawings Overlay",
            ToolView::Templates => "Template Tool",
            ToolView::FeatureControlCenter => "Feature Control Center",
        }
    }
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, EnumIter, JsonSchema,
)]
enum ObjectType {
    Building,
    Controller,
    Equipment,
    Point,
}

impl Tab {
    fn label(self) -> &'static str {
        match self {
            Self::LineAudit => "Line Audit",
            Self::Refactor => "Refactor Plan",
            Self::Features => "Feature Lab",
            Self::Dependencies => "Dependency Strategy",
        }
    }
}

#[derive(Clone)]
struct LineReview {
    area: &'static str,
    finding: &'static str,
    action: &'static str,
    keep: bool,
}

#[derive(Clone)]
struct FeatureIdea {
    name: &'static str,
    value: &'static str,
    complexity: &'static str,
}

#[derive(Clone)]
struct DependencyDecision {
    crate_name: &'static str,
    decision: &'static str,
    rationale: &'static str,
}

struct AutoMateApp {
    tab: Tab,
    dark_mode: bool,
    ui_density: f32,
    search: String,
    reviews: Vec<LineReview>,
    refactor_steps: Vec<&'static str>,
    features: Vec<FeatureIdea>,
    dependencies: Vec<DependencyDecision>,
}

impl AutoMateApp {
    fn new(cc: &CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        Self {
            tab: Tab::LineAudit,
            dark_mode: true,
            ui_density: 1.0,
            search: String::new(),
            reviews: seed_reviews(),
            refactor_steps: vec![
                "Split monolithic model into core domain structs + dedicated services",
                "Extract estimating engine into testable pure module",
                "Replace scattered state mutations with command-style actions",
                "Consolidate export flow behind unified proposal pipeline",
                "Normalize naming and remove duplicate data fields",
                "Introduce view-model layer between UI and domain",
                "Add project diagnostics panel with deterministic checks",
                "Capture performance metrics for large drawings",
            ],
            features: seed_features(),
            dependencies: seed_dependencies(),
        }
    }

    fn top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.heading("🚀 AutoMate BAS Studio 2.0");
                ui.separator();
                ui.label("Reconstruction workspace");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .selectable_label(self.dark_mode, "Dark")
                        .on_hover_text("Toggle theme")
                        .clicked()
                    {
                        self.dark_mode = !self.dark_mode;
                        if self.dark_mode {
                            ctx.set_visuals(egui::Visuals::dark());
                        } else {
                            ctx.set_visuals(egui::Visuals::light());
                        }
                    }
                    ui.add(egui::Slider::new(&mut self.ui_density, 0.8..=1.4).text("Density"));
                });
            });
            ui.add_space(4.0);
        });
    }

    fn side_nav(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_nav")
            .resizable(true)
            .default_width(240.0)
            .show(ctx, |ui| {
                ui.heading("Workspace");
                ui.separator();
                for tab in [Tab::LineAudit, Tab::Refactor, Tab::Features, Tab::Dependencies] {
                    let selected = self.tab == tab;
                    if ui.selectable_label(selected, tab.label()).clicked() {
                        self.tab = tab;
                    }
                }
                ui.separator();
                ui.label("Global Search");
                ui.text_edit_singleline(&mut self.search);
                ui.add_space(8.0);
                let completion = (self.refactor_steps.len() as f32 / 10.0).min(1.0);
                ui.add(egui::ProgressBar::new(completion).text("2.0 Plan Progress"));
                ui.small("Target: complete 10 strategic changes");
            });
    }

    fn central_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(10.0 * self.ui_density, 8.0 * self.ui_density);
            match self.tab {
                Tab::LineAudit => self.render_line_audit(ui),
                Tab::Refactor => self.render_refactor(ui),
                Tab::Features => self.render_features(ui),
                Tab::Dependencies => self.render_dependencies(ui),
            }
        });
    }

    fn render_line_audit(&self, ui: &mut egui::Ui) {
        ui.heading("main.rs line-by-line audit summary");
        ui.label("Each entry captures what should be kept, improved, or removed.");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for entry in self.filtered_reviews() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.strong(entry.area);
                        let chip = if entry.keep { "KEEP" } else { "CHANGE" };
                        let color = if entry.keep {
                            egui::Color32::from_rgb(40, 180, 110)
                        } else {
                            egui::Color32::from_rgb(225, 135, 65)
                        };
                        ui.colored_label(color, chip);
                    });
                    ui.label(format!("Finding: {}", entry.finding));
                    ui.label(format!("Action: {}", entry.action));
                });
            }
        });
    }

    fn filtered_reviews(&self) -> Vec<&LineReview> {
        self.reviews
            .iter()
            .filter(|r| {
                self.search.is_empty()
                    || r.area.to_lowercase().contains(&self.search.to_lowercase())
                    || r.finding.to_lowercase().contains(&self.search.to_lowercase())
                    || r.action.to_lowercase().contains(&self.search.to_lowercase())
            })
            .collect()
    }

    fn render_refactor(&self, ui: &mut egui::Ui) {
        ui.heading("Reconstruction blueprint");
        ui.label("A practical sequence to evolve main.rs into a maintainable platform.");
        ui.separator();

        for (idx, step) in self.refactor_steps.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.monospace(format!("{:02}", idx + 1));
                ui.label(*step);
            });
        }
    }

    fn render_features(&self, ui: &mut egui::Ui) {
        ui.heading("10 new features for 2.0");
        ui.separator();

        egui::Grid::new("features_grid")
            .striped(true)
            .num_columns(3)
            .show(ui, |ui| {
                ui.strong("Feature");
                ui.strong("User Value");
                ui.strong("Complexity");
                ui.end_row();

                for feature in &self.features {
                    ui.label(feature.name);
                    ui.label(feature.value);
                    ui.label(feature.complexity);
                    ui.end_row();
                }
            });
    }

    fn render_dependencies(&self, ui: &mut egui::Ui) {
        ui.heading("Cargo dependency decisions");
        ui.label("Curated keep/remove/add strategy for a faster and cleaner build graph.");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for dep in &self.dependencies {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.strong(dep.crate_name);
                        ui.label("→");
                        ui.colored_label(egui::Color32::LIGHT_BLUE, dep.decision);
                    });
                    ui.label(dep.rationale);
                });
            }
        });
    }

    fn feature_control_center_view(&mut self, ui: &mut Ui) {
        ui.heading("Feature Control Center");
        ui.label("Everything critical in AutoMate is still here. Use this panel as the one-stop operational checklist.");
        ui.add_space(8.0);

        let (ready_count, ready_total) = self.export_readiness_score();
        let overlay_ready = self.project.overlay_pdf.is_some();
        let template_ready = !self.user_templates.is_empty();
        let has_equipment = self
            .project
            .objects
            .iter()
            .any(|o| o.object_type == ObjectType::Equipment);

        Self::card_frame().show(ui, |ui| {
            ui.label(RichText::new("Core Capabilities").strong());
            ui.separator();
            ui.label(format!("{} Project settings + proposal inputs", if !self.project.name.trim().is_empty() { "✅" } else { "⚠" }));
            ui.label(format!("{} Hours estimator", if has_equipment { "✅" } else { "⚠" }));
            ui.label(format!("{} Drawings overlay", if overlay_ready { "✅" } else { "⚠" }));
            ui.label(format!("{} Template engine", if template_ready { "✅" } else { "⚠" }));
            ui.label(format!("{} Export readiness ({ready_count}/{ready_total})", if ready_count == ready_total { "✅" } else { "⚠" }));
        });

        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Go to Project Settings").clicked() {
                self.current_view = ToolView::ProjectSettings;
            }
            if ui.button("Go to Hours Estimator").clicked() {
                self.current_view = ToolView::HoursEstimator;
            }
            if ui.button("Go to Drawings Overlay").clicked() {
                self.current_view = ToolView::DrawingsOverlay;
            }
            if ui.button("Go to Templates").clicked() {
                self.current_view = ToolView::Templates;
            }
        });

        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Run Health Check").clicked() {
                self.status = match self.project_health_summary() {
                    Ok(summary) => summary,
                    Err(err) => format!("Health check failed: {err:#}"),
                };
            }
            if ui.button("Validate Export Readiness").clicked() {
                self.status = match self.validate_export_readiness() {
                    Ok(_) => "Export package ready".to_string(),
                    Err(err) => err,
                };
            }
            if ui.button("Export Proposal Markdown").clicked() {
                self.export_proposal_markdown();
            }
            if ui.button("Export Objects CSV").clicked() {
                self.export_objects_csv();
            }
            if ui.button("Export Project Schema").clicked() {
                self.export_project_schema();
            }
        });
    }

    fn feature_control_center_view(&mut self, ui: &mut Ui) {
        ui.heading("Feature Control Center");
        ui.label("Everything critical in AutoMate is still here. Use this panel as the one-stop operational checklist.");
        ui.add_space(8.0);

        let (ready_count, ready_total) = self.export_readiness_score();
        let overlay_ready = self.project.overlay_pdf.is_some();
        let template_ready = !self.user_templates.is_empty();
        let has_equipment = self
            .project
            .objects
            .iter()
            .any(|o| o.object_type == ObjectType::Equipment);

        Self::card_frame().show(ui, |ui| {
            ui.label(RichText::new("Core Capabilities").strong());
            ui.separator();
            ui.label(format!("{} Project settings + proposal inputs", if !self.project.name.trim().is_empty() { "✅" } else { "⚠" }));
            ui.label(format!("{} Hours estimator", if has_equipment { "✅" } else { "⚠" }));
            ui.label(format!("{} Drawings overlay", if overlay_ready { "✅" } else { "⚠" }));
            ui.label(format!("{} Template engine", if template_ready { "✅" } else { "⚠" }));
            ui.label(format!("{} Export readiness ({ready_count}/{ready_total})", if ready_count == ready_total { "✅" } else { "⚠" }));
        });

        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Go to Project Settings").clicked() {
                self.current_view = ToolView::ProjectSettings;
            }
            if ui.button("Go to Hours Estimator").clicked() {
                self.current_view = ToolView::HoursEstimator;
            }
            if ui.button("Go to Drawings Overlay").clicked() {
                self.current_view = ToolView::DrawingsOverlay;
            }
            if ui.button("Go to Templates").clicked() {
                self.current_view = ToolView::Templates;
            }
        });

        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Run Health Check").clicked() {
                self.status = match self.project_health_summary() {
                    Ok(summary) => summary,
                    Err(err) => format!("Health check failed: {err:#}"),
                };
            }
            if ui.button("Validate Export Readiness").clicked() {
                self.status = match self.validate_export_readiness() {
                    Ok(_) => "Export package ready".to_string(),
                    Err(err) => err,
                };
            }
            if ui.button("Export Proposal Markdown").clicked() {
                self.export_proposal_markdown();
            }
            if ui.button("Export Objects CSV").clicked() {
                self.export_objects_csv();
            }
            if ui.button("Export Project Schema").clicked() {
                self.export_project_schema();
            }
        });
    }

    fn feature_control_center_view(&mut self, ui: &mut Ui) {
        ui.heading("Feature Control Center");
        ui.label("Everything critical in AutoMate is still here. Use this panel as the one-stop operational checklist.");
        ui.add_space(8.0);

        let (ready_count, ready_total) = self.export_readiness_score();
        let overlay_ready = self.project.overlay_pdf.is_some();
        let template_ready = !self.user_templates.is_empty();
        let has_equipment = self
            .project
            .objects
            .iter()
            .any(|o| o.object_type == ObjectType::Equipment);

        Self::card_frame().show(ui, |ui| {
            ui.label(RichText::new("Core Capabilities").strong());
            ui.separator();
            ui.label(format!("{} Project settings + proposal inputs", if !self.project.name.trim().is_empty() { "✅" } else { "⚠" }));
            ui.label(format!("{} Hours estimator", if has_equipment { "✅" } else { "⚠" }));
            ui.label(format!("{} Drawings overlay", if overlay_ready { "✅" } else { "⚠" }));
            ui.label(format!("{} Template engine", if template_ready { "✅" } else { "⚠" }));
            ui.label(format!("{} Export readiness ({ready_count}/{ready_total})", if ready_count == ready_total { "✅" } else { "⚠" }));
        });

        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Go to Project Settings").clicked() {
                self.current_view = ToolView::ProjectSettings;
            }
            if ui.button("Go to Hours Estimator").clicked() {
                self.current_view = ToolView::HoursEstimator;
            }
            if ui.button("Go to Drawings Overlay").clicked() {
                self.current_view = ToolView::DrawingsOverlay;
            }
            if ui.button("Go to Templates").clicked() {
                self.current_view = ToolView::Templates;
            }
        });

        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Run Health Check").clicked() {
                self.status = match self.project_health_summary() {
                    Ok(summary) => summary,
                    Err(err) => format!("Health check failed: {err:#}"),
                };
            }
            if ui.button("Validate Export Readiness").clicked() {
                self.status = match self.validate_export_readiness() {
                    Ok(_) => "Export package ready".to_string(),
                    Err(err) => err,
                };
            }
            if ui.button("Export Proposal Markdown").clicked() {
                self.export_proposal_markdown();
            }
            if ui.button("Export Objects CSV").clicked() {
                self.export_objects_csv();
            }
            if ui.button("Export Project Schema").clicked() {
                self.export_project_schema();
            }
        });
    }
}

impl App for AutoMateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.configure_viewport_for_screen(ctx);
        self.handle_shortcuts(ctx);
        if self.app_screen == AppScreen::Studio {
            self.draw_studio_background(ctx);
        }
        ctx.set_pixels_per_point(self.project.settings.ui_scale);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(6.0, 6.0);
        if self.app_screen != AppScreen::Studio {
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
                            ToolView::FeatureControlCenter => self.feature_control_center_view(ui),
                        }
                    });

                self.dialogs(ctx);
            }
        }
        ctx.request_repaint();
    }
}
