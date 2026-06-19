//! Motion helpers for quiet desktop micro-interactions.

use std::time::Duration;

use gpui::{
    Animation, AnimationElement, AnimationExt, Element, ElementId, Styled, ease_out_quint,
    pulsating_between, px,
};

/// Standard motion durations used by Relay components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionDuration {
    Instant,
    Fast,
    Slow,
}

impl MotionDuration {
    pub fn duration(self) -> Duration {
        match self {
            MotionDuration::Instant => Duration::from_millis(50),
            MotionDuration::Fast => Duration::from_millis(150),
            MotionDuration::Slow => Duration::from_millis(300),
        }
    }
}

impl From<MotionDuration> for Duration {
    fn from(value: MotionDuration) -> Self {
        value.duration()
    }
}

/// Direction for entry motion on floating or newly inserted surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionDirection {
    FromBottom,
    FromLeft,
    FromRight,
    FromTop,
}

/// Common GPUI animation helpers for Relay components.
pub trait MotionExt: Styled + Element + Sized + 'static {
    fn motion_fade_in(self) -> AnimationElement<Self> {
        let animation_id = motion_id(&self, "motion-fade-in");
        self.with_animation(
            animation_id,
            Animation::new(MotionDuration::Fast.into()).with_easing(ease_out_quint()),
            |this, delta| this.opacity(0.35 + delta * 0.65),
        )
    }

    fn motion_slide_in(self, direction: MotionDirection, fade: bool) -> AnimationElement<Self> {
        let animation_id = motion_id(&self, direction.animation_name());
        self.with_animation(
            animation_id,
            Animation::new(MotionDuration::Fast.into()).with_easing(ease_out_quint()),
            move |mut this, delta| {
                if fade {
                    this = this.opacity(0.35 + delta * 0.65);
                }

                let offset = -8.0 + delta * 8.0;
                match direction {
                    MotionDirection::FromBottom => this.bottom(px(offset)),
                    MotionDirection::FromLeft => this.left(px(offset)),
                    MotionDirection::FromRight => this.right(px(offset)),
                    MotionDirection::FromTop => this.top(px(offset)),
                }
            },
        )
    }

    fn motion_pulse(self, min_opacity: f32, max_opacity: f32) -> AnimationElement<Self> {
        let animation_id = motion_id(&self, "motion-pulse");
        self.with_animation(
            animation_id,
            Animation::new(MotionDuration::Slow.into())
                .repeat()
                .with_easing(pulsating_between(min_opacity, max_opacity)),
            |this, delta| this.opacity(delta),
        )
    }
}

impl<E> MotionExt for E where E: Styled + Element + Sized + 'static {}

impl MotionDirection {
    fn animation_name(self) -> &'static str {
        match self {
            MotionDirection::FromBottom => "motion-slide-bottom",
            MotionDirection::FromLeft => "motion-slide-left",
            MotionDirection::FromRight => "motion-slide-right",
            MotionDirection::FromTop => "motion-slide-top",
        }
    }
}

fn motion_id(element: &impl Element, animation_name: &'static str) -> ElementId {
    element
        .id()
        .map_or_else(|| animation_name.into(), |id| (id, animation_name).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motion_duration_values_are_stable() {
        assert_eq!(MotionDuration::Fast.duration(), Duration::from_millis(150));
    }

    #[test]
    fn motion_direction_names_are_stable() {
        assert_eq!(
            MotionDirection::FromTop.animation_name(),
            "motion-slide-top"
        );
    }
}
