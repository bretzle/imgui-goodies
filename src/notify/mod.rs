use imgui::{
    sys::{igGetCursorPosY, igGetWindowHeight, igSetCursorPosY},
    Condition, StyleColor, StyleVar, Ui, WindowFlags,
};

mod toast;

pub use toast::*;

const PADDING_X: f32 = 20.0;
const PADDING_Y: f32 = 20.0;
const PADDING_MESSAGE_Y: f32 = 10.0;
const FADE_IN_OUT_TIME: usize = 150;
const OPACITY: f32 = 1.0;

#[derive(Default)]
pub struct Notifications(Vec<Toast>);

impl Notifications {
    pub fn push(&mut self, toast: Toast) {
        self.0.push(toast);
    }

    pub fn render(&mut self, ui: &Ui) {
        let vp_size = unsafe { (*imgui::sys::igGetMainViewport()).Size };
        let mut height = 0.0;
        let mut idx = 0;

        let _sv = ui.push_style_var(StyleVar::WindowRounding(5.0));
        let _ct = ui.push_style_color(
            StyleColor::WindowBg,
            [43. / 255., 43. / 255., 43. / 255., 100. / 255.],
        );

        self.0.retain(|toast| {
            if toast.get_phase() == ToastPhase::Expired {
                return false;
            }

            let icon = toast.get_icon();
            let title = toast.get_title();
            let content = toast.get_content();
            let default_title = toast.get_default_title();
            let opacity = toast.get_fade_percent();

            let mut text_color = toast.get_color();
            text_color[3] = opacity;

            let window_name = format!("##TOAST{idx}");

            ui.window(window_name)
                .bg_alpha(opacity)
                .position(
                    [vp_size.x - PADDING_X, vp_size.y - PADDING_Y - height],
                    Condition::Always,
                )
                .position_pivot([1.0, 1.0])
                .flags(
                    WindowFlags::ALWAYS_AUTO_RESIZE
                        | WindowFlags::NO_DECORATION
                        | WindowFlags::NO_INPUTS
                        | WindowFlags::NO_NAV
                        | WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS
                        | WindowFlags::NO_FOCUS_ON_APPEARING,
                )
                .build(|| {
                    let _t = ui.push_text_wrap_pos_with_pos(vp_size.x / 3.0);
                    let mut was_title_rendered = false;

                    if let Some(_icon) = icon {
                        todo!()
                    }

                    if let Some(title) = title {
                        if icon.is_some() {
                            ui.same_line();
                        }

                        ui.text(title);
                        was_title_rendered = true;
                    } else if let Some(title) = default_title {
                        if icon.is_some() {
                            ui.same_line();
                        }

                        ui.text(title);
                        was_title_rendered = true;
                    }

                    if was_title_rendered && content.is_some() {
                        unsafe {
                            igSetCursorPosY(igGetCursorPosY() + 5.0);
                        }
                    }

                    if let Some(content) = content {
                        if was_title_rendered {
                            ui.separator();
                        }
                        ui.text(content);
                    }

                    height += unsafe { igGetWindowHeight() } + PADDING_MESSAGE_Y;
                });

            idx += 1;

            true
        });
    }
}
