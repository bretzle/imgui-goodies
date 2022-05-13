use imgui::{
    sys::{igGetCursorPosY, igGetWindowHeight, igSetCursorPosY},
    Condition, Ui, WindowFlags, StyleVar, StyleColor,
};
use std::time::{Duration, Instant};

const PADDING_X: f32 = 20.0;
const PADDING_Y: f32 = 20.0;
const PADDING_MESSAGE_Y: f32 = 10.0;
const FADE_IN_OUT_TIME: usize = 150;
const OPACITY: f32 = 1.0;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ToastType {
    None,
    Success,
    Warning,
    Error,
    Info,
}

#[derive(Clone, Copy, PartialEq)]
enum ToastPhase {
    FadeIn,
    Wait,
    FadeOut,
    Expired,
}

pub struct Toast {
    typ: ToastType,
    title: Option<String>,
    content: Option<String>,
    dismiss_time: usize,
    creation_time: Instant,
}

impl Toast {
    pub fn new(typ: ToastType, duration: usize) -> Self {
        Self {
            typ,
            title: None,
            content: None,
            dismiss_time: duration,
            creation_time: Instant::now(),
        }
    }

    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn content<S: Into<String>>(mut self, content: S) -> Self {
        self.content = Some(content.into());
        self
    }

    fn get_elapsed_time(&self) -> Duration {
        self.creation_time.elapsed()
    }

    fn get_phase(&self) -> ToastPhase {
        let elapsed = self.get_elapsed_time().as_millis() as usize;

        if elapsed > (FADE_IN_OUT_TIME + self.dismiss_time + FADE_IN_OUT_TIME) {
            ToastPhase::Expired
        } else if elapsed > FADE_IN_OUT_TIME + self.dismiss_time {
            ToastPhase::FadeOut
        } else if elapsed > FADE_IN_OUT_TIME {
            ToastPhase::Wait
        } else {
            ToastPhase::FadeIn
        }
    }

    fn get_icon(&self) -> Option<()> {
        // match self.typ {
        //     ToastType::None => todo!(),
        //     ToastType::Success => todo!(),
        //     ToastType::Warning => todo!(),
        //     ToastType::Error => todo!(),
        //     ToastType::Info => todo!(),
        // }
        None
    }

    fn get_title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    fn get_content(&self) -> Option<&str> {
        self.content.as_deref()
    }

    fn get_default_title(&self) -> Option<&str> {
        match self.title {
            Some(_) => self.title.as_deref(),
            None => match self.typ {
                ToastType::None => None,
                ToastType::Success => Some("Success"),
                ToastType::Warning => Some("Warning"),
                ToastType::Error => Some("Error"),
                ToastType::Info => Some("Info"),
            },
        }
    }

    fn get_fade_percent(&self) -> f32 {
        let phase = self.get_phase();
        let elapsed = self.get_elapsed_time();

        if phase == ToastPhase::FadeIn {
            (elapsed.as_millis() as f32) / (FADE_IN_OUT_TIME as f32) * OPACITY
        } else if phase == ToastPhase::FadeOut {
            (1.0 - (((elapsed.as_millis() as f32)
                - (FADE_IN_OUT_TIME as f32)
                - (self.dismiss_time as f32))
                / (FADE_IN_OUT_TIME as f32)))
                * OPACITY
        } else {
            1.0 * OPACITY
        }
    }

    fn get_color(&self) -> [f32; 4] {
        match self.typ {
            ToastType::None => [255.0, 255.0, 255.0, 255.0],
            ToastType::Success => [0.0, 255.0, 0.0, 255.0],
            ToastType::Warning => [255.0, 255.0, 0.0, 255.0],
            ToastType::Error => [255.0, 0.0, 0.0, 255.0],
            ToastType::Info => [0.0, 157.0, 255.0, 255.0],
        }
    }
}

pub struct Notifications(Vec<Toast>);

impl Notifications {
    pub fn new() -> Self {
        Self(vec![])
    }

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
