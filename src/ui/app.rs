use std::sync::Arc;

use druid::{Widget, widget::{Flex, MainAxisAlignment}, WidgetExt};

use crate::state::AppState;

use super::{QrWidget, NonMutWrappable};

pub fn app_widget() -> impl Widget<AppState> {
    Flex::row()
        .main_axis_alignment(MainAxisAlignment::Center)
        .with_child(
            QrWidget::new()
                .fix_size(350., 350.)
                .nonmut_wrap(|s: &AppState| Arc::new(s.qr_code().unwrap()))
        )
}
