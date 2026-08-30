use std::sync::Arc;

use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Shadow, Stroke, Style,
    TextStyle, Visuals,
};

const FIGTREE_400: &[u8] = include_bytes!("../assets/fonts/figtree-400.ttf");
const FIGTREE_600: &[u8] = include_bytes!("../assets/fonts/figtree-600.ttf");

pub const BG: Color32 = Color32::from_rgb(0x06, 0x03, 0x05);
pub const RAISED: Color32 = Color32::from_rgb(0x12, 0x0c, 0x14);
pub const RAISED2: Color32 = Color32::from_rgb(0x1a, 0x12, 0x1e);
pub const INK: Color32 = Color32::from_rgb(0xd2, 0xd2, 0xd2);
pub const MUTE: Color32 = Color32::from_rgb(0x8a, 0x82, 0x8c);
pub const ACCENT: Color32 = Color32::from_rgb(0xa0, 0x51, 0xa2);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(0x7a, 0x3d, 0x7c);
pub const DANGER: Color32 = Color32::from_rgb(0xfb, 0x71, 0x85);
pub const ON_ACCENT: Color32 = Color32::from_rgb(0x12, 0x08, 0x1c);
pub const LINE: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 20);

pub fn hairline(color: Color32) -> Stroke {
    Stroke::new(1.0_f32, color)
}

pub fn apply(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "figtree".into(),
        Arc::new(FontData::from_static(FIGTREE_400)),
    );
    fonts.font_data.insert(
        "figtree-semibold".into(),
        Arc::new(FontData::from_static(FIGTREE_600)),
    );
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "figtree".into());
    fonts.families.insert(
        FontFamily::Name("semibold".into()),
        vec![
            "figtree-semibold".into(),
            "figtree".into(),
            "Ubuntu-Light".into(),
        ],
    );
    ctx.set_fonts(fonts);

    let mut v = Visuals::dark();
    v.panel_fill = BG;
    v.window_fill = BG;
    v.extreme_bg_color = RAISED;
    v.faint_bg_color = RAISED2;
    v.override_text_color = Some(INK);
    v.window_stroke = Stroke::NONE;
    v.window_shadow = Shadow::NONE;
    v.popup_shadow = Shadow::NONE;
    v.widgets.noninteractive.fg_stroke = hairline(INK);
    v.widgets.inactive.bg_fill = RAISED2;
    v.widgets.inactive.weak_bg_fill = RAISED2;
    v.widgets.inactive.fg_stroke = hairline(INK);
    v.widgets.hovered.bg_fill = Color32::from_rgb(0x22, 0x18, 0x28);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x22, 0x18, 0x28);
    v.widgets.active.bg_fill = RAISED;
    v.selection.bg_fill = Color32::from_rgba_unmultiplied(160, 81, 162, 60);
    v.hyperlink_color = ACCENT;
    ctx.set_visuals(v);

    let mut style = Style::default();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(16.0, 10.0);
    style.spacing.window_margin = egui::Margin::ZERO;
    style.visuals = ctx.style().visuals.clone();
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(24.0, FontFamily::Name("semibold".into())),
    );
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(14.5, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(14.0, FontFamily::Name("semibold".into())),
    );
    style
        .text_styles
        .insert(TextStyle::Small, FontId::new(12.5, FontFamily::Proportional));
    style.interaction.selectable_labels = false;
    ctx.set_style(style);
}

/// Block browser-like shortcuts; allow paste only when a text field has focus.
pub fn block_editor_shortcuts(ctx: &egui::Context) {
    let allow_paste = ctx.wants_keyboard_input();
    ctx.input_mut(|i| {
        if !i.modifiers.ctrl {
            return;
        }
        for key in [egui::Key::A, egui::Key::C, egui::Key::X] {
            if i.key_pressed(key) {
                i.consume_key(egui::Modifiers::CTRL, key);
            }
        }
        if !allow_paste && i.key_pressed(egui::Key::V) {
            i.consume_key(egui::Modifiers::CTRL, egui::Key::V);
        }
    });
}

pub fn semibold() -> FontId {
    FontId::new(14.5, FontFamily::Name("semibold".into()))
}

pub fn title_font() -> FontId {
    FontId::new(22.0, FontFamily::Name("semibold".into()))
}

pub fn primary_cta(ui: &mut egui::Ui, label: &str, width: f32, height: f32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());
    let fill = if resp.is_pointer_button_down_on() {
        ACCENT_DIM
    } else if resp.hovered() {
        Color32::from_rgb(0xb8, 0x62, 0xba)
    } else {
        ACCENT
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(12), fill);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::new(14.0, FontFamily::Name("semibold".into())),
        ON_ACCENT,
    );
    resp
}

pub fn primary_cta_full_width(ui: &mut egui::Ui, label: &str) -> egui::Response {
    primary_cta(ui, label, ui.available_width(), 44.0)
}

pub fn primary_cta_disabled(ui: &mut egui::Ui, label: &str, width: f32, height: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(12), RAISED2);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::new(14.0, FontFamily::Name("semibold".into())),
        MUTE,
    );
}

pub fn ghost_button_sized(ui: &mut egui::Ui, label: &str, width: f32, height: f32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());
    let fill = if resp.hovered() {
        Color32::from_rgba_unmultiplied(255, 255, 255, 12)
    } else {
        Color32::TRANSPARENT
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(10), fill);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::new(13.0, FontFamily::Name("semibold".into())),
        INK,
    );
    resp
}

pub fn ghost_button_disabled(ui: &mut egui::Ui, label: &str, width: f32, height: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::new(13.0, FontFamily::Name("semibold".into())),
        MUTE,
    );
}

pub fn ghost_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let w = label.chars().count() as f32 * 7.5 + 32.0;
    ghost_button_sized(ui, label, w.max(88.0), 34.0)
}

pub fn icon_hit(ui: &mut egui::Ui, label: &str, danger: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(36.0, TITLEBAR_HIT), egui::Sense::click());
    let fill = if danger && resp.hovered() {
        DANGER
    } else if resp.hovered() {
        Color32::from_rgba_unmultiplied(255, 255, 255, 14)
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, CornerRadius::ZERO, fill);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::new(15.0, FontFamily::Proportional),
        if danger && resp.hovered() {
            Color32::WHITE
        } else {
            MUTE
        },
    );
    resp
}

const TITLEBAR_HIT: f32 = 36.0;

pub fn step_dots(ui: &mut egui::Ui, current: u32, total: u32, pulse: f32) {
    ui.horizontal(|ui| {
        for i in 0..total {
            let active = i == current;
            let done = i < current;
            let radius = if active { 5.0 + pulse * 1.5 } else { 4.0 };
            let color = if done || active {
                ACCENT
            } else {
                LINE
            };
            let (r, _) = ui.allocate_exact_size(egui::vec2(radius * 2.0 + 4.0, 12.0), egui::Sense::hover());
            ui.painter().circle_filled(r.center(), radius, color);
            if active {
                ui.painter().circle_stroke(
                    r.center(),
                    radius + 3.0 + pulse * 2.0,
                    hairline(Color32::from_rgba_unmultiplied(160, 81, 162, (80.0 + pulse * 80.0) as u8)),
                );
            }
            if i + 1 < total {
                let (line, _) = ui.allocate_exact_size(egui::vec2(14.0, 12.0), egui::Sense::hover());
                ui.painter().hline(
                    line.x_range(),
                    line.center().y,
                    hairline(if done { ACCENT } else { LINE }),
                );
            }
        }
    });
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InstallStepVisual {
    Pending,
    Active,
    Done,
}

pub fn install_step_row(ui: &mut egui::Ui, state: InstallStepVisual, label: &str, spin: f32) {
    ui.horizontal(|ui| {
        let (icon, _) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
        let c = icon.center();
        match state {
            InstallStepVisual::Done => {
                ui.painter().circle_filled(c, 9.0, ACCENT);
                ui.painter().text(
                    c,
                    egui::Align2::CENTER_CENTER,
                    "✓",
                    FontId::new(11.0, FontFamily::Proportional),
                    ON_ACCENT,
                );
            }
            InstallStepVisual::Active => {
                ui.painter().circle_stroke(c, 8.0, hairline(ACCENT));
                let a = spin * std::f32::consts::TAU;
                let p = c + egui::vec2(a.cos(), a.sin()) * 5.0;
                ui.painter().circle_filled(p, 2.0, ACCENT);
            }
            InstallStepVisual::Pending => {
                ui.painter().circle_stroke(c, 7.0, hairline(LINE));
            }
        }
        ui.add_space(6.0);
        let color = match state {
            InstallStepVisual::Pending => MUTE,
            InstallStepVisual::Active => INK,
            InstallStepVisual::Done => INK,
        };
        ui.label(
            egui::RichText::new(label)
                .color(color)
                .size(if state == InstallStepVisual::Active {
                    14.5
                } else {
                    14.0
                }),
        );
    });
}

pub fn progress_bar_animated(ui: &mut egui::Ui, t: f32) {
    let desired = egui::vec2(ui.available_width(), 8.0);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(4), RAISED2);
    let w = (rect.width() * t.clamp(0.0, 1.0)).max(if t > 0.0 { 6.0 } else { 0.0 });
    if w > 0.0 {
        let mut fill = rect;
        fill.max.x = fill.min.x + w;
        ui.painter()
            .rect_filled(fill, CornerRadius::same(4), ACCENT);
    }
}

pub fn success_check_animated(ui: &mut egui::Ui, t: f32, pulse: f32) -> egui::Response {
    let size = 72.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let scale = 0.6 + 0.4 * t;
    let r = rect.center();
    let radius = (size * 0.5 - 4.0) * scale;
    ui.painter().circle_stroke(
        r,
        radius + 6.0 + pulse * 8.0,
        hairline(Color32::from_rgba_unmultiplied(160, 81, 162, (40.0 + pulse * 60.0) as u8)),
    );
    ui.painter().circle_filled(r, radius, ACCENT);
    if t > 0.35 {
        ui.painter().text(
            r,
            egui::Align2::CENTER_CENTER,
            "✓",
            FontId::new(radius * 0.9, FontFamily::Proportional),
            ON_ACCENT,
        );
    }
    resp
}

fn paint_checkbox_box(ui: &mut egui::Ui, on: bool, rect: egui::Rect) {
    ui.painter()
        .rect_filled(rect, CornerRadius::same(6), RAISED2);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(6),
        hairline(if on { ACCENT } else { LINE }),
        egui::StrokeKind::Inside,
    );
    if on {
        ui.painter()
            .rect_filled(rect.shrink(4.0), CornerRadius::same(3), ACCENT);
    }
}

pub fn checkbox(ui: &mut egui::Ui, on: &mut bool, label: &str) -> egui::Response {
    ui.horizontal(|ui| {
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::click());
        if resp.clicked() {
            *on = !*on;
        }
        paint_checkbox_box(ui, *on, rect);
        ui.add_space(8.0);
        ui.label(egui::RichText::new(label).color(INK).size(14.0));
        resp
    })
    .inner
}

pub fn checkbox_with_sublabel(
    ui: &mut egui::Ui,
    on: &mut bool,
    label: &str,
    sublabel: &str,
) -> egui::Response {
    ui.horizontal_top(|ui| {
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::click());
        if resp.clicked() {
            *on = !*on;
        }
        paint_checkbox_box(ui, *on, rect);
        ui.add_space(8.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(egui::RichText::new(label).color(INK).size(14.0));
            ui.label(egui::RichText::new(sublabel).color(MUTE).size(12.5));
        });
        resp
    })
    .inner
}

pub fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(RAISED)
        .stroke(hairline(LINE))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(egui::Margin::same(16))
        .show(ui, add);
}

pub fn mark(ui: &mut egui::Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same((size * 0.28) as u8), ACCENT);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "O",
        FontId::new(size * 0.52, FontFamily::Name("semibold".into())),
        BG,
    );
}

// Legacy aliases kept for any remaining call sites.
pub fn accent_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    primary_cta_full_width(ui, label)
}

pub fn progress_bar(ui: &mut egui::Ui, t: f32) {
    progress_bar_animated(ui, t);
}
