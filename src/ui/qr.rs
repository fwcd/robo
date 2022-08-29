use std::sync::Arc;

use druid::{Widget, Event, EventCtx, LifeCycleCtx, LifeCycle, UpdateCtx, Env, LayoutCtx, BoxConstraints, PaintCtx, Size, RenderContext, Rect, Point, Color};
use qrcodegen::QrCode;

/// A widget rendering a QR code.
pub struct QrWidget;

impl QrWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget<Arc<QrCode>> for QrWidget {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut Arc<QrCode>, _env: &Env) {
        
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &Arc<QrCode>, _env: &Env) {
        
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &Arc<QrCode>, _data: &Arc<QrCode>, _env: &Env) {
        
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &Arc<QrCode>, _env: &Env) -> Size {
        bc.constrain_aspect_ratio(1.0, bc.min().width)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Arc<QrCode>, _env: &Env) {
        let size = ctx.size();
        let qr_size = data.size();
        let mod_width = (size.width / qr_size as f64).round();
        let mod_height = (size.height / qr_size as f64).round();
        let mod_size = Size::new(mod_width, mod_height);
        for y in 0..qr_size {
            for x in 0..qr_size {
                let rect = Rect::from_origin_size(
                    Point::new(x as f64 * mod_width, y as f64 * mod_height),
                    mod_size
                );
                let color = if data.get_module(x, y) { Color::BLACK } else { Color::WHITE };
                ctx.fill(rect, &color);
            }
        }
    }
}
