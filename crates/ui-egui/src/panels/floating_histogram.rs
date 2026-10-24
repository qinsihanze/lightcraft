//! Floating Histogram overlay on the photo preview canvas (Loupe/Detail view).
//!
//! Renders a draggable, semi-transparent HUD histogram card above the photo canvas,
//! with live multi-channel RGB + luminance curves, clipping warning toggles, HDR indicators,
//! and key exposure EXIF metrics.

use egui::epaint::{Mesh, Vertex};
use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, StrokeKind, pos2, vec2};
use lightcraft_catalog::PhotoId;
use serde_json::json;

use crate::LightcraftApp;
use crate::icons::{Icon, paint};
use crate::render::Slot;
use crate::theme::Tokens;
use crate::widgets::{hex, register};

/// Default dimensions of the floating histogram overlay (in logical points).
const WIDTH: f32 = 230.0;
const HEIGHT: f32 = 142.0;

/// Render the floating histogram overlay on top of the loupe photo preview canvas.
pub fn show(app: &mut LightcraftApp, ui: &mut egui::Ui, canvas: Rect, id: PhotoId) {
    let t = Tokens::get(ui.ctx());
    let initial_pos = pos2(canvas.right() - WIDTH - 16.0, canvas.top() + 16.0);
    let mut origin = match app.ui.loupe_histogram_pos {
        Some((x, y)) => pos2(x, y),
        None => initial_pos,
    };

    // Ensure the floating histogram remains completely within canvas boundaries
    let max_x = (canvas.right() - WIDTH - 8.0).max(canvas.left() + 8.0);
    let max_y = (canvas.bottom() - HEIGHT - 8.0).max(canvas.top() + 8.0);
    origin.x = origin.x.clamp(canvas.left() + 8.0, max_x);
    origin.y = origin.y.clamp(canvas.top() + 8.0, max_y);
    let frame = Rect::from_min_size(origin, vec2(WIDTH, HEIGHT));

    let p = ui.painter_at(canvas);
    // Frosted glassmorphism background and subtle border
    p.rect_filled(frame, 6.0, Color32::from_rgba_unmultiplied(24, 24, 26, 215));
    p.rect_stroke(frame, 6.0, Stroke::new(1.0, Color32::from_white_alpha(35)), StrokeKind::Inside);
    register(ui.ctx(), "loupe:floatingHistogram", frame);

    // Title bar area: drag handle for moving the floating overlay
    let title_rect = Rect::from_min_size(frame.min, vec2(frame.width(), 24.0));
    let title_resp = ui.interact(title_rect, egui::Id::new("floating-histogram-drag"), Sense::click_and_drag());
    if title_resp.dragged() {
        let delta = title_resp.drag_delta();
        let new_pos = (origin.x + delta.x, origin.y + delta.y);
        app.ui.loupe_histogram_pos = Some(new_pos);
    }

    // Title label
    p.text(pos2(frame.left() + 10.0, frame.top() + 12.0), Align2::LEFT_CENTER, crate::i18n::tr("Histogram"), t.semibold(12.0), t.text);

    // Live histogram data acquisition (supports live HDR updates during drag)
    let hdr_edit = app.session.develop_of(id).is_some_and(|d| d.hdr.enabled);
    let interacting = app.session.interaction.is_some();
    if interacting && hdr_edit {
        let edge = lightcraft_pipeline::HDR_HISTOGRAM_DRAG_EDGE;
        if let Some(job) = app.session.render_job(id, edge, edge, false, app.ui.right != crate::state::RightPanel::Crop)
            && app.renderer.live_hdr_histogram.as_ref().is_none_or(|l| l.1 != job.key)
        {
            app.renderer.request(Slot::HdrHistogram, job, 95);
        }
    }
    let live = app.renderer.live_hdr_histogram.as_ref().filter(|l| interacting && l.0 == id).map(|l| l.2.clone());
    let hist = live.or_else(|| super::edit::preview_histogram(app, id));
    let hist = super::edit::steady_hdr_histogram(ui.ctx(), id, hdr_edit, hist);
    let clipping = hist.as_ref().map(|h| h.clipping());

    // Clipping warning buttons: Shadows and Highlights
    let mut toggle_clipping = false;
    let shadows_rect = Rect::from_center_size(pos2(frame.right() - 56.0, frame.top() + 12.0), vec2(16.0, 16.0));
    let highlights_rect = Rect::from_center_size(pos2(frame.right() - 36.0, frame.top() + 12.0), vec2(16.0, 16.0));
    let close_rect = Rect::from_center_size(pos2(frame.right() - 14.0, frame.top() + 12.0), vec2(16.0, 16.0));

    // Shadow clipping button
    let s_resp = ui.interact(shadows_rect, egui::Id::new("floating-hist-shadows"), Sense::click());
    register(ui.ctx(), "loupe:floatingHistogram:shadows", shadows_rect);
    let s_clipped = clipping.is_some_and(|(lo, _)| lo > 0.003);
    let s_active = app.ui.show_clipping;
    p.rect_filled(shadows_rect, 2.0, if s_active { t.pressed } else if s_resp.hovered() { t.hover } else { Color32::from_gray(50) });
    paint(&p, Rect::from_center_size(shadows_rect.center(), vec2(12.0, 12.0)), Icon::ChevronDown, if s_clipped || s_active { t.text } else { t.text_dim });
    if s_resp.clicked() {
        toggle_clipping = true;
    }
    s_resp.on_hover_text(crate::i18n::tr("Shadow clipping warnings"));

    // Highlight clipping button
    let h_resp = ui.interact(highlights_rect, egui::Id::new("floating-hist-highlights"), Sense::click());
    register(ui.ctx(), "loupe:floatingHistogram:highlights", highlights_rect);
    let h_clipped = clipping.is_some_and(|(_, hi)| hi > 0.003);
    p.rect_filled(highlights_rect, 2.0, if s_active { t.pressed } else if h_resp.hovered() { t.hover } else { Color32::from_gray(50) });
    paint(&p, Rect::from_center_size(highlights_rect.center(), vec2(12.0, 12.0)), Icon::ChevronDown, if h_clipped || s_active { t.text } else { t.text_dim });
    if h_resp.clicked() {
        toggle_clipping = true;
    }
    h_resp.on_hover_text(crate::i18n::tr("Highlight clipping warnings"));

    // Close button (×)
    let close_resp = ui.interact(close_rect, egui::Id::new("floating-hist-close"), Sense::click());
    register(ui.ctx(), "loupe:floatingHistogram:close", close_rect);
    let close_c = close_rect.center();
    let m = 3.0;
    let cross_color = if close_resp.hovered() { Color32::WHITE } else { t.text_dim };
    p.line_segment([close_c - vec2(m, m), close_c + vec2(m, m)], Stroke::new(1.2, cross_color));
    p.line_segment([close_c - vec2(m, -m), close_c + vec2(m, -m)], Stroke::new(1.2, cross_color));
    if close_resp.clicked() {
        app.ui.loupe_histogram = false;
    }
    close_resp.on_hover_text(crate::i18n::tr("Hide histogram overlay"));

    if toggle_clipping {
        let _ = app.run("view.clipping", json!({}));
    }

    // Plot area inside the floating card
    let plot = Rect::from_min_max(pos2(frame.left() + 10.0, frame.top() + 26.0), pos2(frame.right() - 10.0, frame.bottom() - 24.0));
    p.rect_filled(plot, 2.0, Color32::from_black_alpha(120));

    if let Some(h) = hist {
        let smooth = |v: &[u32]| -> Vec<f32> {
            (0..256)
                .map(|i| {
                    let (a, b) = (i.max(2) - 2, (i + 3).min(256));
                    v[a..b].iter().map(|x| *x as f32).sum::<f32>() / (b - a) as f32
                })
                .collect()
        };

        // HDR stop markers if in HDR mode
        if let Some(b) = h.hdr {
            let x0 = plot.left() + plot.width() * b.from as f32 / 255.0;
            p.line_segment([pos2(x0, plot.top()), pos2(x0, plot.bottom())], Stroke::new(1.0, Color32::from_gray(140)));
            let limit = app.session.develop_of(id).map_or(b.stops, |d| d.hdr.max_ev as f32);
            if limit.is_finite() && limit >= 0.0 && limit < b.stops - 0.05 {
                let x = x0 + (plot.right() - x0) * (limit / b.stops).clamp(0.0, 1.0);
                p.line_segment([pos2(x, plot.top()), pos2(x, plot.bottom())], Stroke::new(1.0, Color32::from_rgb(230, 150, 60)));
            }
        }

        let chans = [(smooth(&h.r), hex("#df3939")), (smooth(&h.g), hex("#44b072")), (smooth(&h.b), hex("#3b6fe0"))];
        let peak = chans.iter().flat_map(|(v, _)| v.iter().skip(2).take(252)).fold(1.0f32, |a, b| a.max(*b));
        let to = |i: usize, v: f32| pos2(plot.left() + plot.width() * i as f32 / 255.0, plot.bottom() - plot.height() * (v / peak).sqrt().min(1.0));

        // Luminance fill mesh
        let lum = smooth(&h.luma);
        let mut mesh = Mesh::default();
        for i in 0..256 {
            let top = to(i, lum[i]);
            mesh.vertices.push(Vertex { pos: top, uv: Pos2::ZERO, color: Color32::from_gray(120).gamma_multiply(0.35) });
            mesh.vertices.push(Vertex { pos: pos2(top.x, plot.bottom()), uv: Pos2::ZERO, color: Color32::from_gray(120).gamma_multiply(0.35) });
            if i > 0 {
                let k = (i as u32) * 2;
                mesh.indices.extend([k - 2, k - 1, k, k - 1, k + 1, k]);
            }
        }
        p.add(mesh);

        // Greyscale check
        let spread: f32 = (0..256).map(|i| (chans[0].0[i] - chans[1].0[i]).abs() + (chans[1].0[i] - chans[2].0[i]).abs()).sum();
        let total: f32 = chans[1].0.iter().sum::<f32>().max(1.0);
        if spread / total < 0.02 {
            let pts: Vec<Pos2> = (0..256).map(|i| to(i, chans[1].0[i])).collect();
            p.add(egui::Shape::line(pts, Stroke::new(1.2, Color32::from_gray(200))));
        } else {
            for (v, c) in &chans {
                let pts: Vec<Pos2> = (0..256).map(|i| to(i, v[i])).collect();
                p.add(egui::Shape::line(pts, Stroke::new(1.2, *c)));
            }
        }
    }

    // EXIF summary footer line
    if let Some(ph) = app.session.catalog.photo(id) {
        let m = &ph.meta;
        let parts: Vec<String> = [
            m.iso.map(|i| format!("ISO {i}")),
            m.focal_mm.map(|f| format!("{f:.0}mm")),
            m.aperture.map(|a| format!("f/{a:.1}")),
            if m.shutter.is_empty() { None } else { Some(m.shutter.clone()) },
        ]
        .into_iter()
        .flatten()
        .collect();

        if !parts.is_empty() {
            let text = parts.join(" · ");
            p.text(pos2(frame.center().x, frame.bottom() - 11.0), Align2::CENTER_CENTER, text, t.font(10.5), t.text_dim);
        }
    }
}
