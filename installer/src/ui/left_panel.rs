use eframe::egui::{self, Color32, CornerRadius, Pos2, Rect, Sense, Vec2};

use crate::theme::{ACCENT, INK, MUTE, RAISED2};
use crate::ui::APP_NAME;

const BLOCK: f32 = 10.0;

#[derive(Clone, Copy)]
enum Block {
    Air,
    Grass,
    Dirt,
    Stone,
    Deepslate,
    Accent,
    Trunk,
    Leaf,
}

impl Block {
    fn color(self) -> Color32 {
        match self {
            Block::Air => Color32::TRANSPARENT,
            Block::Grass => Color32::from_rgb(0x5a, 0x9a, 0x3c),
            Block::Dirt => Color32::from_rgb(0x8b, 0x5a, 0x2b),
            Block::Stone => Color32::from_rgb(0x7a, 0x7a, 0x7a),
            Block::Deepslate => Color32::from_rgb(0x3a, 0x3f, 0x48),
            Block::Accent => ACCENT,
            Block::Trunk => Color32::from_rgb(0x5c, 0x3d, 0x1e),
            Block::Leaf => Color32::from_rgb(0x2d, 0x6a, 0x34),
        }
    }
}

fn install_scene() -> [[Block; 16]; 14] {
    use Block::*;
    [
        [Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air],
        [Air, Air, Air, Air, Air, Leaf, Leaf, Leaf, Air, Air, Air, Air, Air, Air, Air, Air],
        [Air, Air, Air, Air, Leaf, Leaf, Leaf, Leaf, Leaf, Air, Air, Air, Air, Air, Air, Air],
        [Air, Air, Air, Leaf, Leaf, Leaf, Leaf, Leaf, Leaf, Leaf, Air, Air, Air, Air, Air, Air],
        [Air, Air, Air, Air, Air, Trunk, Trunk, Trunk, Air, Air, Air, Air, Air, Air, Air, Air],
        [Air, Air, Air, Air, Air, Trunk, Trunk, Trunk, Air, Air, Air, Air, Air, Air, Air, Air],
        [Air, Air, Air, Air, Air, Trunk, Trunk, Trunk, Air, Air, Air, Air, Air, Air, Air, Air],
        [Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air],
        [Grass, Grass, Grass, Grass, Grass, Grass, Grass, Grass, Grass, Grass, Grass, Grass, Grass, Grass, Grass, Grass],
        [Dirt, Dirt, Dirt, Dirt, Dirt, Dirt, Dirt, Dirt, Dirt, Dirt, Dirt, Dirt, Dirt, Dirt, Dirt, Dirt],
        [Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone],
        [Stone, Stone, Stone, Accent, Accent, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone],
        [Stone, Stone, Stone, Accent, Accent, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone],
        [Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone],
    ]
}

fn uninstall_scene() -> [[Block; 16]; 14] {
    use Block::*;
    [
        [Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air],
        [Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air],
        [Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air],
        [Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air],
        [Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air],
        [Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air],
        [Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air],
        [Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air, Air],
        [Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate],
        [Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate, Deepslate],
        [Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone],
        [Stone, Stone, Stone, Accent, Accent, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone],
        [Stone, Stone, Stone, Accent, Accent, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone],
        [Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone],
    ]
}

fn draw_scene(painter: &egui::Painter, origin: Pos2, scene: &[[Block; 16]; 14]) {
    for (row, line) in scene.iter().enumerate() {
        for (col, block) in line.iter().enumerate() {
            if matches!(block, Block::Air) {
                continue;
            }
            let min = origin + Vec2::new(col as f32 * BLOCK, row as f32 * BLOCK);
            let rect = Rect::from_min_size(min, Vec2::splat(BLOCK));
            painter.rect_filled(rect, CornerRadius::ZERO, block.color());
            painter.rect_stroke(
                rect,
                CornerRadius::ZERO,
                egui::Stroke::new(0.5_f32, Color32::from_rgba_unmultiplied(0, 0, 0, 40)),
                egui::StrokeKind::Inside,
            );
        }
    }
}

pub fn paint(ui: &mut egui::Ui, uninstall: bool, version: &str) {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), ui.available_height()),
        Sense::hover(),
    );
    let painter = ui.painter();

    let top = Color32::from_rgb(0x0a, 0x06, 0x0c);
    let bottom = Color32::from_rgb(0x14, 0x0c, 0x16);
    painter.rect_filled(rect, CornerRadius::ZERO, bottom);
    painter.line_segment(
        [rect.left_top(), rect.right_top()],
        egui::Stroke::new(rect.height(), top),
    );

    let scene = if uninstall {
        uninstall_scene()
    } else {
        install_scene()
    };
    let scene_w = 16.0 * BLOCK;
    let scene_h = 14.0 * BLOCK;
    let origin = rect.center() - Vec2::new(scene_w * 0.5, scene_h * 0.5 + 16.0);
    draw_scene(painter, origin, &scene);

    let footer = Rect::from_min_max(
        egui::pos2(rect.min.x + 20.0, rect.max.y - 72.0),
        rect.max - Vec2::new(20.0, 16.0),
    );
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(footer), |ui| {
        ui.horizontal(|ui| {
            crate::theme::mark(ui, 28.0);
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(APP_NAME)
                        .color(INK)
                        .font(crate::theme::semibold()),
                );
                ui.label(
                    egui::RichText::new(format!("v{version}"))
                        .color(MUTE)
                        .size(12.0),
                );
            });
        });
        ui.add_space(8.0);
        ui.painter().hline(
            footer.x_range(),
            footer.min.y,
            crate::theme::hairline(RAISED2),
        );
    });
}
