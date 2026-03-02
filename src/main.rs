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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    LineAudit,
    Refactor,
    Features,
    Dependencies,
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
}

impl App for AutoMateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.top_bar(ctx);
        self.side_nav(ctx);
        self.central_ui(ctx);
    }
}

fn seed_reviews() -> Vec<LineReview> {
    vec![
        LineReview {
            area: "Imports",
            finding: "Large import surface mixes UI, PDF, IO, exports, diagnostics, and logging in one file.",
            action: "Split into modules (`ui`, `domain`, `io`, `overlay`, `export`) and limit imports per module.",
            keep: false,
        },
        LineReview {
            area: "App bootstrap",
            finding: "Native window setup is solid and provides product-quality defaults.",
            action: "Keep native bootstrap but move window constants into `config.rs`.",
            keep: true,
        },
        LineReview {
            area: "Domain model",
            finding: "Single file contains too many structs and optional fields, increasing schema drift risk.",
            action: "Normalize model types and use enums/newtypes to reduce optional-field sprawl.",
            keep: false,
        },
        LineReview {
            area: "Estimator logic",
            finding: "Core formulas are valuable but deeply coupled to UI state.",
            action: "Extract estimator into pure functions with deterministic tests.",
            keep: false,
        },
        LineReview {
            area: "Overlay workflow",
            finding: "Overlay capabilities are strong but stateful code makes undo/redo difficult to verify.",
            action: "Introduce command history abstraction for canvas actions.",
            keep: false,
        },
        LineReview {
            area: "Persistence",
            finding: "JSON + archive support is useful and should remain.",
            action: "Keep persistence path but centralize all save/load errors under one error type.",
            keep: true,
        },
    ]
}

fn seed_features() -> Vec<FeatureIdea> {
    vec![
        FeatureIdea { name: "Live proposal preview", value: "See export output before generating files", complexity: "Medium" },
        FeatureIdea { name: "Command palette", value: "Keyboard-first navigation for power users", complexity: "Low" },
        FeatureIdea { name: "Scenario compare mode", value: "A/B estimate alternatives in one view", complexity: "Medium" },
        FeatureIdea { name: "Template versioning", value: "Track changes and roll back bad edits", complexity: "Medium" },
        FeatureIdea { name: "Dependency health dashboard", value: "Visibility into build and security posture", complexity: "Low" },
        FeatureIdea { name: "Overlay snap + guides", value: "Faster and cleaner routing alignment", complexity: "Medium" },
        FeatureIdea { name: "Plugin-style exporters", value: "Custom outputs without touching core", complexity: "High" },
        FeatureIdea { name: "Realtime collaboration hints", value: "Surface merge conflicts early", complexity: "High" },
        FeatureIdea { name: "Audit trail timeline", value: "Who changed what and when", complexity: "Medium" },
        FeatureIdea { name: "Smart point suggestions", value: "Generate point lists from equipment context", complexity: "High" },
    ]
}

fn seed_dependencies() -> Vec<DependencyDecision> {
    vec![
        DependencyDecision {
            crate_name: "eframe / egui",
            decision: "Keep + upgrade cadence",
            rationale: "Core UI stack; preserve and track minor releases for rendering and input improvements.",
        },
        DependencyDecision {
            crate_name: "rayon",
            decision: "Keep",
            rationale: "Useful for parallel project diagnostics and estimator workloads on larger models.",
        },
        DependencyDecision {
            crate_name: "printpdf",
            decision: "Keep (re-evaluate with integration tests)",
            rationale: "Needed for proposal package output until a full replacement benchmark exists.",
        },
        DependencyDecision {
            crate_name: "pdfium-render",
            decision: "Keep with lazy loading",
            rationale: "Critical for drawing overlays; isolate initialization and report runtime status.",
        },
        DependencyDecision {
            crate_name: "pollster",
            decision: "Remove",
            rationale: "No longer needed in the streamlined sync-first architecture.",
        },
        DependencyDecision {
            crate_name: "once_cell",
            decision: "Remove",
            rationale: "Can be replaced by `const` and `std::sync::OnceLock` where static init is required.",
        },
        DependencyDecision {
            crate_name: "petgraph (new)",
            decision: "Add",
            rationale: "Formal graph model for BAS object traversal, validation, and path diagnostics.",
        },
        DependencyDecision {
            crate_name: "tracing / tracing-subscriber",
            decision: "Keep + expand instrumentation",
            rationale: "Essential for diagnosing export and overlay latency in production builds.",
        },
    ]
}
