use std::time::{Duration, Instant};

use super::{FADE_IN_OUT_TIME, OPACITY};

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
pub(super) enum ToastPhase {
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

    pub(super) fn get_phase(&self) -> ToastPhase {
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

    pub(super) fn get_icon(&self) -> Option<()> {
        // match self.typ {
        //     ToastType::None => todo!(),
        //     ToastType::Success => todo!(),
        //     ToastType::Warning => todo!(),
        //     ToastType::Error => todo!(),
        //     ToastType::Info => todo!(),
        // }
        None
    }

    pub(super) fn get_title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub(super) fn get_content(&self) -> Option<&str> {
        self.content.as_deref()
    }

    pub(super) fn get_default_title(&self) -> Option<&str> {
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

    pub(super) fn get_fade_percent(&self) -> f32 {
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

    pub(super) fn get_color(&self) -> [f32; 4] {
        match self.typ {
            ToastType::None => [255.0, 255.0, 255.0, 255.0],
            ToastType::Success => [0.0, 255.0, 0.0, 255.0],
            ToastType::Warning => [255.0, 255.0, 0.0, 255.0],
            ToastType::Error => [255.0, 0.0, 0.0, 255.0],
            ToastType::Info => [0.0, 157.0, 255.0, 255.0],
        }
    }
}
