use std::sync::Arc;

use druid::{Widget, widget::{Flex, MainAxisAlignment, List, Label, CrossAxisAlignment}, WidgetExt, im, Color};

use crate::{gui::state::AppState, server::ClientInfo};

use super::{QrWidget, NonMutWrappable};

pub fn app_widget() -> impl Widget<AppState> {
    Flex::row()
        .main_axis_alignment(MainAxisAlignment::Center)
        .with_child(
            QrWidget::new()
                .fix_size(350., 350.)
                .nonmut_wrap(|s: &AppState| Arc::new(s.qr_code().unwrap()))
                .padding(20.)
                .background(Color::WHITE)
        )
        .with_spacer(20.)
        .with_child(
            Flex::column()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .with_child(Label::new("Connected clients:"))
                .with_spacer(10.0)
                .with_child(
                    List::new(|| Label::dynamic(|(_, v): &(im::Vector<ClientInfo>, ClientInfo), _| v.name.clone()))
                        .nonmut_wrap(|s: &AppState| (s.connected_clients.clone(), s.connected_clients.clone()))
                )
        )
}
