use std::sync::Arc;

use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Shadow, Stroke, Style,
    TextStyle, Visuals,
};

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
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
    let dir = std::path::PathBuf::from(windir).join("Fonts");
    if let Ok(regular) = std::fs::read(dir.join("segoeui.ttf")) {
        fonts
            .font_data
            .insert("segoe".into(), Arc::new(FontData::from_owned(regular)));
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "segoe".into());
    }
    if let Ok(bold) = std::fs::read(dir.join("segoeuib.ttf")) {
        fonts
            .font_data
            .insert("segoe-bold".into(), Arc::new(FontData::from_owned(bold)));
        fonts.families.insert(
            FontFamily::Name("semibold".into()),
            vec!["segoe-bold".into(), "segoe".into()],
        );
    }
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
    v.widgets.hovered.bg_fill = Color32::from_rgb(0x22, 0x28, 0x36);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x22, 0x28, 0x36);
    v.widgets.active.bg_fill = RAISED;
    v.selection.bg_fill = Color32::from_rgba_unmultiplied(0xc4, 0xa7, 0xff, 60);
    v.hyperlink_color = ACCENT;
    ctx.set_visuals(v);

    let mut style = Style::default();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(16.0, 8.0);
    style.spacing.window_margin = egui::Margin::ZERO;
    style.visuals = ctx.style().visuals.clone();
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(26.0, FontFamily::Name("semibold".into())),
    );
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(15.0, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(14.0, FontFamily::Name("semibold".into())),
    );
    style
        .text_styles
        .insert(TextStyle::Small, FontId::new(12.5, FontFamily::Proportional));
    ctx.set_style(style);
}

pub fn semibold() -> FontId {
    FontId::new(15.0, FontFamily::Name("semibold".into()))
}

pub fn title_font() -> FontId {
    FontId::new(28.0, FontFamily::Name("semibold".into()))
}

pub fn accent_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let w = 128.0;
    let h = 40.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::click());
    let fill = if resp.is_pointer_button_down_on() {
        ACCENT_DIM
    } else if resp.hovered() {
        Color32::from_rgb(0xce, 0xb6, 0xff)
    } else {
        ACCENT
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(20), fill);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::new(14.0, FontFamily::Name("semibold".into())),
        ON_ACCENT,
    );
    resp
}

pub fn ghost_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let w = 110.0;
    let h = 40.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::click());
    let fill = if resp.hovered() {
        Color32::from_rgba_unmultiplied(255, 255, 255, 12)
    } else {
        Color32::TRANSPARENT
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(20), fill);
    ui.painter()
        .rect_stroke(rect, CornerRadius::same(20), hairline(LINE), egui::StrokeKind::Inside);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::new(14.0, FontFamily::Name("semibold".into())),
        INK,
    );
    resp
}

pub fn icon_hit(ui: &mut egui::Ui, label: &str, danger: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(42.0, 40.0), egui::Sense::click());
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
        FontId::new(16.0, FontFamily::Proportional),
        if danger && resp.hovered() {
            Color32::WHITE
        } else {
            MUTE
        },
    );
    resp
}

pub fn checkbox(ui: &mut egui::Ui, on: &mut bool, label: &str) -> egui::Response {
    ui.horizontal(|ui| {
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::click());
        if resp.clicked() {
            *on = !*on;
        }
        ui.painter()
            .rect_filled(rect, CornerRadius::same(6), RAISED2);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(6),
            hairline(if *on { ACCENT } else { LINE }),
            egui::StrokeKind::Inside,
        );
        if *on {
            ui.painter().rect_filled(
                rect.shrink(4.0),
                CornerRadius::same(3),
                ACCENT,
            );
        }
        ui.add_space(8.0);
        ui.label(egui::RichText::new(label).color(INK).size(14.0));
        resp
    })
    .inner
}

pub fn progress_bar(ui: &mut egui::Ui, t: f32) {
    let desired = egui::vec2(ui.available_width(), 8.0);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(4), RAISED2);
    let w = (rect.width() * t.clamp(0.0, 1.0)).max(if t > 0.0 { 8.0 } else { 0.0 });
    if w > 0.0 {
        let mut fill = rect;
        fill.max.x = fill.min.x + w;
        ui.painter()
            .rect_filled(fill, CornerRadius::same(4), ACCENT);
    }
}

pub fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(RAISED)
        .stroke(hairline(LINE))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(egui::Margin::same(18))
        .show(ui, add);
}

pub fn mark(ui: &mut egui::Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same((size * 0.28) as u8), ACCENT);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "L",
        FontId::new(size * 0.52, FontFamily::Name("semibold".into())),
        BG,
    );
}
