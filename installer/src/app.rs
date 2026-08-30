use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use eframe::egui;

use crate::args::InstallPreset;
use crate::elevate;
use crate::install::{self, InstallRequest, UninstallRequest};
use crate::theme::{self, InstallStepVisual, ACCENT, DANGER, INK, MUTE};
use crate::ui::animation::{self, PROGRESS_LERP};
use crate::ui::layout::{self, FooterActions};
use crate::ui::APP_NAME;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Welcome,
    Path,
    Options,
    Progress,
    Done,
    Error,
}

enum JobEvent {
    Progress(f32, String),
    Ok { webview_ok: bool },
    Err(String),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StepKind {
    Prepare,
    Copy,
    Runtime,
    Shortcuts,
    Finalize,
}

impl StepKind {
    fn label(self) -> &'static str {
        match self {
            StepKind::Prepare => "Przygotowanie",
            StepKind::Copy => "Kopiowanie plików",
            StepKind::Runtime => "Środowisko uruchomieniowe",
            StepKind::Shortcuts => "Skróty",
            StepKind::Finalize => "Finalizacja",
        }
    }

    fn all() -> [StepKind; 5] {
        [
            StepKind::Prepare,
            StepKind::Copy,
            StepKind::Runtime,
            StepKind::Shortcuts,
            StepKind::Finalize,
        ]
    }
}

pub struct InstallerApp {
    uninstall: bool,
    page: Page,
    prev_page: Page,
    page_anim_t: f32,
    dest: String,
    start_menu: bool,
    desktop: bool,
    all_users: bool,
    launch_after: bool,
    remove_data: bool,
    progress: f32,
    display_progress: f32,
    status: String,
    error: Option<String>,
    webview_ok: bool,
    fonts_ready: bool,
    auto_close: bool,
    close_now: bool,
    success_anim_t: f32,
    rx: Option<Receiver<JobEvent>>,
}

impl InstallerApp {
    pub fn new(cc: &eframe::CreationContext<'_>, uninstall: bool, preset: InstallPreset) -> Self {
        theme::apply(&cc.egui_ctx);
        let dest = preset
            .dest
            .unwrap_or_else(|| install::default_dest(preset.all_users));
        Self {
            uninstall,
            page: Page::Welcome,
            prev_page: Page::Welcome,
            page_anim_t: 1.0,
            dest: dest.to_string_lossy().into_owned(),
            start_menu: preset.start_menu,
            desktop: preset.desktop,
            all_users: preset.all_users,
            launch_after: true,
            remove_data: false,
            progress: 0.0,
            display_progress: 0.0,
            status: String::new(),
            error: None,
            webview_ok: true,
            fonts_ready: false,
            auto_close: false,
            close_now: false,
            success_anim_t: 0.0,
            rx: None,
        }
    }

    pub fn start_elevated_install(&mut self) {
        self.begin_install();
    }

    pub fn start_unattended(&mut self, req: InstallRequest) {
        self.dest = req.dest.to_string_lossy().into_owned();
        self.start_menu = req.start_menu;
        self.desktop = req.desktop;
        self.all_users = req.all_users;
        self.launch_after = false;
        self.auto_close = true;
        self.spawn_install(req);
    }

    pub fn start_unattended_uninstall(&mut self) {
        self.auto_close = true;
        self.begin_uninstall();
    }

    fn set_page(&mut self, page: Page) {
        if self.page != page {
            self.prev_page = self.page;
            self.page = page;
            self.page_anim_t = 0.0;
            if page == Page::Done {
                self.success_anim_t = 0.0;
            }
        }
    }

    fn begin_install(&mut self) {
        let req = InstallRequest {
            dest: PathBuf::from(&self.dest),
            start_menu: self.start_menu,
            desktop: self.desktop,
            all_users: self.all_users,
            update: false,
            restart: false,
            restart_args: Vec::new(),
        };
        self.spawn_install(req);
    }

    fn spawn_install(&mut self, req: InstallRequest) {
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        self.set_page(Page::Progress);
        self.progress = 0.0;
        self.display_progress = 0.0;
        self.status = "Przygotowywanie…".into();
        thread::spawn(move || run_install_job(req, tx));
    }

    fn begin_uninstall(&mut self) {
        let req = UninstallRequest {
            dest: PathBuf::from(&self.dest),
            all_users: self.all_users,
            remove_data: self.remove_data,
        };
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        self.set_page(Page::Progress);
        self.progress = 0.0;
        self.display_progress = 0.0;
        self.status = "Usuwanie…".into();
        thread::spawn(move || run_uninstall_job(req, tx));
    }

    fn pump(&mut self) {
        let events: Vec<JobEvent> = match &self.rx {
            Some(rx) => rx.try_iter().collect(),
            None => return,
        };
        for ev in events {
            match ev {
                JobEvent::Progress(p, s) => {
                    self.progress = p;
                    self.status = s;
                }
                JobEvent::Ok { webview_ok } => {
                    self.webview_ok = webview_ok;
                    if self.auto_close {
                        self.close_now = true;
                    } else {
                        self.set_page(Page::Done);
                    }
                }
                JobEvent::Err(e) => {
                    self.error = Some(e);
                    self.set_page(Page::Error);
                }
            }
        }
    }

    fn tick_animations(&mut self, ctx: &egui::Context) {
        let dt = ctx.input(|i| i.stable_dt);
        self.page_anim_t = animation::advance_t(
            self.page_anim_t,
            dt,
            animation::PAGE_TRANSITION_MS,
        );
        self.display_progress = animation::lerp(
            self.display_progress,
            self.progress,
            PROGRESS_LERP,
        );
        if self.page == Page::Done {
            self.success_anim_t = animation::advance_t(self.success_anim_t, dt, 450.0);
        }
    }

    fn step_index(&self) -> u32 {
        match self.page {
            Page::Welcome => 0,
            Page::Path => 1,
            Page::Options => 2,
            Page::Progress => 3,
            Page::Done | Page::Error => 3,
        }
    }

    fn install_step_states(&self) -> Vec<(StepKind, InstallStepVisual)> {
        let active = active_step_from_status(&self.status, self.progress);
        StepKind::all()
            .into_iter()
            .map(|kind| {
                let visual = if (kind as u8) < (active as u8) {
                    InstallStepVisual::Done
                } else if kind == active {
                    InstallStepVisual::Active
                } else {
                    InstallStepVisual::Pending
                };
                (kind, visual)
            })
            .collect()
    }
}

impl eframe::App for InstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.fonts_ready {
            theme::apply(ctx);
            self.fonts_ready = true;
        }
        theme::block_editor_shortcuts(ctx);
        self.pump();
        self.tick_animations(ctx);
        if self.close_now {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ctx.request_repaint();

        let screen = ctx.screen_rect();
        ctx.layer_painter(egui::LayerId::background())
            .rect_filled(screen, 0.0, theme::BG);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                layout::titlebar(ui, self.uninstall);
                let _body = ui.available_rect_before_wrap();
                let version = env!("LUMEN_VERSION");
                layout::split_shell(ui, self.uninstall, version, |ui, right_rect| {
                    let (content, footer) = layout::right_content_rect(right_rect);
                    let alpha = animation::page_alpha(self.page_anim_t);
                    let offset = animation::page_offset(self.page_anim_t);

                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(content), |ui| {
                        ui.add_space(offset);
                        ui.set_opacity(alpha);
                        if self.uninstall {
                            uninstall_body(ui, self);
                        } else {
                            install_body(ui, self);
                        }
                    });

                    draw_footer(ui, footer, self);
                });
            });
    }
}

fn draw_footer(ui: &mut egui::Ui, footer: egui::Rect, app: &mut InstallerApp) {
    let ctx = ui.ctx().clone();

    let (back, primary) = match app.page {
        Page::Welcome => {
            let (b, p) = layout::footer_with_cta(
                ui,
                footer,
                FooterActions {
                    back_label: Some("Anuluj"),
                    back: true,
                    primary_label: if app.uninstall { "Usuń" } else { "Dalej" },
                    primary: true,
                },
            );
            if b {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            if p {
                if app.uninstall {
                    app.begin_uninstall();
                } else {
                    app.set_page(Page::Path);
                }
            }
            return;
        }
        Page::Path => layout::footer_with_cta(
            ui,
            footer,
            FooterActions {
                back_label: Some("Wstecz"),
                back: true,
                primary_label: "Dalej",
                primary: true,
            },
        ),
        Page::Options => layout::footer_with_cta(
            ui,
            footer,
            FooterActions {
                back_label: Some("Wstecz"),
                back: true,
                primary_label: "Zainstaluj",
                primary: true,
            },
        ),
        Page::Done => layout::footer_with_cta(
            ui,
            footer,
            FooterActions {
                back_label: None,
                back: false,
                primary_label: "Zakończ",
                primary: true,
            },
        ),
        Page::Error => layout::footer_with_cta(
            ui,
            footer,
            FooterActions {
                back_label: Some("Wstecz"),
                back: true,
                primary_label: "Zamknij",
                primary: true,
            },
        ),
        Page::Progress => return,
    };

    if back {
        match app.page {
            Page::Path => app.set_page(Page::Welcome),
            Page::Options => app.set_page(Page::Path),
            Page::Error => {
                app.set_page(if app.uninstall {
                    Page::Welcome
                } else {
                    Page::Options
                });
            }
            _ => {}
        }
    }
    if primary {
        match app.page {
            Page::Path => app.set_page(Page::Options),
            Page::Options => try_start_install(app, &ctx),
            Page::Done => {
                if !app.uninstall && app.launch_after {
                    install::launch_octra(&PathBuf::from(&app.dest));
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Page::Error => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            _ => {}
        }
    }
}

fn install_body(ui: &mut egui::Ui, app: &mut InstallerApp) {
    match app.page {
        Page::Welcome => welcome(ui, app),
        Page::Path => path_page(ui, app),
        Page::Options => options_page(ui, app),
        Page::Progress => progress_page(ui, app, false),
        Page::Done => done_page(ui, app, false),
        Page::Error => error_page(ui, app),
    }
}

fn uninstall_body(ui: &mut egui::Ui, app: &mut InstallerApp) {
    match app.page {
        Page::Progress => progress_page(ui, app, true),
        Page::Done => done_page(ui, app, true),
        Page::Error => error_page(ui, app),
        _ => uninstall_confirm(ui, app),
    }
}

fn page_header(ui: &mut egui::Ui, app: &InstallerApp, show_dots: bool) {
    if show_dots {
        let pulse = animation::pulse(ui.input(|i| i.time), 2.0);
        theme::step_dots(ui, app.step_index(), 4, pulse);
        ui.add_space(14.0);
    }
}

fn welcome(ui: &mut egui::Ui, app: &mut InstallerApp) {
    page_header(ui, app, true);
    ui.label(
        egui::RichText::new(format!("Witaj w {APP_NAME}"))
            .color(INK)
            .font(theme::title_font()),
    );
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new(format!(
            "Ten kreator zainstaluje {APP_NAME} na tym komputerze — szybko i bez zbędnej konfiguracji."
        ))
        .color(MUTE)
        .size(14.5),
    );
}

fn path_page(ui: &mut egui::Ui, app: &mut InstallerApp) {
    page_header(ui, app, true);
    ui.label(
        egui::RichText::new("Folder instalacji")
            .color(INK)
            .font(theme::title_font()),
    );
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new(format!(
            "{APP_NAME} zostanie zainstalowana w tym folderze. Możesz wybrać inną lokalizację."
        ))
        .color(MUTE)
        .size(14.5),
    );
    ui.add_space(16.0);
    theme::card(ui, |ui| {
        ui.label(egui::RichText::new("Lokalizacja").color(MUTE).size(12.0));
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let edit = egui::TextEdit::singleline(&mut app.dest)
                .desired_width(ui.available_width() - 110.0)
                .text_color(INK);
            ui.add(edit);
            if theme::ghost_button(ui, "Przeglądaj").clicked() {
                if let Some(dir) = rfd::FileDialog::new()
                    .set_directory(&app.dest)
                    .pick_folder()
                {
                    app.dest = dir.to_string_lossy().into_owned();
                }
            }
        });
        ui.add_space(8.0);
        if let Some(free) = install::disk_free_bytes(&PathBuf::from(&app.dest)) {
            ui.label(
                egui::RichText::new(format!(
                    "Wolne miejsce na dysku: {}",
                    install::format_bytes(free)
                ))
                .color(MUTE)
                .size(12.5),
            );
        }
    });
    ui.add_space(12.0);
    let was_all = app.all_users;
    theme::checkbox_with_sublabel(
        ui,
        &mut app.all_users,
        "Zainstaluj dla wszystkich użytkowników",
        "(wymaga uprawnień administratora)",
    );
    if app.all_users && !was_all {
        app.dest = crate::args::default_machine_dir()
            .to_string_lossy()
            .into_owned();
    } else if !app.all_users && was_all {
        app.dest = crate::args::default_user_dir()
            .to_string_lossy()
            .into_owned();
    }
}

fn options_page(ui: &mut egui::Ui, app: &mut InstallerApp) {
    page_header(ui, app, true);
    ui.label(egui::RichText::new("Skróty").color(INK).font(theme::title_font()));
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new(format!(
            "Wybierz, gdzie {APP_NAME} ma być widoczna po instalacji."
        ))
        .color(MUTE)
        .size(14.5),
    );
    ui.add_space(16.0);
    theme::card(ui, |ui| {
        theme::checkbox(ui, &mut app.start_menu, "Utwórz skrót w menu Start");
        ui.add_space(10.0);
        theme::checkbox(ui, &mut app.desktop, "Utwórz skrót na pulpicie");
    });
}

fn try_start_install(app: &mut InstallerApp, ctx: &egui::Context) {
    if (app.all_users || elevate::path_needs_admin(&PathBuf::from(&app.dest)))
        && !elevate::is_elevated()
    {
        let mut args = vec![
            "--elevated".into(),
            "--all-users".into(),
            "--dir".into(),
            app.dest.clone(),
        ];
        if app.start_menu {
            args.push("--start-menu".into());
        } else {
            args.push("--no-start-menu".into());
        }
        if app.desktop {
            args.push("--desktop".into());
        } else {
            args.push("--no-desktop".into());
        }
        match elevate::relaunch_elevated(&args) {
            Ok(()) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Err(e) => {
                app.error = Some(e);
                app.set_page(Page::Error);
            }
        }
        return;
    }
    app.begin_install();
}

fn progress_page(ui: &mut egui::Ui, app: &mut InstallerApp, uninstall: bool) {
    page_header(ui, app, !uninstall);
    ui.label(
        egui::RichText::new(if uninstall {
            "Trwa usuwanie".to_string()
        } else {
            format!("Instalowanie {APP_NAME}")
        })
        .color(INK)
        .font(theme::title_font()),
    );
    ui.add_space(8.0);
    ui.label(egui::RichText::new(&app.status).color(MUTE).size(14.0));
    ui.add_space(16.0);

    if !uninstall {
        let spin = ui.input(|i| i.time) as f32;
        for (kind, state) in app.install_step_states() {
            theme::install_step_row(ui, state, kind.label(), spin);
            ui.add_space(4.0);
        }
        ui.add_space(12.0);
    }

    theme::progress_bar_animated(ui, app.display_progress);
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new(format!(
            "{}%",
            (app.display_progress * 100.0).round() as i32
        ))
        .color(ACCENT)
        .font(theme::semibold()),
    );
}

fn done_page(ui: &mut egui::Ui, app: &mut InstallerApp, uninstall: bool) {
    let pulse = animation::pulse(ui.input(|i| i.time), 1.4);
    theme::success_check_animated(ui, app.success_anim_t, pulse);
    ui.add_space(12.0);
    ui.label(
        egui::RichText::new(if uninstall {
            format!("{APP_NAME} została usunięta")
        } else {
            "Instalacja zakończona".to_string()
        })
        .color(INK)
        .font(theme::title_font()),
    );
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new(if uninstall {
            "Program został usunięty z tego komputera.".to_string()
        } else {
            format!("{APP_NAME} jest gotowa do użycia.")
        })
        .color(MUTE)
        .size(14.5),
    );
    if !uninstall && !app.webview_ok {
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new(
                "Przy pierwszym uruchomieniu system może poprosić o składnik Microsoft Edge WebView2 — warto go zainstalować.",
            )
            .color(MUTE)
            .size(13.0),
        );
    }
    if !uninstall {
        ui.add_space(14.0);
        theme::checkbox(ui, &mut app.launch_after, &format!("Uruchom {APP_NAME}"));
    }
}

fn error_page(ui: &mut egui::Ui, app: &mut InstallerApp) {
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new("Nie udało się dokończyć")
            .color(INK)
            .font(theme::title_font()),
    );
    ui.add_space(12.0);
    theme::card(ui, |ui| {
        ui.label(
            egui::RichText::new(app.error.as_deref().unwrap_or("Wystąpił nieoczekiwany błąd."))
                .color(DANGER)
                .size(14.0),
        );
    });
}

fn uninstall_confirm(ui: &mut egui::Ui, app: &mut InstallerApp) {
    ui.label(
        egui::RichText::new(format!("Usunąć {APP_NAME}?"))
            .color(INK)
            .font(theme::title_font()),
    );
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new(
            "Program zostanie usunięty z tego komputera. Instancje i zapisy gry pozostaną, chyba że zaznaczysz ich usunięcie.",
        )
        .color(MUTE)
        .size(14.5),
    );
    ui.add_space(16.0);
    theme::card(ui, |ui| {
        ui.label(
            egui::RichText::new(format!("Folder: {}", app.dest))
                .color(MUTE)
                .size(13.0),
        );
        ui.add_space(12.0);
        theme::checkbox(
            ui,
            &mut app.remove_data,
            "Usuń także dane gier i instancji",
        );
    });
}

fn active_step_from_status(status: &str, progress: f32) -> StepKind {
    let s = status.to_ascii_lowercase();
    if s.contains("kończ") || s.contains("gotowe") || s.contains("zapisywanie deinstalatora") {
        StepKind::Finalize
    } else if s.contains("skrót") {
        StepKind::Shortcuts
    } else if s.contains("środowisk") || s.contains("webview") {
        StepKind::Runtime
    } else if s.contains("kopiow") || s.contains("rozpak") {
        StepKind::Copy
    } else if progress >= 0.97 {
        StepKind::Finalize
    } else if progress >= 0.86 {
        StepKind::Runtime
    } else if progress >= 0.08 {
        StepKind::Copy
    } else {
        StepKind::Prepare
    }
}

fn run_install_job(req: InstallRequest, tx: Sender<JobEvent>) {
    let send_p = |p: f32, s: &str| {
        let _ = tx.send(JobEvent::Progress(p, s.to_string()));
    };
    match install::run_install(&req, send_p) {
        Ok(r) => {
            let _ = tx.send(JobEvent::Ok {
                webview_ok: r.webview_ok,
            });
        }
        Err(e) => {
            let _ = tx.send(JobEvent::Err(e));
        }
    }
}

fn run_uninstall_job(req: UninstallRequest, tx: Sender<JobEvent>) {
    let send_p = |p: f32, s: &str| {
        let _ = tx.send(JobEvent::Progress(p, s.to_string()));
    };
    match install::run_uninstall(&req, send_p) {
        Ok(()) => {
            let _ = tx.send(JobEvent::Ok { webview_ok: true });
        }
        Err(e) => {
            let _ = tx.send(JobEvent::Err(e));
        }
    }
}
