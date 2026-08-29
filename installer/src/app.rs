use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use eframe::egui::{self, Align, Color32, Layout, PointerButton, Sense};

use crate::args::InstallPreset;
use crate::elevate;
use crate::install::{self, InstallRequest, UninstallRequest};
use crate::theme::{self, ACCENT, BG, DANGER, INK, LINE, MUTE};

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

pub struct InstallerApp {
    uninstall: bool,
    page: Page,
    dest: String,
    start_menu: bool,
    desktop: bool,
    all_users: bool,
    launch_after: bool,
    remove_data: bool,
    progress: f32,
    status: String,
    error: Option<String>,
    webview_ok: bool,
    fonts_ready: bool,
    auto_close: bool,
    close_now: bool,
    rx: Option<Receiver<JobEvent>>,
}

impl InstallerApp {
    pub fn new(cc: &eframe::CreationContext<'_>, uninstall: bool, preset: InstallPreset) -> Self {
        theme::apply(&cc.egui_ctx);
        let dest = preset
            .dest
            .unwrap_or_else(|| install::default_dest(preset.all_users));
        let mut app = Self {
            uninstall,
            page: if uninstall { Page::Welcome } else { Page::Welcome },
            dest: dest.to_string_lossy().into_owned(),
            start_menu: preset.start_menu,
            desktop: preset.desktop,
            all_users: preset.all_users,
            launch_after: true,
            remove_data: false,
            progress: 0.0,
            status: String::new(),
            error: None,
            webview_ok: true,
            fonts_ready: false,
            auto_close: false,
            close_now: false,
            rx: None,
        };
        if preset.all_users && uninstall {
            app.page = Page::Welcome;
        }
        app
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
        self.page = Page::Progress;
        self.progress = 0.0;
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
        self.page = Page::Progress;
        self.progress = 0.0;
        self.status = "Usuwanie…".into();
        thread::spawn(move || run_uninstall_job(req, tx));
    }

    fn pump(&mut self) {
        let Some(rx) = &self.rx else { return };
        while let Ok(ev) = rx.try_recv() {
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
                        self.page = Page::Done;
                    }
                }
                JobEvent::Err(e) => {
                    self.error = Some(e);
                    self.page = Page::Error;
                }
            }
        }
    }
}

impl eframe::App for InstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.fonts_ready {
            theme::apply(ctx);
            self.fonts_ready = true;
        }
        self.pump();
        if self.close_now {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ctx.request_repaint();

        let screen = ctx.screen_rect();
        let painter = ctx.layer_painter(egui::LayerId::background());
        painter.rect_filled(screen, 0.0, BG);
        painter.circle_filled(
            egui::pos2(screen.max.x - 40.0, 8.0),
            220.0,
            Color32::from_rgba_unmultiplied(0xc4, 0xa7, 0xff, 18),
        );
        painter.circle_filled(
            egui::pos2(screen.min.x + 80.0, screen.max.y + 20.0),
            180.0,
            Color32::from_rgba_unmultiplied(125, 211, 252, 10),
        );

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                titlebar(ui, self.uninstall);
                ui.add_space(4.0);
                let body = ui.available_rect_before_wrap();
                ui.allocate_new_ui(
                    egui::UiBuilder::new().max_rect(body.shrink2(egui::vec2(36.0, 18.0))),
                    |ui| {
                        if self.uninstall {
                            uninstall_body(ui, self);
                        } else {
                            install_body(ui, self);
                        }
                    },
                );
            });
    }
}

fn titlebar(ui: &mut egui::Ui, uninstall: bool) {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 44.0), Sense::drag());
    if resp.dragged_by(PointerButton::Primary) {
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.vertical(|ui| {
                ui.add_space(10.0);
                theme::mark(ui, 22.0);
            });
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new(if uninstall {
                        "Octra — deinstalacja"
                    } else {
                        "Octra Launcher"
                    })
                    .color(INK)
                    .font(theme::semibold()),
                );
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if theme::icon_hit(ui, "×", true).clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if theme::icon_hit(ui, "–", false).clicked() {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }
            });
        });
    });
    let y = rect.bottom();
    ui.painter().hline(
        rect.x_range(),
        y,
        theme::hairline(LINE),
    );
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

fn steps(ui: &mut egui::Ui, current: u32) {
    ui.horizontal(|ui| {
        for i in 0..4 {
            let on = i <= current;
            let (r, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), Sense::hover());
            ui.painter().circle_filled(r.center(), 4.0, if on { ACCENT } else { LINE });
            if i < 3 {
                let (line, _) = ui.allocate_exact_size(egui::vec2(18.0, 8.0), Sense::hover());
                ui.painter().hline(
                    line.x_range(),
                    line.center().y,
                    theme::hairline(LINE),
                );
            }
        }
    });
    ui.add_space(18.0);
}

fn welcome(ui: &mut egui::Ui, app: &mut InstallerApp) {
    steps(ui, 0);
    ui.label(egui::RichText::new("Witamy").color(INK).font(theme::title_font()));
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new(
            "Ten kreator zainstaluje Octra Launcher na tym komputerze. Instalacja nie wymaga uprawnień administratora.",
        )
        .color(MUTE)
        .size(14.5),
    );
    ui.add_space(20.0);
    theme::card(ui, |ui| {
        feature(ui, "Instancje i paczki", "Vanilla, Fabric, Forge i gotowe zestawy modów w jednym miejscu.");
        ui.add_space(10.0);
        feature(ui, "Java bez konfiguracji", "Octra dobierze środowisko uruchomieniowe, gdy będzie potrzebne.");
        ui.add_space(10.0);
        feature(ui, "Konto Microsoft", "Logowanie tym samym kontem, którego używasz do Minecraft Java Edition.");
    });
    footer(ui, |ui| {
        if theme::ghost_button(ui, "Anuluj").clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ui.add_space(10.0);
        if theme::accent_button(ui, "Dalej").clicked() {
            app.page = Page::Path;
        }
    });
}

fn feature(ui: &mut egui::Ui, title: &str, body: &str) {
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), Sense::hover());
        ui.painter().circle_filled(r.center(), 3.5, ACCENT);
        ui.add_space(6.0);
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(title).color(INK).font(theme::semibold()));
            ui.label(egui::RichText::new(body).color(MUTE).size(13.0));
        });
    });
}

fn path_page(ui: &mut egui::Ui, app: &mut InstallerApp) {
    steps(ui, 1);
    ui.label(
        egui::RichText::new("Folder instalacji")
            .color(INK)
            .font(theme::title_font()),
    );
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new(
            "Octra zostanie zainstalowana w tym folderze. Możesz wybrać inną lokalizację.",
        )
        .color(MUTE)
        .size(14.5),
    );
    ui.add_space(18.0);
    theme::card(ui, |ui| {
        ui.label(egui::RichText::new("Lokalizacja").color(MUTE).size(12.0));
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let edit = egui::TextEdit::singleline(&mut app.dest)
                .desired_width(ui.available_width() - 130.0)
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
        ui.add_space(10.0);
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
    ui.add_space(14.0);
    let was_all = app.all_users;
    theme::checkbox(
        ui,
        &mut app.all_users,
        "Zainstaluj dla wszystkich użytkowników (wymaga uprawnień administratora)",
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
    footer(ui, |ui| {
        if theme::ghost_button(ui, "Wstecz").clicked() {
            app.page = Page::Welcome;
        }
        ui.add_space(10.0);
        if theme::accent_button(ui, "Dalej").clicked() {
            app.page = Page::Options;
        }
    });
}

fn options_page(ui: &mut egui::Ui, app: &mut InstallerApp) {
    steps(ui, 2);
    ui.label(egui::RichText::new("Skróty").color(INK).font(theme::title_font()));
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new("Wybierz, gdzie Octra ma być widoczny po instalacji.")
            .color(MUTE)
            .size(14.5),
    );
    ui.add_space(18.0);
    theme::card(ui, |ui| {
        theme::checkbox(ui, &mut app.start_menu, "Utwórz skrót w menu Start");
        ui.add_space(10.0);
        theme::checkbox(ui, &mut app.desktop, "Utwórz skrót na pulpicie");
    });
    footer(ui, |ui| {
        if theme::ghost_button(ui, "Wstecz").clicked() {
            app.page = Page::Path;
        }
        ui.add_space(10.0);
        if theme::accent_button(ui, "Zainstaluj").clicked() {
            try_start_install(app, ui.ctx());
        }
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
                app.page = Page::Error;
            }
        }
        return;
    }
    app.begin_install();
}

fn progress_page(ui: &mut egui::Ui, app: &mut InstallerApp, uninstall: bool) {
    steps(ui, 3);
    ui.label(
        egui::RichText::new(if uninstall {
            "Trwa usuwanie"
        } else {
            "Instalowanie Octra"
        })
        .color(INK)
        .font(theme::title_font()),
    );
    ui.add_space(8.0);
    ui.label(egui::RichText::new(&app.status).color(MUTE).size(14.5));
    ui.add_space(22.0);
    theme::progress_bar(ui, app.progress);
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new(format!("{}%", (app.progress * 100.0).round() as i32))
            .color(ACCENT)
            .font(theme::semibold()),
    );
}

fn done_page(ui: &mut egui::Ui, app: &mut InstallerApp, uninstall: bool) {
    ui.add_space(12.0);
    ui.label(
        egui::RichText::new(if uninstall {
            "Octra została usunięta"
        } else {
            "Instalacja zakończona"
        })
        .color(INK)
        .font(theme::title_font()),
    );
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new(if uninstall {
            "Program został usunięty z tego komputera."
        } else {
            "Octra jest gotowa do użycia."
        })
        .color(MUTE)
        .size(14.5),
    );
    if !uninstall && !app.webview_ok {
        ui.add_space(12.0);
        ui.label(
            egui::RichText::new(
                "Przy pierwszym uruchomieniu system może poprosić o składnik Microsoft Edge WebView2 — warto go zainstalować.",
            )
            .color(MUTE)
            .size(13.0),
        );
    }
    if !uninstall {
        ui.add_space(18.0);
        theme::checkbox(ui, &mut app.launch_after, "Uruchom Octra");
    }
    footer(ui, |ui| {
        if theme::accent_button(ui, "Zakończ").clicked() {
            if !uninstall && app.launch_after {
                install::launch_octra(&PathBuf::from(&app.dest));
            }
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    });
}

fn error_page(ui: &mut egui::Ui, app: &mut InstallerApp) {
    ui.add_space(12.0);
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
    footer(ui, |ui| {
        if theme::ghost_button(ui, "Wstecz").clicked() {
            app.page = if app.uninstall {
                Page::Welcome
            } else {
                Page::Options
            };
        }
        ui.add_space(10.0);
        if theme::accent_button(ui, "Zamknij").clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    });
}

fn uninstall_confirm(ui: &mut egui::Ui, app: &mut InstallerApp) {
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new("Usunąć Octra?")
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
    ui.add_space(18.0);
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
    footer(ui, |ui| {
        if theme::ghost_button(ui, "Anuluj").clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ui.add_space(10.0);
        if theme::accent_button(ui, "Usuń").clicked() {
            app.begin_uninstall();
        }
    });
}

fn footer(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    ui.with_layout(Layout::bottom_up(Align::RIGHT), |ui| {
        ui.add_space(8.0);
        ui.horizontal(|ui| add(ui));
        ui.add_space(12.0);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), Sense::hover());
        ui.painter()
            .rect_filled(rect, 0.0, LINE);
    });
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
