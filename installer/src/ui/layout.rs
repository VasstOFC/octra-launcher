use eframe::egui::{self, Align, Layout, PointerButton, Sense, Vec2};

use crate::theme::{self, LINE};
use crate::ui::left_panel;
use crate::ui::APP_NAME;

pub const LEFT_WIDTH: f32 = 280.0;
pub const RIGHT_WIDTH: f32 = 440.0;
pub const TITLEBAR_HEIGHT: f32 = 36.0;
pub const FOOTER_HEIGHT: f32 = 72.0;
pub const FOOTER_BTN_HEIGHT: f32 = 44.0;
pub const FOOTER_BTN_GAP: f32 = 10.0;
pub const CONTENT_H_PAD: f32 = 28.0;
pub const CONTENT_V_PAD: f32 = 16.0;
pub const CONTENT_FOOTER_GAP: f32 = 16.0;

pub struct SplitShell<'a> {
    pub right: egui::Rect,
    pub uninstall: bool,
    pub version: &'a str,
}

pub fn split_shell(
    ui: &mut egui::Ui,
    uninstall: bool,
    version: &str,
    draw_right: impl FnOnce(&mut egui::Ui, egui::Rect),
) {
    let full = ui.available_rect_before_wrap();
    let left_rect = egui::Rect::from_min_size(full.min, Vec2::new(LEFT_WIDTH, full.height()));
    let right_rect = egui::Rect::from_min_max(
        egui::pos2(full.min.x + LEFT_WIDTH, full.min.y),
        full.max,
    );

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(left_rect), |ui| {
        left_panel::paint(ui, uninstall, version);
    });

    ui.painter().vline(
        right_rect.min.x,
        right_rect.y_range(),
        theme::hairline(LINE),
    );

    draw_right(ui, right_rect);
}

pub fn titlebar(ui: &mut egui::Ui, uninstall: bool) {
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), TITLEBAR_HEIGHT),
        Sense::drag(),
    );
    if resp.dragged_by(PointerButton::Primary) {
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            ui.add_space(12.0);
            theme::mark(ui, 20.0);
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(if uninstall {
                    format!("{APP_NAME} — deinstalacja")
                } else {
                    APP_NAME.to_string()
                })
                .color(theme::INK)
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

    ui.painter().hline(
        rect.x_range(),
        rect.bottom(),
        theme::hairline(LINE),
    );
}

pub struct FooterActions<'a> {
    pub back_label: Option<&'a str>,
    pub back: bool,
    pub primary_label: &'a str,
    pub primary: bool,
}

pub fn footer_with_cta(ui: &mut egui::Ui, rect: egui::Rect, actions: FooterActions<'_>) -> (bool, bool) {
    let mut back_clicked = false;
    let mut primary_clicked = false;

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
            ui.set_width(rect.width());
            ui.add_space(4.0);
            let (line, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
            ui.painter().rect_filled(line, 0.0, LINE);
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = FOOTER_BTN_GAP;
                let total_w = ui.available_width();
                let has_back = actions.back_label.is_some();
                let (back_w, primary_w) = if has_back {
                    let inner = total_w - FOOTER_BTN_GAP;
                    (inner * 0.4, inner * 0.6)
                } else {
                    (0.0, total_w)
                };

                if let Some(label) = actions.back_label {
                    if actions.back {
                        if theme::ghost_button_sized(ui, label, back_w, FOOTER_BTN_HEIGHT).clicked() {
                            back_clicked = true;
                        }
                    } else {
                        theme::ghost_button_disabled(ui, label, back_w, FOOTER_BTN_HEIGHT);
                    }
                }

                if actions.primary {
                    if theme::primary_cta(ui, actions.primary_label, primary_w, FOOTER_BTN_HEIGHT).clicked() {
                        primary_clicked = true;
                    }
                } else {
                    theme::primary_cta_disabled(ui, actions.primary_label, primary_w, FOOTER_BTN_HEIGHT);
                }
            });

            ui.add_space(8.0);
        });
    });

    (back_clicked, primary_clicked)
}

pub fn right_content_rect(body: egui::Rect) -> (egui::Rect, egui::Rect) {
    let inner = body.shrink2(Vec2::new(CONTENT_H_PAD, CONTENT_V_PAD));
    let footer = egui::Rect::from_min_max(
        egui::pos2(inner.min.x, inner.max.y - FOOTER_HEIGHT),
        inner.max,
    );
    let content = egui::Rect::from_min_max(
        inner.min,
        egui::pos2(inner.max.x, footer.min.y - CONTENT_FOOTER_GAP),
    );
    (content, footer)
}
