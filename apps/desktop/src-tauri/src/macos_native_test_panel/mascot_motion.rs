use std::time::Instant;

use super::mascot::{NativeMascotMotion, NativeMascotState};
use super::panel_constants::MASCOT_WAKE_ANGRY_SECONDS;
use super::panel_helpers::lerp;

pub(super) fn native_mascot_target_motion(
    state: NativeMascotState,
    t: f64,
    wake_started_at: Option<Instant>,
    now: Instant,
) -> NativeMascotMotion {
    match state {
        NativeMascotState::Bouncing => {
            let bounce = (t * 5.8).sin().abs();
            let hang = bounce.powf(0.72);
            let landing = (1.0 - bounce).powf(3.2);
            NativeMascotMotion {
                offset_x: (t * 3.1).sin() * 0.28,
                offset_y: hang * 5.6,
                scale_x: 1.0 + landing * 0.18 + hang * 0.018,
                scale_y: 1.0 - landing * 0.16 + hang * 0.018,
                shell_alpha: 1.0,
                shadow_opacity: 0.46,
                shadow_radius: 5.4,
            }
        }
        NativeMascotState::Approval => {
            let pulse = ((t * 7.2).sin() + 1.0) * 0.5;
            NativeMascotMotion {
                offset_x: (t * 9.0).sin() * 0.34,
                offset_y: 0.0,
                scale_x: 1.0 + pulse * 0.025,
                scale_y: 1.0 - pulse * 0.018,
                shell_alpha: 1.0,
                shadow_opacity: 0.52,
                shadow_radius: 6.0,
            }
        }
        NativeMascotState::Question => {
            let tilt = (t * 4.4).sin();
            NativeMascotMotion {
                offset_x: tilt * 0.28,
                offset_y: (t * 5.1).sin() * 0.55,
                scale_x: 1.0 + tilt.abs() * 0.012,
                scale_y: 1.0,
                shell_alpha: 1.0,
                shadow_opacity: 0.50,
                shadow_radius: 5.8,
            }
        }
        NativeMascotState::MessageBubble => {
            let bob = ((t * 3.2).sin() + 1.0) * 0.5;
            NativeMascotMotion {
                offset_x: 0.0,
                offset_y: bob * 1.6,
                scale_x: 1.0 + bob * 0.012,
                scale_y: 1.0 - bob * 0.008,
                shell_alpha: 1.0,
                shadow_opacity: 0.46,
                shadow_radius: 5.2,
            }
        }
        NativeMascotState::Complete => {
            let bob = ((t * 2.4).sin() + 1.0) * 0.5;
            NativeMascotMotion {
                offset_x: 0.0,
                offset_y: bob * 0.8,
                scale_x: 1.0 + bob * 0.010,
                scale_y: 1.0 - bob * 0.006,
                shell_alpha: 1.0,
                shadow_opacity: 0.48,
                shadow_radius: 5.4,
            }
        }
        NativeMascotState::Sleepy => {
            let breath = ((t * 0.9).sin() + 1.0) * 0.5;
            let sleepy_phase = (t + 0.9).rem_euclid(7.6);
            let nod = if sleepy_phase > 5.1 && sleepy_phase < 5.95 {
                (((sleepy_phase - 5.1) / 0.85) * std::f64::consts::PI).sin()
            } else {
                0.0
            };
            NativeMascotMotion {
                offset_x: 0.0,
                offset_y: nod * -0.7,
                scale_x: 1.0 + breath * 0.012,
                scale_y: 0.96 - breath * 0.012 + nod * 0.01,
                shell_alpha: 0.70,
                shadow_opacity: 0.18,
                shadow_radius: 3.0,
            }
        }
        NativeMascotState::WakeAngry => {
            let elapsed = wake_started_at
                .map(|started_at| now.saturating_duration_since(started_at).as_secs_f64())
                .unwrap_or(0.0);
            let fade = 1.0 - smoothstep_range(0.52, MASCOT_WAKE_ANGRY_SECONDS, elapsed);
            NativeMascotMotion {
                offset_x: (elapsed * 30.0).sin() * 1.85 * fade,
                offset_y: 0.0,
                scale_x: 1.0 + 0.045 * fade,
                scale_y: 1.0 - 0.04 * fade,
                shell_alpha: 1.0,
                shadow_opacity: 0.56,
                shadow_radius: 6.4,
            }
        }
        NativeMascotState::Idle => {
            let breath = ((t * 1.1).sin() + 1.0) * 0.5;
            NativeMascotMotion {
                offset_x: 0.0,
                offset_y: 0.0,
                scale_x: 1.0 + breath * 0.006,
                scale_y: 1.0 - breath * 0.004,
                shell_alpha: 1.0,
                shadow_opacity: 0.34,
                shadow_radius: 4.0,
            }
        }
    }
}

pub(super) fn native_mascot_color(
    state: NativeMascotState,
    _t: f64,
    wake_started_at: Option<Instant>,
    now: Instant,
) -> [f64; 4] {
    match state {
        NativeMascotState::Approval | NativeMascotState::Question => [1.0, 0.48, 0.14, 1.0],
        NativeMascotState::MessageBubble | NativeMascotState::Complete => [1.0, 0.48, 0.14, 1.0],
        NativeMascotState::Bouncing => [1.0, 0.48, 0.14, 1.0],
        NativeMascotState::Sleepy => [0.72, 0.30, 0.13, 1.0],
        NativeMascotState::WakeAngry => {
            let elapsed = wake_started_at
                .map(|started_at| now.saturating_duration_since(started_at).as_secs_f64())
                .unwrap_or(0.0);
            let blink = if (elapsed * 12.0).sin() >= 0.0 {
                1.0
            } else {
                0.0
            };
            [
                lerp(1.0, 1.0, blink),
                lerp(0.38, 0.48, blink),
                lerp(0.24, 0.14, blink),
                1.0,
            ]
        }
        NativeMascotState::Idle => [1.0, 0.48, 0.14, 1.0],
    }
}

pub(super) fn native_mascot_lerp_motion(
    start: NativeMascotMotion,
    end: NativeMascotMotion,
    progress: f64,
) -> NativeMascotMotion {
    NativeMascotMotion {
        offset_x: lerp(start.offset_x, end.offset_x, progress),
        offset_y: lerp(start.offset_y, end.offset_y, progress),
        scale_x: lerp(start.scale_x, end.scale_x, progress),
        scale_y: lerp(start.scale_y, end.scale_y, progress),
        shell_alpha: lerp(start.shell_alpha, end.shell_alpha, progress),
        shadow_opacity: lerp(
            start.shadow_opacity as f64,
            end.shadow_opacity as f64,
            progress,
        ) as f32,
        shadow_radius: lerp(start.shadow_radius, end.shadow_radius, progress),
    }
}

pub(super) fn smoothstep_unit(progress: f64) -> f64 {
    let progress = progress.clamp(0.0, 1.0);
    progress * progress * (3.0 - (2.0 * progress))
}

pub(super) fn smoothstep_range(edge0: f64, edge1: f64, value: f64) -> f64 {
    if (edge1 - edge0).abs() <= f64::EPSILON {
        return if value >= edge1 { 1.0 } else { 0.0 };
    }
    smoothstep_unit((value - edge0) / (edge1 - edge0))
}
