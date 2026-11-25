// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Animated 3D / Phase-Space Vector Icons powered by `aether_native::EtalonVectorWidget`
//! and `etalon_vibe` zero-heap 10D->3D projection, SDF evaluation, Spherical Harmonics
//! irradiance, and symplectic Verlet spring-damper mechanics.

use crate::state::VolumeStage;
use aether::prelude::*;
use aether_native::{
    CustomWidgetAttr, EtalonVectorDelegate, EtalonVectorFrame, EtalonVectorWidget, RenderOutcome,
};
use anyrender::{PaintScene, Scene};
use blitz_dom::IntrinsicSizes;
use blitz_traits::events::UiEvent;
use kurbo::{Affine, BezPath, Circle, Line, Point, RoundedRect, Stroke};
use peniko::{Color, Fill};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use webizen_ua_os::{
    etalon_vibe::{
        oracle_bioelectric_dipole_preview, oracle_project_10d_to_3d, oracle_sdf_torus_eval,
        oracle_sh_irradiance_probe, oracle_spring_damper_force, oracle_verlet_step,
    },
    poet_eval_native_cell,
};

static NEXT_GLYPH_WIDGET_ID: AtomicUsize = AtomicUsize::new(10_000);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoloGlyphKind {
    CommandOrb,
    NotificationsBell,
    TalkWave,
    MailToroid,
    DirectoryConstellation,
    BrowserGlobe10d,
    KeepVaultShield,
    LibraryTesseract,
    InstrumentsOscillator,
    PoetQuillMobius,
    WellfairHeartDyad,
    WalletLightningLoop,
    ProjectsWorkGraph,
    SettingsGearHarmonic,
    AdminSentinelCube,
}

impl HoloGlyphKind {
    pub const fn for_stage(stage: VolumeStage) -> Self {
        match stage {
            VolumeStage::None => Self::CommandOrb,
            VolumeStage::Talk => Self::TalkWave,
            VolumeStage::Mail => Self::MailToroid,
            VolumeStage::Directory => Self::DirectoryConstellation,
            VolumeStage::Browser => Self::BrowserGlobe10d,
            VolumeStage::Keep => Self::KeepVaultShield,
            VolumeStage::Library => Self::LibraryTesseract,
            VolumeStage::Instruments => Self::InstrumentsOscillator,
            VolumeStage::Poet => Self::PoetQuillMobius,
            VolumeStage::Wellfair | VolumeStage::Anatomy => Self::WellfairHeartDyad,
            VolumeStage::Wallet => Self::WalletLightningLoop,
            VolumeStage::Projects => Self::ProjectsWorkGraph,
            VolumeStage::Settings => Self::SettingsGearHarmonic,
            VolumeStage::Admin => Self::AdminSentinelCube,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct HoloGlyphControl {
    kind: HoloGlyphKind,
    active: bool,
    animating: bool,
    omega: f32,
    high_contrast: bool,
}

struct HoloGlyphDelegate {
    control: Arc<Mutex<HoloGlyphControl>>,
    phase: f32,
    last_tick: Option<Instant>,
    spring_pos: [f64; 3],
    spring_vel: [f64; 3],
    hovered: bool,
}

impl HoloGlyphDelegate {
    fn new(control: Arc<Mutex<HoloGlyphControl>>, phase_seed: f32) -> Self {
        Self {
            control,
            phase: phase_seed,
            last_tick: None,
            spring_pos: [0.0, 0.0, 0.0],
            spring_vel: [0.0, 0.0, 0.0],
            hovered: false,
        }
    }

    fn step_kinetics(&mut self, animating: bool, omega: f32, active: bool) -> (f32, f32) {
        if !animating {
            self.last_tick = None;
            return (self.phase, if active { 1.08 } else { 1.0 });
        }
        let now = Instant::now();
        let dt = self
            .last_tick
            .replace(now)
            .map(|prev| now.duration_since(prev).min(Duration::from_millis(64)).as_secs_f32())
            .unwrap_or(0.016);
        let speed = if active || self.hovered {
            omega * 1.45
        } else {
            omega * 0.85
        };
        self.phase = (self.phase + dt * speed).rem_euclid(std::f32::consts::TAU);

        // Step deterministic Etalon spring-damper + symplectic Verlet kinetics (`etalon-vibe`).
        let target_x = if active || self.hovered { 0.14 } else { 0.0 };
        let disp = [
            self.spring_pos[0] - target_x,
            self.spring_pos[1],
            self.spring_pos[2],
        ];
        if let Ok(force) = oracle_spring_damper_force(disp, self.spring_vel, 28.0, 4.2) {
            if let Ok(stepped) =
                oracle_verlet_step(self.spring_pos, self.spring_vel, force, f64::from(dt.max(0.001)))
            {
                self.spring_pos = [stepped[0], stepped[1], stepped[2]];
                self.spring_vel = [stepped[3], stepped[4], stepped[5]];
            }
        }
        let scale_boost = (1.0 + self.spring_pos[0] as f32).clamp(0.92, 1.18);
        (self.phase, scale_boost)
    }
}

/// Project a 3D point surrounded by 7D harmonic phase coordinates through `etalon_vibe::oracle_project_10d_to_3d`.
fn project_10d_point(xyz: [f32; 3], phase: f32, scale_boost: f32) -> [f32; 3] {
    let (s, c) = phase.sin_cos();
    let (s2, c2) = (phase * 0.5).sin_cos();
    let rx = xyz[0] * c + xyz[2] * s;
    let rz = -xyz[0] * s + xyz[2] * c;
    let ry = xyz[1] * c2 - rz * s2 * 0.35;
    let state10d = [
        rx * scale_boost,
        ry * scale_boost,
        rz * scale_boost,
        0.08 * s,
        0.06 * c,
        0.04 * s2,
        0.04 * c2,
        0.02,
        0.02,
        0.01,
    ];
    let weights7d = [0.15, 0.12, 0.10, 0.08, 0.05, 0.04, 0.02];
    oracle_project_10d_to_3d(&state10d, &weights7d).unwrap_or([rx, ry, rz])
}

impl EtalonVectorDelegate for HoloGlyphDelegate {
    fn paint_scene(&mut self, frame: &EtalonVectorFrame<'_>, scene: &mut Scene) -> RenderOutcome {
        let ctrl = *self
            .control
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let (phase, boost) = self.step_kinetics(ctrl.animating, ctrl.omega, ctrl.active);
        let w = f64::from(frame.width);
        let h = f64::from(frame.height);
        let extent = w.min(h);
        let center = Point::new(w * 0.5, h * 0.5);
        let radius = extent * 0.36 * f64::from(boost);

        // Compute Spherical Harmonics L0/L1 irradiance modulation via `etalon-vibe`.
        let normal = [phase.cos(), phase.sin(), 0.72];
        let sh_probe = oracle_sh_irradiance_probe([0.88, 0.32, 0.24, 0.45], normal)
            .unwrap_or([1.0, 0.5]);
        let lum = sh_probe[0].clamp(0.75, 1.35);

        let (primary, secondary, accent, bg_color) = if ctrl.high_contrast {
            (
                Color::from_rgba8(0, 245, 212, 255),
                Color::from_rgba8(102, 204, 255, 255),
                Color::from_rgba8(255, 210, 122, 255),
                Color::from_rgba8(11, 16, 26, 255),
            )
        } else if ctrl.active {
            (
                Color::from_rgba8((40.0 * lum).min(255.0) as u8, 238, 208, 255),
                Color::from_rgba8(95, 198, 255, 245),
                Color::from_rgba8(255, 212, 122, 255),
                Color::from_rgba8(14, 34, 46, 245),
            )
        } else {
            (
                Color::from_rgba8(42, 218, 192, 235),
                Color::from_rgba8(82, 178, 250, 225),
                Color::from_rgba8(162, 145, 255, 225),
                Color::from_rgba8(16, 26, 42, 235),
            )
        };

        // Background rounded vessel
        let bg_rect = RoundedRect::new(1.0, 1.0, (w - 2.0).max(2.0), (h - 2.0).max(2.0), 6.0);
        scene.fill(Fill::NonZero, Affine::IDENTITY, bg_color, None, &bg_rect);

        let stroke_main = Stroke::new((extent * 0.068).clamp(1.25, 2.4));
        let stroke_thin = Stroke::new((extent * 0.044).clamp(0.95, 1.65));

        // Outer phase-space halo ring
        let halo_r = radius * (1.02 + 0.05 * f64::from(phase.sin()));
        scene.stroke(
            &stroke_thin,
            Affine::IDENTITY,
            Color::from_rgba8(76, 185, 255, if ctrl.active { 165 } else { 85 }),
            None,
            &Circle::new(center, halo_r),
        );

        match ctrl.kind {
            HoloGlyphKind::CommandOrb => {
                // 8-sector radial command orb with rotating 10D->3D inner ring
                for sector in 0..8 {
                    let angle = f64::from(phase) + (sector as f64) * std::f64::consts::TAU / 8.0;
                    let (sy, cx) = angle.sin_cos();
                    let p0 = Point::new(center.x + cx * radius * 0.42, center.y + sy * radius * 0.42);
                    let p1 = Point::new(center.x + cx * radius * 0.92, center.y + sy * radius * 0.92);
                    let col = if sector % 2 == 0 { primary } else { secondary };
                    scene.stroke(&stroke_thin, Affine::IDENTITY, col, None, &Line::new(p0, p1));
                }
                scene.stroke(
                    &stroke_main,
                    Affine::IDENTITY,
                    primary,
                    None,
                    &Circle::new(center, radius * 0.54),
                );
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    accent,
                    None,
                    &Circle::new(center, radius * 0.22),
                );
            }
            HoloGlyphKind::NotificationsBell => {
                // Resonant acoustic wavefield bell + pulsing clapper & wave arcs
                let swing = 0.16 * f64::from(phase.sin());
                let mut bell = BezPath::new();
                bell.move_to(Point::new(center.x - radius * 0.65, center.y + radius * 0.42));
                bell.quad_to(
                    Point::new(center.x - radius * 0.52, center.y - radius * 0.65),
                    Point::new(center.x + swing * radius, center.y - radius * 0.72),
                );
                bell.quad_to(
                    Point::new(center.x + radius * 0.52, center.y - radius * 0.65),
                    Point::new(center.x + radius * 0.65, center.y + radius * 0.42),
                );
                bell.close_path();
                scene.stroke(&stroke_main, Affine::IDENTITY, primary, None, &bell);
                let clapper = Point::new(
                    center.x + f64::from((phase * 1.5).sin()) * radius * 0.25,
                    center.y + radius * 0.68,
                );
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    accent,
                    None,
                    &Circle::new(clapper, radius * 0.18),
                );
            }
            HoloGlyphKind::TalkWave => {
                // Dual bioelectric dipole wave (`oracle_bioelectric_dipole_preview`)
                let dipole = oracle_bioelectric_dipole_preview(
                    [phase.cos(), phase.sin(), 0.5],
                    [0.2, 0.1, 0.4],
                    1.2,
                )
                .unwrap_or([0.6, 0.4]);
                let amp = (0.32 + 0.18 * dipole[0].abs()).clamp(0.25, 0.65) as f64;
                for (idx, col) in [(0.0, primary), (1.4, secondary)] {
                    let mut wave = BezPath::new();
                    for step in 0..=16 {
                        let t = (step as f64 / 16.0) * 2.0 - 1.0;
                        let x = center.x + t * radius * 0.85;
                        let y = center.y
                            + (t * 3.4 + f64::from(phase) + idx).sin()
                                * radius
                                * amp
                                * (1.0 - t * t * 0.45);
                        if step == 0 {
                            wave.move_to(Point::new(x, y));
                        } else {
                            wave.line_to(Point::new(x, y));
                        }
                    }
                    scene.stroke(&stroke_main, Affine::IDENTITY, col, None, &wave);
                }
            }
            HoloGlyphKind::MailToroid => {
                // Toroidal SDF envelope (`oracle_sdf_torus_eval`) + directional pairwise chevron
                let sdf =
                    oracle_sdf_torus_eval([phase.cos() * 0.4, phase.sin() * 0.4, 0.1], 0.55, 0.18)
                        .unwrap_or(0.1);
                let inset = (0.72 + (sdf.abs() as f64) * 0.15).clamp(0.65, 0.88);
                let env = RoundedRect::new(
                    center.x - radius * inset,
                    center.y - radius * 0.56,
                    center.x + radius * inset,
                    center.y + radius * 0.56,
                    3.0,
                );
                scene.stroke(&stroke_main, Affine::IDENTITY, primary, None, &env);
                let mut flap = BezPath::new();
                flap.move_to(Point::new(center.x - radius * inset, center.y - radius * 0.50));
                flap.line_to(Point::new(
                    center.x,
                    center.y + radius * (0.14 + 0.12 * f64::from(phase.sin())),
                ));
                flap.line_to(Point::new(center.x + radius * inset, center.y - radius * 0.50));
                scene.stroke(&stroke_main, Affine::IDENTITY, accent, None, &flap);
            }
            HoloGlyphKind::DirectoryConstellation => {
                // 3-node living-being constellation orbiting in 10D->3D
                let mut pts = [Point::ZERO; 3];
                for i in 0..3 {
                    let a = f64::from(phase) * 0.75 + (i as f64) * std::f64::consts::TAU / 3.0;
                    let p3 = project_10d_point([a.cos() as f32 * 0.72, a.sin() as f32 * 0.72, 0.2], phase * 0.4, 1.0);
                    pts[i] = Point::new(
                        center.x + f64::from(p3[0]) * radius,
                        center.y + f64::from(p3[1]) * radius,
                    );
                }
                for (a, b) in [(0, 1), (1, 2), (2, 0)] {
                    scene.stroke(
                        &stroke_thin,
                        Affine::IDENTITY,
                        secondary,
                        None,
                        &Line::new(pts[a], pts[b]),
                    );
                }
                for (i, pt) in pts.into_iter().enumerate() {
                    let col = if i == 0 { accent } else { primary };
                    scene.fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        col,
                        None,
                        &Circle::new(pt, radius * 0.23),
                    );
                }
            }
            HoloGlyphKind::BrowserGlobe10d => {
                // Rotating 3D geodesic globe projected via `oracle_project_10d_to_3d`
                scene.stroke(
                    &stroke_main,
                    Affine::IDENTITY,
                    primary,
                    None,
                    &Circle::new(center, radius * 0.88),
                );
                for meridian in [-0.55_f32, 0.0, 0.55] {
                    let mut path = BezPath::new();
                    for step in 0..=14 {
                        let theta =
                            (step as f32 / 14.0) * std::f32::consts::PI - std::f32::consts::FRAC_PI_2;
                        let x = theta.cos() * (meridian + phase * 0.7).sin();
                        let y = theta.sin();
                        let z = theta.cos() * (meridian + phase * 0.7).cos();
                        let p3 = project_10d_point([x, y, z], 0.15, 0.88);
                        let pt = Point::new(
                            center.x + f64::from(p3[0]) * radius,
                            center.y + f64::from(p3[1]) * radius,
                        );
                        if step == 0 {
                            path.move_to(pt);
                        } else {
                            path.line_to(pt);
                        }
                    }
                    scene.stroke(&stroke_thin, Affine::IDENTITY, secondary, None, &path);
                }
                scene.stroke(
                    &stroke_thin,
                    Affine::IDENTITY,
                    accent,
                    None,
                    &Line::new(
                        Point::new(center.x - radius * 0.86, center.y),
                        Point::new(center.x + radius * 0.86, center.y),
                    ),
                );
            }
            HoloGlyphKind::KeepVaultShield => {
                // 4-Layer Sanctuary Vault Shield + rotating Argon2id lock core
                let mut shield = BezPath::new();
                shield.move_to(Point::new(center.x, center.y - radius * 0.88));
                shield.line_to(Point::new(center.x + radius * 0.76, center.y - radius * 0.45));
                shield.line_to(Point::new(center.x + radius * 0.58, center.y + radius * 0.42));
                shield.line_to(Point::new(center.x, center.y + radius * 0.90));
                shield.line_to(Point::new(center.x - radius * 0.58, center.y + radius * 0.42));
                shield.line_to(Point::new(center.x - radius * 0.76, center.y - radius * 0.45));
                shield.close_path();
                scene.stroke(&stroke_main, Affine::IDENTITY, primary, None, &shield);
                let inner_r = radius * (0.34 + 0.06 * f64::from(phase.cos()));
                scene.stroke(
                    &stroke_thin,
                    Affine::IDENTITY,
                    accent,
                    None,
                    &Circle::new(center, inner_r),
                );
            }
            HoloGlyphKind::LibraryTesseract
            | HoloGlyphKind::AdminSentinelCube
            | HoloGlyphKind::ProjectsWorkGraph => {
                // Rotating 3D/10D projected polyhedron / hypercube (`oracle_project_10d_to_3d`)
                let verts: [[f32; 3]; 8] = [
                    [-0.6, -0.6, -0.6],
                    [0.6, -0.6, -0.6],
                    [0.6, 0.6, -0.6],
                    [-0.6, 0.6, -0.6],
                    [-0.6, -0.6, 0.6],
                    [0.6, -0.6, 0.6],
                    [0.6, 0.6, 0.6],
                    [-0.6, 0.6, 0.6],
                ];
                let pts = verts.map(|v| {
                    let p3 = project_10d_point(v, phase * 0.85, 0.92);
                    Point::new(
                        center.x + f64::from(p3[0]) * radius,
                        center.y - f64::from(p3[1]) * radius,
                    )
                });
                let edges = [
                    (0, 1),
                    (1, 2),
                    (2, 3),
                    (3, 0),
                    (4, 5),
                    (5, 6),
                    (6, 7),
                    (7, 4),
                    (0, 4),
                    (1, 5),
                    (2, 6),
                    (3, 7),
                ];
                for (idx, (a, b)) in edges.into_iter().enumerate() {
                    let col = if idx < 4 {
                        primary
                    } else if idx < 8 {
                        secondary
                    } else {
                        accent
                    };
                    scene.stroke(
                        &stroke_thin,
                        Affine::IDENTITY,
                        col,
                        None,
                        &Line::new(pts[a], pts[b]),
                    );
                }
            }
            HoloGlyphKind::InstrumentsOscillator => {
                // Harmonic Lissajous figure-8 oscillator + logic phase needle
                let mut lissajous = BezPath::new();
                for step in 0..=28 {
                    let t = (step as f64 / 28.0) * std::f64::consts::TAU;
                    let x = center.x + (t * 2.0 + f64::from(phase)).sin() * radius * 0.82;
                    let y = center.y + (t * 3.0).cos() * radius * 0.68;
                    if step == 0 {
                        lissajous.move_to(Point::new(x, y));
                    } else {
                        lissajous.line_to(Point::new(x, y));
                    }
                }
                lissajous.close_path();
                scene.stroke(&stroke_main, Affine::IDENTITY, primary, None, &lissajous);
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    accent,
                    None,
                    &Circle::new(center, radius * 0.18),
                );
            }
            HoloGlyphKind::PoetQuillMobius => {
                // Hyperbolic v=2 Mobius ribbon + authoring quill diagonal
                let mut mobius = BezPath::new();
                for step in 0..=24 {
                    let t = (step as f64 / 24.0) * std::f64::consts::TAU;
                    let r = radius * (0.58 + 0.26 * (t * 2.0 + f64::from(phase)).cos());
                    let pt = Point::new(center.x + r * t.cos(), center.y + r * t.sin() * 0.78);
                    if step == 0 {
                        mobius.move_to(pt);
                    } else {
                        mobius.line_to(pt);
                    }
                }
                mobius.close_path();
                scene.stroke(&stroke_main, Affine::IDENTITY, primary, None, &mobius);
                scene.stroke(
                    &stroke_main,
                    Affine::IDENTITY,
                    accent,
                    None,
                    &Line::new(
                        Point::new(center.x - radius * 0.65, center.y + radius * 0.65),
                        Point::new(center.x + radius * 0.68, center.y - radius * 0.68),
                    ),
                );
            }
            HoloGlyphKind::WellfairHeartDyad => {
                // Parametric cardio-dyad curve + pulsing harmonic aura
                let pulse = 1.0 + 0.10 * f64::from((phase * 2.0).sin());
                let mut heart = BezPath::new();
                for step in 0..=28 {
                    let t = (step as f64 / 28.0) * std::f64::consts::TAU;
                    let st = t.sin();
                    let x = 16.0 * st * st * st;
                    let y =
                        13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos();
                    let pt = Point::new(
                        center.x + (x / 18.0) * radius * pulse,
                        center.y - (y / 18.0) * radius * pulse,
                    );
                    if step == 0 {
                        heart.move_to(pt);
                    } else {
                        heart.line_to(pt);
                    }
                }
                heart.close_path();
                scene.stroke(&stroke_main, Affine::IDENTITY, primary, None, &heart);
            }
            HoloGlyphKind::WalletLightningLoop => {
                // Two-Loop Permissive Commons infinity ring + lightning bolt
                let mut bolt = BezPath::new();
                bolt.move_to(Point::new(center.x + radius * 0.15, center.y - radius * 0.88));
                bolt.line_to(Point::new(center.x - radius * 0.45, center.y + radius * 0.06));
                bolt.line_to(Point::new(center.x + radius * 0.08, center.y + radius * 0.06));
                bolt.line_to(Point::new(center.x - radius * 0.18, center.y + radius * 0.88));
                bolt.line_to(Point::new(center.x + radius * 0.48, center.y - radius * 0.08));
                bolt.line_to(Point::new(center.x - radius * 0.05, center.y - radius * 0.08));
                bolt.close_path();
                scene.stroke(&stroke_main, Affine::IDENTITY, accent, None, &bolt);
            }
            HoloGlyphKind::SettingsGearHarmonic => {
                // Rotating 8-tooth precision harmonic gear
                let mut gear = BezPath::new();
                for step in 0..16 {
                    let a = f64::from(phase) * 0.5 + (step as f64) * std::f64::consts::TAU / 16.0;
                    let r = if step % 2 == 0 {
                        radius * 0.90
                    } else {
                        radius * 0.62
                    };
                    let pt = Point::new(center.x + a.cos() * r, center.y + a.sin() * r);
                    if step == 0 {
                        gear.move_to(pt);
                    } else {
                        gear.line_to(pt);
                    }
                }
                gear.close_path();
                scene.stroke(&stroke_main, Affine::IDENTITY, primary, None, &gear);
                scene.stroke(
                    &stroke_thin,
                    Affine::IDENTITY,
                    accent,
                    None,
                    &Circle::new(center, radius * 0.28),
                );
            }
        }

        if ctrl.animating {
            RenderOutcome::ContinuousRedraw
        } else {
            RenderOutcome::Static
        }
    }

    fn handle_event(&mut self, event: &UiEvent, _scale: f64) {
        match event {
            UiEvent::PointerMove(_) => {
                self.hovered = true;
                self.spring_vel[0] += 0.45;
            }
            UiEvent::PointerLeave(_) => {
                self.hovered = false;
            }
            UiEvent::PointerDown(_) => {
                self.spring_vel[0] += 1.15;
            }
            _ => {}
        }
    }

    fn intrinsic_sizes(&self) -> IntrinsicSizes {
        IntrinsicSizes {
            width: Some(30.0),
            height: Some(30.0),
            ratio: Some(1.0),
        }
    }
}

#[derive(Clone)]
struct CachedHoloGlyph {
    attr: CustomWidgetAttr,
    control: Arc<Mutex<HoloGlyphControl>>,
}

/// Evaluate the user's VibeScript icon kinetic formula into an angular frequency `omega`.
pub fn eval_vibe_icon_omega(formula: &str) -> f32 {
    let trimmed = formula.trim();
    if trimmed.is_empty() {
        return 1.8;
    }
    let res = poet_eval_native_cell(trimmed, 0);
    if res.ok {
        res.value
            .trim()
            .parse::<f32>()
            .ok()
            .filter(|v| v.is_finite())
            .map(|v| v.clamp(0.25, 8.0))
            .unwrap_or(1.8)
    } else {
        1.8
    }
}

#[component]
pub fn HoloGlyphIcon(
    kind: HoloGlyphKind,
    #[props(default)] active: bool,
    #[props(default = true)] animating: bool,
    #[props(default = 1.8)] omega: f32,
    #[props(default)] high_contrast: bool,
) -> Element {
    let cache = use_hook(|| {
        let id = NEXT_GLYPH_WIDGET_ID.fetch_add(1, Ordering::Relaxed);
        let phase_seed = ((id % 17) as f32) * 0.37;
        let control = Arc::new(Mutex::new(HoloGlyphControl {
            kind,
            active,
            animating,
            omega,
            high_contrast,
        }));
        let widget = EtalonVectorWidget::new(HoloGlyphDelegate::new(
            Arc::clone(&control),
            phase_seed,
        ));
        let attr = CustomWidgetAttr::with_stable_id(id, widget);
        Rc::new(RefCell::new(CachedHoloGlyph { attr, control }))
    });

    let attr = {
        let entry = cache.borrow();
        if let Ok(mut ctrl) = entry.control.lock() {
            ctrl.kind = kind;
            ctrl.active = active;
            ctrl.animating = animating;
            ctrl.omega = omega;
            ctrl.high_contrast = high_contrast;
        }
        entry.attr.clone()
    };

    rsx! {
        object {
            class: "holo-icon-object",
            aria_hidden: "true",
            data: attr,
        }
    }
}
