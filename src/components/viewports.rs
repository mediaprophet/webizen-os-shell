// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use std::{
    borrow::Cow,
    cell::RefCell,
    future::Future,
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

use aether::prelude::*;
use aether_native::{
    emf_hud_substrate::{EtalonEmfUiSubstrate, HudVertex},
    use_window, use_window_event,
    viewport::{
        EtalonFrameContext, EtalonSurfaceDelegate, EtalonSurfaceError, EtalonSurfaceWidget,
        EtalonViewportState, RenderOutcome,
    },
    winit::{event::WindowEvent, window::Window},
    AetherNativeWindowRenderer, CustomWidgetAttr,
};
use anyrender::{PaintScene, Scene};
use blitz_dom::IntrinsicSizes;
use blitz_traits::events::{BlitzWheelDelta, MouseEventButton, UiEvent};
use kurbo::{Affine, BezPath, Circle, Line, Point, Rect, Stroke};
use peniko::{Color, Fill};
use webizen_ua_os::{
    evaluate_hybrid_page_request, project_emf_ui_field, proxy_corners_for_specimen,
    resolve_holo_10d_specimen, resolve_mode_for_uri, BrowserExecutionMode,
    HoloShaderVisualizationMode, HybridWebCapabilityChannel, SemanticSceneGraph,
    SpatialManifoldTopology, VerifiedHolo10dSpecimen, DEFAULT_HOLO_SPECIMENS, HOLO_MAX_RAYMARCH_STEPS,
    UX_HOLO_SPECIMENS,
};

use crate::components::spatial_stage_3d::pick_semantic_id as pick_graph_semantic_id;
use crate::state::{session, ShellState, VolumeStage};

const MAX_HARMONIC: u8 = 4;
const MAX_STEPS: u32 = HOLO_MAX_RAYMARCH_STEPS;
const UNIFORM_BYTES: u64 = 32;
const HOLO_NODE_STRIDE: u64 = 96;
const HOLO_NODE_COUNT: u64 = 64;
const HOLO_NODE_BYTES: u64 = HOLO_NODE_STRIDE * HOLO_NODE_COUNT;
const EMF_CELL_STRIDE: u64 = 64;
const EMF_CELL_COUNT: u64 = 256;
const EMF_FIELD_BYTES: u64 = EMF_CELL_STRIDE * EMF_CELL_COUNT;
const CYAN: Color = Color::from_rgba8(47, 224, 200, 220);
const VIOLET: Color = Color::from_rgba8(139, 124, 255, 160);
const BACKGROUND: Color = Color::from_rgba8(13, 17, 30, 255);
const CORNERS: [[f32; 3]; 8] = [
    [-0.7, -0.7, -0.7],
    [0.7, -0.7, -0.7],
    [0.7, 0.7, -0.7],
    [-0.7, 0.7, -0.7],
    [-0.7, -0.7, 0.7],
    [0.7, -0.7, 0.7],
    [0.7, 0.7, 0.7],
    [-0.7, 0.7, 0.7],
];
const PROXY_EDGES: [(usize, usize); 12] = [
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
const PROXY_TRIANGLES: [[usize; 3]; 12] = [
    [0, 2, 1],
    [0, 3, 2],
    [4, 5, 6],
    [4, 6, 7],
    [0, 1, 5],
    [0, 5, 4],
    [3, 7, 6],
    [3, 6, 2],
    [0, 4, 7],
    [0, 7, 3],
    [1, 2, 6],
    [1, 6, 5],
];

#[derive(Clone, Debug)]
struct HoloViewportControl {
    specimen: Option<Arc<VerifiedHolo10dSpecimen>>,
    mode: HoloShaderVisualizationMode,
    harmonic_k: u8,
    animating: bool,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    raymarch_steps: u32,
    submitted_extent: Option<(u32, u32)>,
    fallback_extent: Option<(u32, u32)>,
    pick_graph: SemanticSceneGraph,
    last_picked: Option<u64>,
    widget_size: (f32, f32),
}

impl HoloViewportControl {
    fn new(harmonic_k: u8, animating: bool) -> Self {
        Self {
            specimen: None,
            mode: HoloShaderVisualizationMode::PhaseSpaceWavefield,
            harmonic_k: harmonic_k.min(MAX_HARMONIC),
            animating,
            yaw: 0.55,
            pitch: 0.35,
            zoom: 1.0,
            raymarch_steps: 24,
            submitted_extent: None,
            fallback_extent: None,
            pick_graph: SemanticSceneGraph::default(),
            last_picked: None,
            widget_size: (1.0, 1.0),
        }
    }

    fn reset_camera(&mut self) {
        self.yaw = 0.55;
        self.pitch = 0.35;
        self.zoom = 1.0;
    }

    fn orbit(&mut self, x: f32, y: f32) {
        if x.is_finite() && y.is_finite() {
            self.yaw = (self.yaw + x * 0.009).rem_euclid(std::f32::consts::TAU);
            self.pitch = (self.pitch + y * 0.009).clamp(-1.5, 1.5);
        }
    }

    fn zoom_by(&mut self, scroll: f64) {
        if scroll.is_finite() {
            self.zoom =
                (self.zoom * (scroll as f32 * 0.004).clamp(-1.0, 1.0).exp()).clamp(0.35, 2.8);
        }
    }

    fn steps(&self) -> u32 {
        self.raymarch_steps.clamp(1, self.step_limit())
    }

    fn step_limit(&self) -> u32 {
        self.specimen.as_ref().map_or(MAX_STEPS, |specimen| {
            u32::from(specimen.max_raymarch_steps).clamp(1, MAX_STEPS)
        })
    }

    fn harmonic_limit(&self) -> u8 {
        self.specimen.as_ref().map_or(MAX_HARMONIC, |specimen| {
            specimen.max_harmonic_order.min(MAX_HARMONIC)
        })
    }

    fn harmonic(&self) -> u8 {
        self.harmonic_k.min(self.harmonic_limit())
    }

    fn projection_bounds(&self) -> [[f32; 3]; 8] {
        let Some(specimen) = &self.specimen else {
            return CORNERS;
        };
        let min = specimen.proxy_aabb_min.map(f64::from);
        let max = specimen.proxy_aabb_max.map(f64::from);
        // Center the camera on the specimen and scale profiles larger than the viewport.
        let center: [f64; 3] = std::array::from_fn(|axis| (min[axis] + max[axis]) * 0.5);
        let divisor = (0..3)
            .map(|axis| (max[axis] - min[axis]) * 0.5)
            .fold(1.0_f64, f64::max);
        proxy_corners_for_specimen(specimen).map(|corner| {
            std::array::from_fn(|axis| ((f64::from(corner[axis]) - center[axis]) / divisor) as f32)
        })
    }

    fn project(&self, [x, y, z]: [f32; 3]) -> [f32; 2] {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let x1 = cy * x + sy * z;
        let z1 = -sy * x + cy * z;
        [x1 * self.zoom, (cp * y - sp * z1) * self.zoom]
    }
}

fn controls(control: &Mutex<HoloViewportControl>) -> MutexGuard<'_, HoloViewportControl> {
    control.lock().unwrap_or_else(|error| error.into_inner())
}

struct HoloGpu {
    pipeline: etalon::RenderPipeline,
    uniform: etalon::Buffer,
    bindings: etalon::BindGroup,
    nodes: etalon::Buffer,
    node_bindings: etalon::BindGroup,
    emf: etalon::Buffer,
    emf_bindings: etalon::BindGroup,
}

/// Etalon delegate for the bounded holographic phase-space projection preview.
pub struct WebizenHolo10dSurfaceDelegate {
    control: Arc<Mutex<HoloViewportControl>>,
    window: Arc<dyn Window>,
    gpu: Option<HoloGpu>,
    dragging_from: Option<(f32, f32)>,
    focused: bool,
    phase: f32,
    previous_frame: Option<Instant>,
}

impl WebizenHolo10dSurfaceDelegate {
    fn new(control: Arc<Mutex<HoloViewportControl>>, window: Arc<dyn Window>) -> Self {
        Self {
            control,
            window,
            gpu: None,
            dragging_from: None,
            focused: false,
            phase: 0.0,
            previous_frame: None,
        }
    }

    fn frame_phase(&mut self, animating: bool) -> f32 {
        let now = Instant::now();
        if animating {
            if let Some(previous) = self.previous_frame.replace(now) {
                self.phase = (self.phase
                    + now
                        .duration_since(previous)
                        .min(Duration::from_millis(100))
                        .as_secs_f32()
                        * 2.0)
                    .rem_euclid(std::f32::consts::TAU);
            }
        } else {
            self.previous_frame = None;
        }
        self.phase
    }
}

fn uniform_data(
    view: &HoloViewportControl,
    width: u32,
    height: u32,
    phase: f32,
) -> [u8; UNIFORM_BYTES as usize] {
    let mut bytes = [0; UNIFORM_BYTES as usize];
    let bounds = view.projection_bounds();
    let [x, y, z] = bounds[6].map(|axis| (axis.clamp(0.0, 1.0) * 1023.0).round() as u32);
    let packed_bounds = x | (y << 10) | (z << 20);
    let sigma = view
        .specimen
        .as_ref()
        .map_or(0.56, |spec| spec.reference_sigma);
    let packed_view = view.steps()
        | (u32::from(view.harmonic()) << 6)
        | (view.mode.shader_mode_index() << 9)
        | (((sigma * 255.0).round() as u32) << 11);
    let words = [
        view.yaw.to_bits(),
        view.pitch.to_bits(),
        view.zoom.to_bits(),
        phase.to_bits(),
        (width as f32).to_bits(),
        (height as f32).to_bits(),
        packed_bounds,
        packed_view,
    ];
    for (lane, word) in bytes.chunks_exact_mut(4).zip(words) {
        lane.copy_from_slice(&word.to_le_bytes());
    }
    bytes
}

fn projected_emf_field(view: &HoloViewportControl) -> webizen_ua_os::EmfUiField {
    project_emf_ui_field(
        &view.pick_graph,
        SpatialManifoldTopology::OrbitalVolumetricAmphitheatre10d,
    )
}

fn holo_node_bytes(view: &HoloViewportControl) -> Vec<u8> {
    projected_emf_field(view).holo_node_bytes()
}

fn emf_field_bytes(view: &HoloViewportControl) -> Vec<u8> {
    projected_emf_field(view).emf_cell_bytes()
}

fn etalon_scope_result(scope: etalon::ErrorScopeGuard) -> Result<(), EtalonSurfaceError> {
    let mut result = std::pin::pin!(scope.pop());
    match result
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(None) => Ok(()),
        Poll::Ready(Some(error)) => Err(EtalonSurfaceError::RenderPassFailed(error.to_string())),
        Poll::Pending => Err(EtalonSurfaceError::RenderPassFailed(
            "Etalon validation has not completed".into(),
        )),
    }
}

impl EtalonSurfaceDelegate for WebizenHolo10dSurfaceDelegate {
    fn prepare(
        &mut self,
        device: &etalon::Device,
        _: &etalon::Queue,
    ) -> Result<(), EtalonSurfaceError> {
        self.gpu = None;
        let oom = device.push_error_scope(etalon::ErrorFilter::OutOfMemory);
        let validation = device.push_error_scope(etalon::ErrorFilter::Validation);
        let layout = device.create_bind_group_layout(&etalon::BindGroupLayoutDescriptor {
            label: Some("Webizen HOLO uniform binding"),
            entries: &[etalon::BindGroupLayoutEntry {
                binding: 0,
                visibility: etalon::ShaderStages::FRAGMENT,
                ty: etalon::BindingType::Buffer {
                    ty: etalon::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: etalon::BufferSize::new(UNIFORM_BYTES),
                },
                count: None,
            }],
        });
        let node_layout = device.create_bind_group_layout(&etalon::BindGroupLayoutDescriptor {
            label: Some("Webizen HOLO node storage"),
            entries: &[etalon::BindGroupLayoutEntry {
                binding: 0,
                visibility: etalon::ShaderStages::FRAGMENT,
                ty: etalon::BindingType::Buffer {
                    ty: etalon::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: etalon::BufferSize::new(HOLO_NODE_BYTES),
                },
                count: None,
            }],
        });
        let emf_layout = device.create_bind_group_layout(&etalon::BindGroupLayoutDescriptor {
            label: Some("Webizen EMF field storage"),
            entries: &[etalon::BindGroupLayoutEntry {
                binding: 0,
                visibility: etalon::ShaderStages::FRAGMENT,
                ty: etalon::BindingType::Buffer {
                    ty: etalon::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: etalon::BufferSize::new(EMF_FIELD_BYTES),
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&etalon::PipelineLayoutDescriptor {
            label: Some("Webizen HOLO pipeline layout"),
            bind_group_layouts: &[Some(&layout), Some(&node_layout), Some(&emf_layout)],
            ..Default::default()
        });
        let uniform = device.create_buffer(&etalon::BufferDescriptor {
            label: Some("Webizen HOLO camera and budget"),
            size: UNIFORM_BYTES,
            usage: etalon::BufferUsages::UNIFORM | etalon::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let nodes = device.create_buffer(&etalon::BufferDescriptor {
            label: Some("Webizen HOLO holographic nodes"),
            size: HOLO_NODE_BYTES,
            usage: etalon::BufferUsages::STORAGE | etalon::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let emf = device.create_buffer(&etalon::BufferDescriptor {
            label: Some("Webizen EMF field cells"),
            size: EMF_FIELD_BYTES,
            usage: etalon::BufferUsages::STORAGE | etalon::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bindings = device.create_bind_group(&etalon::BindGroupDescriptor {
            label: Some("Webizen HOLO uniform group"),
            layout: &layout,
            entries: &[etalon::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        let node_bindings = device.create_bind_group(&etalon::BindGroupDescriptor {
            label: Some("Webizen HOLO node group"),
            layout: &node_layout,
            entries: &[etalon::BindGroupEntry {
                binding: 0,
                resource: nodes.as_entire_binding(),
            }],
        });
        let emf_bindings = device.create_bind_group(&etalon::BindGroupDescriptor {
            label: Some("Webizen EMF field group"),
            layout: &emf_layout,
            entries: &[etalon::BindGroupEntry {
                binding: 0,
                resource: emf.as_entire_binding(),
            }],
        });
        let shader = device.create_shader_module(etalon::ShaderModuleDescriptor {
            label: Some("Webizen HOLO preview"),
            source: etalon::ShaderSource::Wgsl(Cow::Borrowed(HOLO_SHADER)),
        });
        let pipeline = device.create_render_pipeline(&etalon::RenderPipelineDescriptor {
            label: Some("Webizen HOLO preview pipeline"),
            layout: Some(&pipeline_layout),
            vertex: etalon::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(etalon::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(etalon::ColorTargetState {
                    format: etalon::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: etalon::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let validated = etalon_scope_result(validation);
        let allocated = etalon_scope_result(oom);
        validated.and(allocated)?;
        self.gpu = Some(HoloGpu {
            pipeline,
            uniform,
            bindings,
            nodes,
            node_bindings,
            emf,
            emf_bindings,
        });
        self.window.request_redraw();
        Ok(())
    }

    fn render_frame(
        &mut self,
        ctx: &EtalonFrameContext<'_>,
    ) -> Result<RenderOutcome, EtalonSurfaceError> {
        let view = controls(&self.control).clone();
        let phase = self.frame_phase(view.animating);
        let gpu = self
            .gpu
            .as_ref()
            .ok_or(EtalonSurfaceError::NoGpuContextAvailable)?;
        let validation = ctx.device.push_error_scope(etalon::ErrorFilter::Validation);
        ctx.queue.write_buffer(
            &gpu.uniform,
            0,
            &uniform_data(&view, ctx.width, ctx.height, phase),
        );
        ctx.queue.write_buffer(&gpu.nodes, 0, &holo_node_bytes(&view));
        ctx.queue.write_buffer(&gpu.emf, 0, &emf_field_bytes(&view));
        controls(&self.control).widget_size = (ctx.width as f32, ctx.height as f32);
        let mut encoder = ctx
            .device
            .create_command_encoder(&etalon::CommandEncoderDescriptor {
                label: Some("Webizen HOLO frame"),
            });
        {
            let attachments = [Some(etalon::RenderPassColorAttachment {
                view: ctx.target_view,
                depth_slice: None,
                resolve_target: None,
                ops: etalon::Operations {
                    load: etalon::LoadOp::Clear(etalon::Color::BLACK),
                    store: etalon::StoreOp::Store,
                },
            })];
            let mut pass = encoder.begin_render_pass(&etalon::RenderPassDescriptor {
                label: Some("Webizen HOLO projection"),
                color_attachments: &attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&gpu.pipeline);
            pass.set_bind_group(0, &gpu.bindings, &[]);
            pass.set_bind_group(1, &gpu.node_bindings, &[]);
            pass.set_bind_group(2, &gpu.emf_bindings, &[]);
            pass.draw(0..3, 0..1);
        }
        ctx.queue.submit([encoder.finish()]);
        etalon_scope_result(validation)?;
        let extent = Some((ctx.width, ctx.height));
        if view.submitted_extent != extent {
            controls(&self.control).submitted_extent = extent;
            self.window.request_redraw();
        }
        Ok(if view.animating {
            RenderOutcome::ContinuousRedraw
        } else {
            RenderOutcome::Static
        })
    }

    fn paint_fallback(&mut self, width: u32, height: u32, scale: f64) -> Scene {
        let view = controls(&self.control).clone();
        let phase = self.frame_phase(view.animating);
        let mut scene = Scene::new();
        let fallback_extent = Some((width, height));
        let (width, height) = (f64::from(width), f64::from(height));
        let extent = width.min(height);
        let origin = Point::new(width * 0.5, height * 0.5);
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            BACKGROUND,
            None,
            &Rect::new(0.0, 0.0, width, height),
        );
        paint_cpu_hud_fallback(&mut scene, width, height);
        let points = view.projection_bounds().map(|corner| {
            let [x, y] = view.project(corner);
            Point::new(
                origin.x + f64::from(x) * extent * 0.27,
                origin.y - f64::from(y) * extent * 0.27,
            )
        });
        let line = Stroke::new(scale.clamp(1.0, 2.0));
        paint_shader_fallback(&mut scene, &view, origin, extent, phase, &line);
        for (a, b) in PROXY_EDGES {
            scene.stroke(
                &line,
                Affine::IDENTITY,
                CYAN,
                None,
                &Line::new(points[a], points[b]),
            );
        }
        let diagonal = Stroke::new(line.width * 0.7);
        for triangle in PROXY_TRIANGLES {
            let [a, b, c] = triangle;
            if let Some((from, to)) = [(a, b), (b, c), (c, a)].into_iter().find(|edge| {
                !PROXY_EDGES
                    .iter()
                    .any(|&(a, b)| *edge == (a, b) || *edge == (b, a))
            }) {
                scene.stroke(
                    &diagonal,
                    Affine::IDENTITY,
                    Color::from_rgba8(47, 224, 200, 85),
                    None,
                    &Line::new(points[from], points[to]),
                );
            }
        }
        if view.fallback_extent != fallback_extent {
            controls(&self.control).fallback_extent = fallback_extent;
            self.window.request_redraw();
        } else if view.animating {
            self.window.request_redraw();
        }
        scene
    }

    fn handle_event(&mut self, event: &UiEvent, _: f64) {
        let mut view = controls(&self.control);
        let repaint = match event {
            UiEvent::PointerDown(pointer)
                if pointer.is_primary && pointer.button == MouseEventButton::Main =>
            {
                let (width, height) = view.widget_size;
                if let Some(id) = pick_graph_semantic_id(
                    &view.pick_graph,
                    pointer.coords.client_x as f32,
                    pointer.coords.client_y as f32,
                    width,
                    height,
                ) {
                    view.last_picked = Some(id);
                }
                self.dragging_from = Some((pointer.coords.client_x, pointer.coords.client_y));
                false
            }
            UiEvent::PointerMove(pointer) if pointer.is_primary && self.dragging_from.is_some() => {
                let next = (pointer.coords.client_x, pointer.coords.client_y);
                if let Some((x, y)) = self.dragging_from.replace(next) {
                    view.orbit(next.0 - x, next.1 - y);
                }
                true
            }
            UiEvent::PointerUp(_) | UiEvent::PointerCancel(_) | UiEvent::PointerLeave(_) => {
                self.dragging_from = None;
                false
            }
            UiEvent::Wheel(wheel) => {
                let delta = match wheel.delta {
                    BlitzWheelDelta::Lines(_, y) => y * 38.0,
                    BlitzWheelDelta::Pixels(_, y) => y,
                };
                view.zoom_by(delta);
                true
            }
            UiEvent::KeyDown(key) if self.focused && !key.is_auto_repeating => {
                match key.key.to_string().as_str() {
                    " " => {
                        view.animating = !view.animating;
                        true
                    }
                    "r" | "R" => {
                        view.reset_camera();
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        };
        drop(view);
        if repaint {
            self.window.request_redraw();
        }
    }

    fn focus_changed(&mut self, focused: bool) {
        self.focused = focused;
        if !focused {
            self.dragging_from = None;
        }
    }

    fn intrinsic_sizes(&self) -> IntrinsicSizes {
        IntrinsicSizes {
            width: Some(560.0),
            height: Some(320.0),
            ratio: Some(560.0 / 320.0),
        }
    }

    fn destroy(&mut self) {
        self.gpu = None;
        self.dragging_from = None;
        self.previous_frame = None;
        let mut control = controls(&self.control);
        control.submitted_extent = None;
        control.fallback_extent = None;
    }

    fn pick_semantic_id(
        &self,
        local_x: f32,
        local_y: f32,
        width: f32,
        height: f32,
    ) -> Option<u64> {
        let view = controls(&self.control);
        pick_graph_semantic_id(&view.pick_graph, local_x, local_y, width, height)
    }
}

fn paint_cpu_hud_fallback(scene: &mut Scene, width: f64, height: f64) {
    let mut substrate = EtalonEmfUiSubstrate::new();
    substrate.rebuild_workspace_chrome(width as f32, height as f32);
    let mut verts = [HudVertex {
        pos: [0.0; 2],
        uv: [0.0; 2],
        color: [0.0; 4],
    }; 24];
    let count = substrate.generate_hud_vertices(&mut verts);
    for quad in verts[..count].chunks_exact(6) {
        let color = quad[0].color;
        let fill = Color::from_rgba8(
            (color[0].clamp(0.0, 1.0) * 255.0) as u8,
            (color[1].clamp(0.0, 1.0) * 255.0) as u8,
            (color[2].clamp(0.0, 1.0) * 255.0) as u8,
            (color[3].clamp(0.0, 1.0) * 255.0) as u8,
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            fill,
            None,
            &Rect::new(
                f64::from(quad[0].pos[0]),
                f64::from(quad[0].pos[1]),
                f64::from(quad[2].pos[0]),
                f64::from(quad[2].pos[1]),
            ),
        );
    }
}

fn paint_shader_fallback(
    scene: &mut Scene,
    view: &HoloViewportControl,
    origin: Point,
    extent: f64,
    phase: f32,
    stroke: &Stroke,
) {
    use HoloShaderVisualizationMode::*;
    let radius = extent * f64::from(view.zoom) * 0.32;
    let phase = f64::from(phase);
    let harmonic = view.harmonic();
    match view.mode {
        PhaseSpaceWavefield => {
            for order in 0..=harmonic {
                let radial =
                    0.21 * f64::from(order + 1) + 0.025 * (phase + f64::from(order) * 0.55).sin();
                let circle = Circle::new(origin, radius * radial);
                scene.stroke(stroke, Affine::IDENTITY, VIOLET, None, &circle);
            }
        }
        HyperbolicMobiusV2 => {
            let a = 0.33 + 0.11 * phase.cos();
            for order in 0..=harmonic {
                let ring = 0.15 + f64::from(order) * 0.17;
                let curve = closed_curve(|angle| {
                    let (y, x) = angle.sin_cos();
                    let (x, y) = (x * ring, y * ring);
                    let (dx, dy) = (1.0 + a * x, a * y);
                    let quotient = dx.mul_add(dx, dy * dy);
                    Point::new(
                        origin.x + radius * ((x + a) * dx + y * dy) / quotient,
                        origin.y - radius * (y * dx - (x + a) * dy) / quotient,
                    )
                });
                scene.stroke(stroke, Affine::IDENTITY, VIOLET, None, &curve);
            }
        }
        AnatomicalRadialDepth => {
            let colors = [
                Color::from_rgba8(250, 149, 110, 160),
                Color::from_rgba8(232, 111, 143, 170),
                Color::from_rgba8(159, 98, 221, 180),
                Color::from_rgba8(95, 118, 247, 195),
            ];
            let half = view.projection_bounds()[6];
            for (index, color) in colors.into_iter().enumerate().rev() {
                let contour = closed_curve(|angle| {
                    let (y, x) = angle.sin_cos();
                    let level = (index as f64 + 1.0) * 0.25;
                    let [x, y] = view.project([
                        (x * f64::from(half[0]) * level) as f32,
                        (y * f64::from(half[1]) * level) as f32,
                        0.0,
                    ]);
                    Point::new(
                        origin.x + extent * 0.27 * f64::from(x),
                        origin.y - extent * 0.27 * f64::from(y),
                    )
                });
                scene.stroke(stroke, Affine::IDENTITY, color, None, &contour);
            }
        }
        ScaleSpaceHarmonicSlice => {
            for order in 0..=harmonic {
                let contour = closed_curve(|angle| {
                    let (y, x) = angle.sin_cos();
                    let petals = (angle * f64::from(2 + order) + phase).cos();
                    let r = radius * (0.19 + f64::from(order) * 0.16 + 0.08 * petals);
                    Point::new(origin.x + r * x, origin.y - r * y)
                });
                scene.stroke(stroke, Affine::IDENTITY, VIOLET, None, &contour);
            }
        }
    }
}

fn closed_curve(mut project: impl FnMut(f64) -> Point) -> BezPath {
    const SEGMENTS: usize = 72;
    let mut path = BezPath::new();
    path.move_to(project(0.0));
    for segment in 1..SEGMENTS {
        path.line_to(project(
            segment as f64 * std::f64::consts::TAU / SEGMENTS as f64,
        ));
    }
    path.close_path();
    path
}

const HOLO_SHADER: &str = r#"
struct Frame {
    camera: vec4<f32>,
    dimensions: vec2<f32>,
    packed_bounds: u32,
    packed_view: u32,
}
@group(0) @binding(0) var<uniform> frame: Frame;

struct HolographicNodeGpu {
    position_scale: vec4<f32>,
    half_extents_primitive: vec4<f32>,
    frequency_amplitude: vec4<f32>,
    manifold_tensor: vec4<f32>,
    connection_endpoint: vec4<f32>,
    emissive_color: vec4<f32>,
}
@group(1) @binding(0) var<storage, read> nodes: array<HolographicNodeGpu>;

struct EmfFieldCell {
    amplitude: f32,
    phase: f32,
    frequency: f32,
    scale: f32,
    attention_depth: f32,
    epistemic_weight: f32,
    topological_spin: f32,
    temporal_decay: f32,
    entropy_bias: f32,
    spatial_phase: f32,
    recurrence_frequency: f32,
    density_threshold: f32,
    manifold_curvature: f32,
    stokes_q: f32,
    stokes_u: f32,
    stokes_v: f32,
}
@group(2) @binding(0) var<storage, read> emf: array<EmfFieldCell>;

const corners = array<vec3<f32>, 8>(
    vec3(-1.0,-1.0,-1.0), vec3(1.0,-1.0,-1.0), vec3(1.0,1.0,-1.0), vec3(-1.0,1.0,-1.0),
    vec3(-1.0,-1.0,1.0), vec3(1.0,-1.0,1.0), vec3(1.0,1.0,1.0), vec3(-1.0,1.0,1.0)
);
const edges = array<vec2<u32>, 12>(
    vec2(0u,1u), vec2(1u,2u), vec2(2u,3u), vec2(3u,0u), vec2(4u,5u), vec2(5u,6u),
    vec2(6u,7u), vec2(7u,4u), vec2(0u,4u), vec2(1u,5u), vec2(2u,6u), vec2(3u,7u)
);
const diagonals = array<vec2<u32>, 6>(
    vec2(0u,2u), vec2(4u,6u), vec2(0u,5u), vec2(3u,6u), vec2(0u,7u), vec2(1u,6u)
);

@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let xy = array<vec2<f32>, 3>(vec2(-1.0,-1.0), vec2(3.0,-1.0), vec2(-1.0,3.0));
    return vec4(xy[i], 0.0, 1.0);
}

fn project(p: vec3<f32>) -> vec2<f32> {
    let yr = frame.camera.x;
    let pr = frame.camera.y;
    let x = cos(yr) * p.x + sin(yr) * p.z;
    let z = -sin(yr) * p.x + cos(yr) * p.z;
    return vec2(x, cos(pr) * p.y - sin(pr) * z) * (0.27 * frame.camera.z);
}

fn line_distance(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let d = b - a;
    return length(p - a - clamp(dot(p - a,d) / max(dot(d,d), 0.0000001), 0.0, 1.0) * d);
}

fn inverse_orbit(p: vec3<f32>) -> vec3<f32> {
    let yr = frame.camera.x;
    let pr = frame.camera.y;
    let y = cos(pr) * p.y + sin(pr) * p.z;
    let z = -sin(pr) * p.y + cos(pr) * p.z;
    return vec3(cos(yr) * p.x - sin(yr) * z, y, sin(yr) * p.x + cos(yr) * z);
}

fn box_distance(p: vec3<f32>, half_extent: vec3<f32>) -> f32 {
    let d = abs(p) - half_extent;
    return length(max(d, vec3(0.0))) + min(max(d.x, max(d.y, d.z)), 0.0);
}

fn smin_cubic(a: f32, b: f32, k: f32) -> f32 {
    let h = max(k - abs(a - b), 0.0) / max(k, 0.0001);
    return min(a, b) - h * h * h * k * (1.0 / 6.0);
}

fn torus_sdf(p: vec3<f32>, t: vec2<f32>) -> f32 {
    let q = vec2(length(p.xz) - t.x, p.y);
    return length(q) - t.y;
}

fn holo_scene_sdf(p: vec3<f32>, half_extent: vec3<f32>, harmonic: u32, mode: u32, phase: f32) -> f32 {
    let scale = max(min(half_extent.x, min(half_extent.y, half_extent.z)), 0.25);
    let q = p / scale;
    let core = length(q) - 0.44;
    let orbit_a = vec3(cos(phase) * 0.42, sin(phase * 0.7) * 0.24, sin(phase) * 0.42);
    let orbit_b = vec3(-cos(phase * 0.8) * 0.38, -sin(phase) * 0.26, cos(phase * 0.8) * 0.38);
    let sat_a = length(q - orbit_a) - 0.21;
    let sat_b = length(q - orbit_b) - 0.19;
    let ring_p = vec3(q.x, q.y * cos(0.45) - q.z * sin(0.45), q.y * sin(0.45) + q.z * cos(0.45));
    let torus = torus_sdf(ring_p, vec2(0.58, 0.075));
    var d = smin_cubic(core, smin_cubic(sat_a, sat_b, 0.24), 0.28);
    let n_count = min(arrayLength(&nodes), 4u);
    for (var ni: u32 = 1u; ni < n_count; ni += 1u) {
        let gnode = nodes[ni];
        let gpos = gnode.position_scale.xyz * 0.45;
        let grad = clamp(gnode.position_scale.w * 0.18, 0.10, 0.22);
        let gd = length(q - gpos) - grad;
        d = smin_cubic(d, gd, 0.18);
    }
    if mode != 2u {
        d = smin_cubic(d, torus, 0.16);
    }
    if harmonic >= 1u {
        let freq = 4.0 + f32(harmonic) * 2.5;
        let disp = 0.028 * f32(harmonic) * sin(q.x * freq + phase) * cos(q.y * freq - phase) * sin(q.z * freq);
        d += disp;
    }
    return max(d * scale, box_distance(p, half_extent * 1.02));
}

fn estimate_normal(p: vec3<f32>, half_extent: vec3<f32>, harmonic: u32, mode: u32, phase: f32) -> vec3<f32> {
    let e = vec2(0.004, -0.004);
    return normalize(
        e.xyy * holo_scene_sdf(p + e.xyy, half_extent, harmonic, mode, phase) +
        e.yyx * holo_scene_sdf(p + e.yyx, half_extent, harmonic, mode, phase) +
        e.yxy * holo_scene_sdf(p + e.yxy, half_extent, harmonic, mode, phase) +
        e.xxx * holo_scene_sdf(p + e.xxx, half_extent, harmonic, mode, phase)
    );
}

fn mobius(z: vec2<f32>, a: f32) -> vec2<f32> {
    let d = vec2(1.0 + a * z.x, a * z.y);
    let x = vec2(z.x + a, z.y);
    return vec2(dot(x, d), x.y * d.x - x.x * d.y) / max(dot(d, d), 0.0001);
}

@fragment fn fs_main(@builtin(position) pixel: vec4<f32>) -> @location(0) vec4<f32> {
    let size = frame.dimensions;
    let extent = max(min(size.x, size.y), 1.0);
    let uv = (pixel.xy - size * 0.5) * vec2(1.0, -1.0) / extent;
    let packed = frame.packed_view;
    let step_budget = clamp(packed & 63u, 1u, 32u);
    let harmonic = min((packed >> 6u) & 7u, 4u);
    let mode = (packed >> 9u) & 3u;
    let sigma = f32((packed >> 11u) & 255u) / 255.0;
    let bounds = frame.packed_bounds;
    let half_extent = vec3<f32>(
        f32(bounds & 1023u),
        f32((bounds >> 10u) & 1023u),
        f32((bounds >> 20u) & 1023u)
    ) / 1023.0;
    let zoom = frame.camera.z;
    let phase = frame.camera.w;
    var ray_depth = -1.75;
    var wave = 0.0;
    var radial_depth = 0.0;
    var sdf_lit = vec3(0.0);
    var sdf_alpha = 0.0;
    let ray_origin = inverse_orbit(vec3(uv / (0.27 * zoom), -2.0));
    let ray_target = inverse_orbit(vec3(uv / (0.27 * zoom), 1.5));
    let view_dir = normalize(ray_target - ray_origin);
    for (var hit = 0u; hit < step_budget; hit += 1u) {
        let p = inverse_orbit(vec3(uv / (0.27 * zoom), ray_depth));
        let distance = box_distance(p, half_extent);
        let sdf_d = holo_scene_sdf(p, half_extent, harmonic, mode, phase);
        if sdf_alpha < 0.95 && sdf_d < 0.012 {
            let n = estimate_normal(p, half_extent, harmonic, mode, phase);
            let light_dir = normalize(vec3(0.55, 0.78, -0.45));
            let diff = max(dot(n, light_dir), 0.0);
            let half_v = normalize(light_dir - view_dir);
            let spec = pow(max(dot(n, half_v), 0.0), 28.0);
            let fresnel = pow(1.0 - max(dot(n, -view_dir), 0.0), 2.6);
            // Spherical Harmonics L0/L1 irradiance
            let sh_irr = 0.52 + 0.28 * n.y + 0.18 * n.x - 0.12 * n.z;
            let base_col = select(
                vec3(0.14, 0.78, 0.72),
                mix(vec3(0.38, 0.46, 0.96), vec3(0.98, 0.56, 0.38), clamp(length(p), 0.0, 1.0)),
                mode == 2u
            );
            sdf_lit = base_col * (sh_irr * 0.55 + diff * 0.75)
                + vec3(0.55, 0.92, 1.0) * (spec * 0.65 + fresnel * 0.52);
            sdf_alpha = 1.0;
        }
        if distance < 0.005 {
            let a = cos(length(p) * (5.0 + f32(harmonic) * 2.0) - phase);
            let b = cos(dot(p, vec3(3.5, 1.25, 2.0)) + phase);
            wave = max(wave, max(0.0, a * b));
            let unit_box = abs(p) / max(half_extent, vec3(0.002));
            radial_depth = max(radial_depth, clamp(1.0 - length(unit_box) * 0.57735, 0.0, 1.0));
        }
        let step_len = min(max(distance, 0.0), max(sdf_d * 0.75, 0.02));
        ray_depth += max(step_len, 3.5 / f32(step_budget));
        if ray_depth > 1.75 { break; }
    }
    let width_px = 1.0 / extent;
    let z = uv / (0.32 * zoom);
    let warped = length(mobius(z, -(0.33 + 0.11 * cos(phase))));
    let theta_vector = select(vec2(0.0001, 0.0), uv, length(uv) > 0.0001);
    let theta = atan2(theta_vector.y, theta_vector.x);
    var rings = 0.0;
    for (var order = 0u; order <= harmonic; order += 1u) {
        var distance = abs(length(uv) -
            (0.068 * f32(order + 1u) + 0.008 * sin(phase + f32(order) * 0.55)) * zoom);
        if mode == 1u {
            distance = abs(warped - (0.15 + 0.17 * f32(order))) * 0.32 * zoom;
        }
        if mode == 2u {
            distance = abs(radial_depth - 0.19 * f32(order + 1u)) * 0.32 * zoom;
        }
        if mode == 3u {
            let petals = cos(theta * f32(2u + order) + phase);
            distance = abs(length(z) - (0.19 + 0.16 * f32(order) + 0.08 * petals)) * 0.32 * zoom;
        }
        rings = max(rings, 1.0 - smoothstep(width_px * 0.55, width_px * 1.6, distance));
    }
    var wire = 0.0;
    for (var e = 0u; e < 12u; e += 1u) {
        let segment = edges[e];
        let distance = line_distance(uv, project(corners[segment.x] * half_extent),
            project(corners[segment.y] * half_extent));
        wire = max(wire, 1.0 - smoothstep(width_px * 0.55, width_px * 1.65, distance));
    }
    for (var e = 0u; e < 6u; e += 1u) {
        let segment = diagonals[e];
        let distance = line_distance(uv, project(corners[segment.x] * half_extent),
            project(corners[segment.y] * half_extent));
        wire = max(wire, (1.0 - smoothstep(width_px * 0.35, width_px * 1.35, distance)) * 0.4);
    }
    var rgb = vec3(0.050, 0.067, 0.118) + vec3(0.12, 0.06, 0.19) * wave * (0.5 + sigma);
    rgb = mix(rgb, sdf_lit, sdf_alpha * 0.88);
    var ring_color = vec3(0.55, 0.49, 1.0);
    if mode == 2u {
        ring_color = mix(vec3(0.48, 0.39, 1.0), vec3(1.0, 0.58, 0.40), radial_depth);
        rgb += ring_color * radial_depth * (0.12 + 0.14 * sigma);
    }
    rgb = mix(rgb, ring_color, 0.70 * rings);
    rgb = mix(rgb, vec3(0.18, 0.88, 0.78), 0.86 * wire);
    if arrayLength(&nodes) > 0u {
        rgb = mix(rgb, nodes[0].emissive_color.xyz, 0.08);
    }
    let emf_len = arrayLength(&emf);
    if emf_len > 0u {
        let norm_uv = clamp(pixel.xy / max(size, vec2(1.0)), vec2(0.0), vec2(0.999));
        let gx = min(u32(norm_uv.x * 16.0), 15u);
        let gy = min(u32(norm_uv.y * 16.0), 15u);
        let cell_idx = min(gy * 16u + gx, emf_len - 1u);
        let cell = emf[cell_idx];
        let pol = length(vec3(cell.stokes_q, cell.stokes_u, cell.stokes_v));
        let wave = vec3(cell.amplitude, cell.attention_depth, cell.epistemic_weight);
        rgb = mix(rgb, wave, 0.05 + 0.04 * pol + 0.03 * abs(cell.manifold_curvature));
    }
    return vec4(rgb, 1.0);
}
"#;

#[derive(Clone)]
struct CachedViewport {
    stable_id: usize,
    attr: CustomWidgetAttr,
    control: Arc<Mutex<HoloViewportControl>>,
    read_surface: Rc<dyn Fn() -> EtalonViewportState>,
}

fn make_viewport(
    stable_id: usize,
    control: HoloViewportControl,
    window: Arc<dyn Window>,
) -> CachedViewport {
    let control = Arc::new(Mutex::new(control));
    let widget = EtalonSurfaceWidget::new(WebizenHolo10dSurfaceDelegate::new(
        Arc::clone(&control),
        window,
    ));
    let read_surface = Rc::new(widget.state_reader());
    let attr = CustomWidgetAttr::with_stable_id(stable_id, widget);
    CachedViewport {
        stable_id,
        attr,
        control,
        read_surface,
    }
}

#[component]
pub fn Holo10dViewport(
    state: Signal<ShellState>,
    specimen_label: String,
    #[props(default)] specimen_id: String,
) -> Element {
    let package = match state.read().current_stage {
        VolumeStage::Browser => Some("Browser"),
        VolumeStage::Poet => Some("Poet"),
        VolumeStage::Anatomy => Some("Anatomy"),
        VolumeStage::Library => Some("Library"),
        _ => None,
    };
    if session().is_some()
        && package.is_some_and(|package| {
            state.read().blocks_live_qap_lease(
                package,
                webizen_ua_os::CapabilityLeaseScope::EtalonGpuSurface,
            )
        })
    {
        return rsx! {
            p { role: "status", "The installed Qap policy blocks this Etalon surface." }
        };
    }
    rsx! { AllowedHolo10dViewport { state, specimen_label, specimen_id } }
}

#[component]
fn AllowedHolo10dViewport(
    mut state: Signal<ShellState>,
    specimen_label: String,
    specimen_id: String,
) -> Element {
    let window = use_window();
    let renderer = try_consume_context::<AetherNativeWindowRenderer>();
    let cache = use_hook(|| Rc::new(RefCell::new(None::<CachedViewport>)));
    let mut display = use_signal(|| String::from("Viewport waiting for native surface"));
    let mut budget_display = use_signal(|| 24u32);
    let mut applied_route = use_signal(|| None::<String>);
    let mut resolved =
        use_signal(|| None::<(String, Result<Arc<VerifiedHolo10dSpecimen>, String>)>);
    let specimen_id = specimen_id.trim().to_owned();
    let (id, initial_k, initial_animating, active_specimen_id, mode_override) = {
        let shell = state.read();
        (
            shell.viewport_widget_id,
            shell.holo_scale_k,
            shell.holo_wireframe_animating,
            shell.active_holo_specimen_id.clone(),
            shell.holo_shader_mode,
        )
    };
    // Publish a committed Browser or Library route once. Studio choices can then
    // update the shared selection until the enclosing route changes again.
    use_effect(use_reactive((&specimen_id,), move |(selected,)| {
        if !selected.is_empty() && state.peek().active_holo_specimen_id != selected {
            let mut shell = state.write();
            shell.active_holo_specimen_id.clone_from(&selected);
            shell.holo_shader_mode = None;
        }
        applied_route.set(Some(selected));
    }));
    let active_specimen_id = if !specimen_id.is_empty()
        && applied_route.read().as_deref() != Some(specimen_id.as_str())
    {
        specimen_id
    } else if active_specimen_id.is_empty() {
        DEFAULT_HOLO_SPECIMENS[0].into()
    } else {
        active_specimen_id
    };
    use_effect(use_reactive((&active_specimen_id,), move |(query,)| {
        let result = session()
            .ok_or_else(|| String::from("Open an adopted local session to load specimen files."))
            .and_then(|adopted| resolve_holo_10d_specimen(&adopted.host.storage, &query))
            .map(Arc::new);
        resolved.set(Some((query, result)));
    }));
    let result = resolved
        .read()
        .as_ref()
        .filter(|(query, _)| query == &active_specimen_id)
        .map(|(_, result)| result.clone());
    let (specimen, specimen_status) = match result {
        Some(Ok(specimen)) => {
            let evidence = if specimen.container_profile == "qualiadb-mid2-10d-v2" {
                "MID2/HOLO integrity verified"
            } else {
                "Local Rehearsal · compact HOLO integrity verified"
            };
            let status = format!(
                "{} · {evidence} · {} segment records · {} wavefield records",
                specimen.title, specimen.holo_segment_count, specimen.wavefield_count
            );
            (Some(specimen), status)
        }
        Some(Err(error)) => (None, format!("Procedural projection · {error}")),
        None => (None, String::from("Resolving local HOLO specimen…")),
    };
    let specimen_integrity = specimen.as_ref().map(|specimen| {
        format!(
            "Subject {:016x} · NQuin48 parity {:016x} · SHA3-256 {}",
            specimen.nquin_subject_hash, specimen.sealed_nquin_parity, specimen.sha3_256_hex
        )
    });
    let mode = mode_override.unwrap_or_else(|| {
        specimen.as_ref().map_or(
            HoloShaderVisualizationMode::PhaseSpaceWavefield,
            |specimen| specimen.default_shader_mode,
        )
    });
    let entry = {
        let mut mounted = cache.borrow_mut();
        if mounted
            .as_ref()
            .is_none_or(|previous| previous.stable_id != id)
        {
            *mounted = Some(make_viewport(
                id,
                HoloViewportControl::new(initial_k, initial_animating),
                Arc::clone(&window),
            ));
        }
        mounted.as_ref().cloned()
    };
    if let Some(entry) = entry.as_ref() {
        let graph = state.read().project_semantic_scene_graph();
        let picked = {
            let mut control = controls(&entry.control);
            control.pick_graph = graph;
            control.last_picked.take()
        };
        if let Some(id) = picked {
            let mut shell = state.write();
            shell.last_picked_semantic_id = Some(id);
            if let Some(node) = shell
                .project_semantic_scene_graph()
                .nodes
                .iter()
                .find(|node| node.semantic_id == id)
            {
                let intent = node.primary_intent.clone();
                let _ = shell.apply_semantic_intent(&intent);
            }
        }
    }

    use_window_event({
        let cache = Rc::clone(&cache);
        move |event, _| {
            if !matches!(event, WindowEvent::RedrawRequested) {
                return;
            }
            let Some(entry) = cache.borrow().as_ref().cloned() else {
                return;
            };
            let (animating, submitted_extent, fallback_extent) = {
                let control = controls(&entry.control);
                (
                    control.animating,
                    control.submitted_extent,
                    control.fallback_extent,
                )
            };
            if state.peek().holo_wireframe_animating != animating {
                state.write().holo_wireframe_animating = animating;
            }
            let fallback = || {
                fallback_extent.map_or_else(
                    || String::from("Awaiting vector fallback scene"),
                    |(width, height)| format!("Vector fallback scene {width} × {height}"),
                )
            };
            let surface = match (entry.read_surface)() {
                EtalonViewportState::Suspended => String::from("Viewport suspended"),
                EtalonViewportState::Active => submitted_extent.map_or_else(
                    || String::from("Etalon prepared · awaiting first frame"),
                    |(width, height)| format!("Etalon frame submitted · {width} × {height}"),
                ),
                EtalonViewportState::CpuFallbackNotice(note) => format!("{note} · {}", fallback()),
                EtalonViewportState::Faulted(error) => {
                    format!("Etalon fault ({error:?}) · {}", fallback())
                }
            };
            let surface_changed =
                state.peek().renderer_health.active_holo_surface_status != surface;
            if surface_changed {
                state
                    .write()
                    .renderer_health
                    .active_holo_surface_status
                    .clone_from(&surface);
            }
            let next = if let Some(renderer) = &renderer {
                let health = renderer.health();
                let health_label = if health.faulted {
                    "fault reported"
                } else if health.gpu_active {
                    "GPU active"
                } else {
                    "GPU inactive"
                };
                let description = format!("{surface} · {} · {health_label}", health.backend_name);
                match health.last_error.as_deref() {
                    Some(error) => format!("{description} · {error}"),
                    None => description,
                }
            } else {
                surface
            };
            if *display.peek() != next {
                display.set(next);
            }
        }
    });

    use_drop(move || {
        if let Ok(mut shell) = state.try_write() {
            shell.renderer_health.active_holo_surface_status = "No HOLO surface observation".into();
        }
    });

    let Some(entry) = entry else {
        return VNode::empty();
    };
    let (harmonic_limit, step_limit) = {
        let mut control = controls(&entry.control);
        let previous = control
            .specimen
            .as_ref()
            .map(|specimen| &specimen.sha3_256_hex);
        let current = specimen.as_ref().map(|specimen| &specimen.sha3_256_hex);
        let changed =
            previous != current || control.mode != mode || control.harmonic_k != initial_k;
        if changed {
            control.specimen = specimen.clone();
            control.mode = mode;
            control.harmonic_k = initial_k;
        }
        let limits = (control.harmonic_limit(), control.step_limit());
        drop(control);
        if changed {
            window.request_redraw();
        }
        limits
    };
    let selected_harmonic = initial_k.min(harmonic_limit);
    let selected_budget = budget_display().min(step_limit);
    let attr = entry.attr.clone();
    let scale_control = Arc::clone(&entry.control);
    let scale_window = Arc::clone(&window);
    let step_control = Arc::clone(&entry.control);
    let step_window = Arc::clone(&window);
    let animate_control = Arc::clone(&entry.control);
    let animate_window = Arc::clone(&window);
    let reset_control = Arc::clone(&entry.control);
    let animate_label = if initial_animating {
        "Pause wavefield"
    } else {
        "Animate wavefield"
    };
    rsx! {
        section {
            class: "holo-viewport",
            h4 { "Holographic 10D v2 HOLO (0x0306)" }
            p { "{specimen_label} · {specimen_status}" }
            if let Some(integrity) = specimen_integrity {
                p { class: "muted", "{integrity}" }
            }
            div { class: "input-row",
                label {
                    "Local specimens"
                    select {
                        value: "{active_specimen_id}",
                        onchange: move |event| {
                            let selected = event.value();
                            if !DEFAULT_HOLO_SPECIMENS.contains(&selected.as_str())
                                && !UX_HOLO_SPECIMENS.contains(&selected.as_str())
                            {
                                return;
                            }
                            let mut shell = state.write();
                            shell.active_holo_specimen_id = selected;
                            shell.holo_shader_mode = None;
                        },
                        if !DEFAULT_HOLO_SPECIMENS.contains(&active_specimen_id.as_str())
                            && !UX_HOLO_SPECIMENS.contains(&active_specimen_id.as_str())
                        {
                            option { value: "{active_specimen_id}", "{active_specimen_id}" }
                        }
                        for id in DEFAULT_HOLO_SPECIMENS {
                            option { key: "{id}", value: "{id}", "{id}" }
                        }
                        for id in UX_HOLO_SPECIMENS {
                            option { key: "{id}", value: "{id}", "{id}" }
                        }
                    }
                }
            }
            div { class: "shader-mode-pills", role: "group", aria_label: "HOLO shader visualization",
                for choice in HoloShaderVisualizationMode::ALL {
                    button {
                        key: "{choice.code()}",
                        class: "ribbon-btn",
                        r#type: "button",
                        aria_pressed: if choice == mode { "true" } else { "false" },
                        onclick: move |_| state.write().holo_shader_mode = Some(choice),
                        "{choice.title()}"
                    }
                }
            }
            if let Some(scene) = specimen {
                HoloSceneInspector {
                    key: "{scene.sha3_256_hex}",
                    specimen: scene,
                }
            }
            object {
                class: "holo-viewport-object",
                tabindex: "0",
                "aria-label": "Holographic projection. Drag to orbit, scroll to zoom, Space to animate, R to reset.",
                data: attr,
            }
            p { class: "muted", "{display}" }
            div { class: "input-row",
                label {
                    "Harmonic order k={selected_harmonic}"
                    input {
                        r#type: "range", min: "0", max: "{harmonic_limit}", step: "1", value: "{selected_harmonic}",
                        oninput: move |evt| {
                            if let Ok(k) = evt.value().parse::<u8>() {
                                let k = k.min(harmonic_limit);
                                controls(&scale_control).harmonic_k = k;
                                state.write().holo_scale_k = k;
                                scale_window.request_redraw();
                            }
                        }
                    }
                }
                label {
                    "Raymarch budget {selected_budget}/{step_limit}"
                    input {
                        r#type: "range", min: "1", max: "{step_limit}", step: "1", value: "{selected_budget}",
                        oninput: move |evt| {
                            if let Ok(steps) = evt.value().parse::<u32>() {
                                let steps = steps.clamp(1, step_limit);
                                controls(&step_control).raymarch_steps = steps;
                                budget_display.set(steps);
                                step_window.request_redraw();
                            }
                        }
                    }
                }
                button {
                    r#type: "button", class: "ribbon-btn",
                    onclick: move |_| {
                        let animating = {
                            let mut control = controls(&animate_control);
                            control.animating = !control.animating;
                            control.animating
                        };
                        state.write().holo_wireframe_animating = animating;
                        animate_window.request_redraw();
                    },
                    "{animate_label}"
                }
                button {
                    r#type: "button", class: "ribbon-btn",
                    onclick: move |_| {
                        controls(&reset_control).reset_camera();
                        window.request_redraw();
                    },
                    "Reset Camera"
                }
            }
        }
    }
}

#[component]
fn HoloSceneInspector(specimen: Arc<VerifiedHolo10dSpecimen>) -> Element {
    use webizen_ua_os::MultiScaleStratumLayer;

    let mut filter = use_signal(|| None::<MultiScaleStratumLayer>);
    let selected = filter();
    let leaves = specimen
        .constituent_leaves
        .iter()
        .filter(|leaf| selected.is_none_or(|level| leaf.stratum == level));
    let no_matching_leaves = leaves.clone().next().is_none();
    rsx! {
        section {
            class: "holo-constituency",
            h5 { "Scene strata and constituent references" }
            p { class: "pillet", "{specimen.container_profile}" }
            div {
                class: "shader-mode-pills",
                role: "group",
                aria_label: "Filter constituent references by MSTR level",
                button {
                    class: "ribbon-btn",
                    r#type: "button",
                    aria_pressed: if selected.is_none() { "true" } else { "false" },
                    onclick: move |_| filter.set(None),
                    "All levels"
                }
                for level in MultiScaleStratumLayer::ALL {
                    button {
                        key: "{level.code()}",
                        class: "ribbon-btn",
                        r#type: "button",
                        title: "{level.title()}",
                        aria_pressed: if selected == Some(level) { "true" } else { "false" },
                        onclick: move |_| filter.set(Some(level)),
                        if specimen.active_strata.contains(&level) {
                            "{level.code()} · active"
                        } else {
                            "{level.code()}"
                        }
                    }
                }
            }
            if no_matching_leaves {
                p { "No AREF leaves in the selected levels." }
            } else {
                div {
                    style: "overflow-x: auto",
                    table {
                        aria_label: "AREF / IOVR / INST constituent leaves",
                        thead {
                            tr {
                                th { scope: "col", "AREF slot" }
                                th { scope: "col", "Child reference" }
                                th { scope: "col", "Stratum" }
                                th { scope: "col", "Translation (x, y, z)" }
                                th { scope: "col", "IOVR / INST" }
                            }
                        }
                        tbody {
                            for leaf in leaves {
                                tr {
                                    key: "{leaf.aref_slot}",
                                    td { "{leaf.aref_slot}" }
                                    td { style: "overflow-wrap: anywhere", "{leaf.child_uri}" }
                                    td { "{leaf.stratum.code()}" }
                                    td {
                                        "{leaf.motor_translation[0]:.3}, {leaf.motor_translation[1]:.3}, {leaf.motor_translation[2]:.3}"
                                    }
                                    td { "{leaf.iovr_override_summary}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn BrowserChoraSurface(state: Signal<ShellState>) -> Element {
    let shell = state.read();
    let mode = shell.browser_mode;
    let target = shell.browser_url.clone();
    let slot = shell.browser_product.worker_binary_slot.clone();
    drop(shell);
    if mode == BrowserExecutionMode::QualiaNative {
        let specimen_id = if resolve_mode_for_uri(&target, BrowserExecutionMode::TraditionalWeb)
            == BrowserExecutionMode::QualiaNative
        {
            target.clone()
        } else {
            String::new()
        };
        return rsx! { Holo10dViewport { state, specimen_label: target, specimen_id } };
    }
    #[cfg(all(feature = "chora", not(target_arch = "wasm32")))]
    {
        let worker_path = std::path::Path::new(&slot);
        if worker_path.is_absolute() && worker_path.is_file() {
            return rsx! { TrustedBrowserSurface { key: "{mode.code()}:{slot}", state, slot, mode } };
        }
    }
    let reason = if !cfg!(all(feature = "chora", not(target_arch = "wasm32"))) {
        "This build has no Ring C worker support."
    } else {
        "The Ring C worker binary slot is unavailable."
    };
    rsx! {
        WorkerSupervisorPanel { state, mode, worker_slot: slot.clone(), inactive_reason: reason,
            route_detail: String::new(), can_invalidate: false, oninvalidate: move |_| {} }
        BridgePreview { mode, slot }
    }
}

fn graph_snapshot_probe(
    mode: BrowserExecutionMode,
) -> Result<String, aether_qualia_host::chora_protocol::ReplyError> {
    use aether_qualia_host::{
        chora_protocol::{
            CallerContext, Control, ControlEnvelope, ReplyError, RouteStamp, MAX_REPLY_BYTES,
        },
        vibescript_abi::NQuin48,
        ChoraGraphRoutes,
    };
    const CHANNEL: &str = "qualia.graph.snapshot";
    let stamp = RouteStamp {
        session: [1; 16],
        view: 1,
        generation: 0,
        document: 0,
        viewport: 0,
    };
    let id = [2; 16];
    let request = ControlEnvelope {
        stamp,
        sequence: 1,
        message: Control::PageRequest {
            id,
            caller: CallerContext {
                engine_instance: [3; 16],
                webview: "native-preview".into(),
                pipeline: "native-preview".into(),
                navigation_epoch: [4; 16],
                origin: "https://preview.invalid".into(),
                private_browsing: false,
            },
            channel: CHANNEL.into(),
            payload: "{}".into(),
        },
    };
    // No page route is registered by this button. The real route checker must
    // deny this unsponsored request before a local fixture is shown natively.
    match ChoraGraphRoutes::new().accept(&request, Instant::now()) {
        Err(ReplyError::Denied) => {}
        _ => return Err(ReplyError::InvalidContext),
    }
    if mode != BrowserExecutionMode::HybridAugmented {
        return Err(ReplyError::Denied);
    }
    let example_context = "urn:webizen:example:public-route";
    let row = NQuin48::invariant(
        example_context,
        "urn:webizen:example:route-status",
        webizen_ua_os::q_hash_60("local native fixture"),
    );
    let body = serde_json::to_string_pretty(&serde_json::json!({
        "context": webizen_ua_os::q_hash_60(example_context).to_string(),
        "revision": "0",
        "quins": [row.0.map(|word| word.to_string())],
    }))
    .map_err(|_| ReplyError::Unavailable)?;
    if body.len() > MAX_REPLY_BYTES {
        return Err(ReplyError::TooLarge);
    }
    let reply = ControlEnvelope {
        stamp,
        sequence: 2,
        message: Control::PageReply {
            id,
            result: Ok(body.clone()),
        },
    };
    reply.validate().map_err(|_| ReplyError::TooLarge)?;
    Ok(body)
}

#[component]
fn WorkerSupervisorPanel(
    state: Signal<ShellState>,
    mode: BrowserExecutionMode,
    worker_slot: String,
    inactive_reason: &'static str,
    route_detail: String,
    can_invalidate: bool,
    oninvalidate: EventHandler<()>,
) -> Element {
    let mut graph_test = use_signal(|| {
        None::<(
            BrowserExecutionMode,
            Result<String, aether_qualia_host::chora_protocol::ReplyError>,
        )>
    });
    let preview = graph_test
        .read()
        .as_ref()
        .filter(|(tested_mode, _)| mode == *tested_mode)
        .map(|(_, result)| result.clone());
    let snapshot = state.read().chora_supervisor.clone();
    let slot = if worker_slot.is_empty() {
        "Unconfigured"
    } else {
        worker_slot.as_str()
    };
    let worker_state = if inactive_reason.is_empty() {
        snapshot.worker_state_label.as_str()
    } else {
        inactive_reason
    };
    let frame = if snapshot.frame_seq == 0 {
        String::from("Awaiting first frame")
    } else {
        snapshot.frame_seq.to_string()
    };
    rsx! {
        section {
            class: "browser-viewport-launcher",
            h4 { "Web Engine worker (`chora-worker`) is not running — click to probe or launch" }
            p { class: "muted", "{worker_state}" }
            details {
                class: "tech-details-drawer",
                summary { "Engine details" }
                dl {
                    class: "kv",
                    dt { "Worker binary" }
                    dd { class: "path", "{slot}" }
                    dt { "Worker state" }
                dd { "{worker_state}" }
                if snapshot.view_id != 0 && inactive_reason.is_empty() {
                    dt { "Session tag · view · accepted frame sequence" }
                    dd { "{snapshot.session_id:016x} · {snapshot.view_id} · {frame}" }
                    dt { "Route generation · document · viewport" }
                    dd { "{route_detail}" }
                    dt { "Granted graph routes · pending page requests" }
                    dd { "{snapshot.registered_routes} · {snapshot.pending_page_requests}" }
                    dt { "Inputs written to worker pipe" }
                    dd {
                        "Pointer {snapshot.forwarded_pointer_events} · Wheel {snapshot.forwarded_wheel_events} · Key {snapshot.forwarded_key_events} · IME {snapshot.forwarded_ime_events}"
                    }
                    dt { "Forwarded focus" }
                    dd { if snapshot.focus_active { "Active" } else { "Inactive" } }
                }
                if !snapshot.last_route_invalidation_reason.is_empty() {
                    dt { "Last invalidation" }
                    dd { "{snapshot.last_route_invalidation_reason}" }
                }
            }
            div {
                class: "input-row",
                button {
                    r#type: "button", class: "ribbon-btn",
                    onclick: move |_| graph_test.set(Some((mode, graph_snapshot_probe(mode)))),
                    "Test ChoraGraphRoutes Snapshot"
                }
                button {
                    r#type: "button", class: "ribbon-btn",
                    disabled: !can_invalidate,
                    onclick: move |_| oninvalidate.call(()),
                    "Invalidate Active RouteStamp"
                }
            }
            p { class: "muted",
                "Manual invalidation reloads the current page and clears its pending requests. The worker returns the protocol invalidation acknowledgement."
            }
            match preview {
                Some(Ok(body)) => rsx! {
                    p { role: "status", "Local Preview · qualia.graph.snapshot · the synthetic caller has no graph grant." }
                    pre { "{body}" }
                },
                Some(Err(error)) => rsx! { p { role: "status", "qualia.graph.snapshot · {error:?}" } },
                None => rsx! {},
            }
            }
        }
    }
}

#[component]
fn BridgePreview(mode: BrowserExecutionMode, slot: String) -> Element {
    let mut result = use_signal(|| None::<String>);
    let cases = HybridWebCapabilityChannel::ALL;
    let selected_mode = mode.title();
    let worker_slot = if slot.is_empty() {
        String::from("Worker slot is not configured")
    } else {
        slot
    };
    let last = result.read().clone();
    rsx! {
        section {
            class: "browser-worker-contract",
            h4 { "Sibling Worker and Copied-Frame Contract" }
            p { "{selected_mode} · {worker_slot}" }
            p { "These tests use a synthetic origin and payload. The VibeScript test evaluates one isolated scalar expression." }
            div { class: "input-row",
                for channel in cases {
                    {
                        let label = channel.spec().channel_id;
                        rsx! {
                            button {
                                key: "{label}", r#type: "button", class: "ribbon-btn",
                                onclick: move |_| {
                                    const TEST_ORIGIN: &str = "https://preview.invalid";
                                    let response = if let Some(adopted) = session() {
                                        let tested = evaluate_hybrid_page_request(
                                            mode, &adopted.identifiers, TEST_ORIGIN, channel, channel.local_test_payload(),
                                        );
                                        format!("{label}: {} · NQuin48 parity {:016x}", tested.reply_payload, tested.sealed_nquin_parity)
                                    } else {
                                        format!("{label}: launch the adopted host session to test the preview policy")
                                    };
                                    result.set(Some(response));
                                },
                                "Test {label}"
                            }
                        }
                    }
                }
            }
            if let Some(message) = last {
                p { role: "status", "{message}" }
            }
        }
    }
}

#[cfg(all(feature = "chora", not(target_arch = "wasm32")))]
mod traditional {
    use super::*;
    use aether::aether_core::try_consume_context;
    use aether_native::{
        aether_chora_client::{
            protocol::{
                CallerContext, Control, ControlEnvelope, ExecutionProfile, Nonce, ProtocolError,
                ReplyError, RouteStamp, MAX_URL_BYTES,
            },
            ChoraClient,
        },
        chora_frame::ChoraFrameWidget,
    };
    use aether_qualia_host::{ChoraGraphRoutes, NativeGraphBridge};
    use std::{
        path::Path,
        sync::{
            atomic::{AtomicBool, AtomicUsize, Ordering},
            mpsc,
        },
    };
    use webizen_ua_os::{
        authorize_qap_graph_read, require_qap_capability, AdoptedQualiaFoundation,
        CapabilityLeaseScope, MAX_GOVERNED_VIBESCRIPT_CELL_BYTES,
    };

    const GRAPH_SNAPSHOT_CHANNEL: &str = "qualia.graph.snapshot";

    static NEXT_WORKER_WIDGET: AtomicUsize = AtomicUsize::new(1);

    struct WorkerSession {
        client: ChoraClient,
        sent_address: Option<String>,
        observed_stamp: Option<RouteStamp>,
        manual_reload_generation: Option<u64>,
        last_published: Option<Instant>,
        graph_read: Option<GraphReadTask>,
    }

    fn refresh_worker_supervisor(
        mut state: Signal<ShellState>,
        worker: &mut WorkerSession,
        status: &str,
        pending: bool,
    ) -> Option<RouteStamp> {
        let now = Instant::now();
        if worker
            .last_published
            .is_some_and(|at| now.duration_since(at) < Duration::from_millis(150))
        {
            return None;
        }
        let transport = worker.client.route_telemetry().ok()?;
        let stamp = transport.stamp;
        let mut snapshot = state.peek().chora_supervisor.clone();
        if let Some(previous) = worker.observed_stamp {
            if previous != stamp {
                let reason = if previous.generation != stamp.generation {
                    "Navigation boundary"
                } else if previous.viewport != stamp.viewport {
                    "Viewport resize boundary"
                } else {
                    "Worker document boundary"
                };
                snapshot.invalidate_route(stamp.view, reason);
            }
        }
        snapshot.session_id = u64::from_be_bytes(
            stamp.session[..8]
                .try_into()
                .expect("RouteStamp session prefix"),
        );
        snapshot.view_id = stamp.view;
        snapshot.frame_seq = transport.last_frame_sequence.unwrap_or(0);
        snapshot.registered_routes = usize::from(
            worker
                .graph_read
                .as_ref()
                .is_some_and(|read| read.registered.load(Ordering::Acquire)),
        );
        snapshot.pending_page_requests = transport
            .queued_page_requests
            .saturating_add(usize::from(pending))
            .saturating_add(usize::from(worker.graph_read.is_some()));
        snapshot.forwarded_pointer_events = transport.forwarded_pointer_events;
        snapshot.forwarded_wheel_events = transport.forwarded_wheel_events;
        snapshot.forwarded_key_events = transport.forwarded_key_events;
        snapshot.forwarded_ime_events = transport.forwarded_ime_events;
        snapshot.focus_active = transport.focus_active;
        snapshot.worker_state_label = status.to_owned();
        worker.observed_stamp = Some(stamp);
        worker.last_published = Some(now);
        let changed = state.peek().chora_supervisor != snapshot;
        if changed {
            state.write().chora_supervisor = snapshot;
        }
        Some(stamp)
    }

    #[derive(Clone)]
    struct PendingScript {
        stamp: RouteStamp,
        id: Nonce,
        origin: String,
        payload: String,
        expires_at: Instant,
        graph_caller: Option<CallerContext>,
        graph_authority: Option<AdoptedQualiaFoundation>,
        sequence: u64,
    }

    struct GraphReply {
        snapshot: String,
        authority_generation: u64,
    }

    struct GraphReadTask {
        request: PendingScript,
        receiver: mpsc::Receiver<Result<GraphReply, ReplyError>>,
        registered: Arc<AtomicBool>,
        result: Option<Result<GraphReply, ReplyError>>,
    }

    fn configured_graph_origin() -> Option<String> {
        let configured = std::env::var("WEBIZEN_QUALIA_GRAPH_ORIGIN").ok()?;
        let url = url::Url::parse(&configured).ok()?;
        (url.scheme() == "https" && url.origin().ascii_serialization() == configured)
            .then_some(configured)
    }

    fn same_graph_authority(expected: &AdoptedQualiaFoundation) -> bool {
        let Some(grant) = expected.graph_grant() else {
            return false;
        };
        let Ok(elapsed) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) else {
            return false;
        };
        let now = elapsed.as_secs();
        grant.grant.valid_from_unix <= now
            && now < grant.grant.valid_until_unix
            && try_consume_context::<AdoptedQualiaFoundation>().is_some_and(|current| {
                current.session() == expected.session() && current.graph_grant() == Some(grant)
            })
    }

    fn start_graph_read(
        worker: &mut WorkerSession,
        request: PendingScript,
    ) -> Result<(), ReplyError> {
        if worker.graph_read.is_some() {
            return Err(ReplyError::Busy);
        }
        let caller = request
            .graph_caller
            .clone()
            .ok_or(ReplyError::InvalidContext)?;
        let foundation = request
            .graph_authority
            .as_ref()
            .filter(|host| same_graph_authority(host))
            .cloned()
            .ok_or(ReplyError::Denied)?;
        let storage = session().ok_or(ReplyError::Denied)?.host.storage.clone();
        let grant = foundation
            .graph_grant()
            .cloned()
            .ok_or(ReplyError::Denied)?;
        let session = foundation.session();
        let authority = foundation.authority();
        let envelope = ControlEnvelope {
            stamp: request.stamp,
            sequence: request.sequence,
            message: Control::PageRequest {
                id: request.id,
                caller: caller.clone(),
                channel: GRAPH_SNAPSHOT_CHANNEL.to_owned(),
                payload: request.payload.clone(),
            },
        };
        let id = request.id;
        let expires = request.expires_at;
        let (sender, receiver) = mpsc::sync_channel(1);
        let registered = Arc::new(AtomicBool::new(false));
        let owner_registered = Arc::clone(&registered);
        std::thread::Builder::new()
            .name("webizen-graph-read".into())
            .spawn(move || {
                let result = (|| {
                    if Instant::now() >= expires {
                        return Err(ReplyError::TimedOut);
                    }
                    authorize_qap_graph_read(&storage, "Browser", &foundation)
                        .map_err(|_| ReplyError::Denied)?;
                    let authority_generation = authority
                        .workflow_evidence(&session)
                        .map_err(|_| ReplyError::Denied)?
                        .authority_generation;
                    let bridge = NativeGraphBridge::connect(authority, &session, &grant)
                        .map_err(|_| ReplyError::Denied)?;
                    let mut routes = ChoraGraphRoutes::new();
                    routes.register(
                        envelope.stamp,
                        caller,
                        ExecutionProfile::TrustedEmbedding,
                        bridge,
                    )?;
                    owner_registered.store(true, Ordering::Release);
                    routes.accept(&envelope, Instant::now())?;
                    routes
                        .finish(envelope.stamp, id, Instant::now())
                        .map(|snapshot| GraphReply {
                            snapshot,
                            authority_generation,
                        })
                })();
                owner_registered.store(false, Ordering::Release);
                let _ = sender.try_send(result);
            })
            .map_err(|_| ReplyError::Unavailable)?;
        worker.graph_read = Some(GraphReadTask {
            request,
            receiver,
            registered,
            result: None,
        });
        Ok(())
    }

    fn poll_graph_read(
        worker: &mut WorkerSession,
        mode: BrowserExecutionMode,
        address: &str,
    ) -> Option<String> {
        let active = worker.graph_read.as_ref()?;
        match check_script_route(worker, &active.request, mode, address) {
            Err(ReplyError::Busy) => return None,
            Err(_) => {
                let _ = worker.client.reply(
                    active.request.stamp,
                    active.request.id,
                    Err(ReplyError::Cancelled),
                );
                worker.graph_read = None;
                return Some("The graph request expired or its route changed.".into());
            }
            Ok(()) => {}
        }
        let same_authority = active
            .request
            .graph_authority
            .as_ref()
            .is_some_and(same_graph_authority);
        let active = worker.graph_read.as_mut()?;
        if !same_authority {
            active.result = Some(Err(ReplyError::Denied));
        } else if active.result.is_none() {
            active.result = match active.receiver.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Empty) => return None,
                Err(mpsc::TryRecvError::Disconnected) => Some(Err(ReplyError::Unavailable)),
            };
        }
        let stamp = active.request.stamp;
        let id = active.request.id;
        if let Some(Ok(reply)) = active.result.as_ref() {
            let current = active.request.graph_authority.as_ref().is_some_and(|host| {
                host.authority()
                    .publication_is_current(&host.session(), reply.authority_generation)
            });
            let lease_allowed = session().is_some_and(|session| {
                require_qap_capability(
                    &session.host.storage,
                    "Browser",
                    CapabilityLeaseScope::QualiaQ42Storage,
                )
                .is_ok()
            });
            if !current || !lease_allowed {
                active.result = Some(Err(ReplyError::Denied));
            }
        }
        let result = match active.result.as_ref()? {
            Ok(reply) => Ok(reply.snapshot.clone()),
            Err(error) => Err(error.clone()),
        };
        let status = match &result {
            Ok(_) => "Live Verified · canonical graph snapshot queued for the approved origin."
                .to_owned(),
            Err(error) => format!("Graph snapshot request refused: {error:?}"),
        };
        match worker.client.reply(stamp, id, result) {
            Ok(()) => {
                worker.graph_read = None;
                Some(status)
            }
            Err(ProtocolError::Busy) => None,
            Err(ProtocolError::TooLarge) => {
                worker.graph_read.as_mut()?.result = Some(Err(ReplyError::TooLarge));
                None
            }
            Err(error) => {
                worker.graph_read = None;
                Some(format!("Graph reply could not reach the page: {error}"))
            }
        }
    }

    fn admitted_https_origin(address: &str, origin: &str) -> bool {
        url::Url::parse(address)
            .ok()
            .filter(|url| url.scheme() == "https")
            .is_some_and(|url| url.origin().ascii_serialization() == origin)
    }

    fn check_script_route(
        worker: &WorkerSession,
        script: &PendingScript,
        mode: BrowserExecutionMode,
        address: &str,
    ) -> Result<(), ReplyError> {
        let address = traditional_address(address).map_err(|_| ReplyError::Cancelled)?;
        if mode != BrowserExecutionMode::HybridAugmented
            || Instant::now() >= script.expires_at
            || worker.sent_address.as_deref() != Some(address.as_str())
            || !admitted_https_origin(&address, &script.origin)
        {
            return Err(ReplyError::Cancelled);
        }
        match worker.client.stamp() {
            Ok(stamp) if stamp == script.stamp => Ok(()),
            Err(ProtocolError::Busy) => Err(ReplyError::Busy),
            _ => Err(ReplyError::Cancelled),
        }
    }

    fn traditional_address(value: &str) -> Result<String, &'static str> {
        let value = value.trim();
        if value == "about:blank" {
            return Ok(value.to_owned());
        }
        if value.len() > MAX_URL_BYTES {
            return Err("The address exceeds the worker limit.");
        }
        let parsed = url::Url::parse(value).map_err(|_| "Enter a valid HTTPS or HTTP address.")?;
        if !matches!(parsed.scheme(), "https" | "http")
            || !parsed.has_host()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
        {
            return Err("Enter an HTTPS or HTTP address without embedded credentials.");
        }
        if parsed.as_str().len() > MAX_URL_BYTES {
            return Err("The encoded address exceeds the worker limit.");
        }
        Ok(parsed.to_string())
    }

    #[component]
    pub(super) fn TrustedBrowserSurface(
        mut state: Signal<ShellState>,
        slot: String,
        mode: BrowserExecutionMode,
    ) -> Element {
        let window = use_window();
        let running = use_hook(|| Rc::new(RefCell::new(None::<WorkerSession>)));
        let mut widget_attr = use_signal(|| None::<CustomWidgetAttr>);
        let mut status = use_signal(|| String::from("Worker is stopped"));
        let mut pending_script = use_signal(|| None::<PendingScript>);
        let mut script_notice = use_signal(|| None::<String>);
        let mut route_detail = use_signal(String::new);
        let allowed_graph_origin = use_hook(configured_graph_origin);

        use_drop({
            let running = Rc::clone(&running);
            move || {
                if let Some(owned) = running.borrow_mut().take() {
                    owned.client.shutdown();
                    state
                        .write()
                        .chora_supervisor
                        .clear_worker("Browser route left or worker selection changed");
                }
            }
        });
        use_window_event({
            let running = Rc::clone(&running);
            let allowed_graph_origin = allowed_graph_origin.clone();
            move |event, _| {
                if !matches!(event, WindowEvent::RedrawRequested) {
                    return;
                }
                let mut guard = running.borrow_mut();
                let Some(owned) = guard.as_mut() else {
                    return;
                };
                let (shell_mode, address) = {
                    let shell = state.peek();
                    (shell.browser_mode, shell.browser_url.clone())
                };
                let stale = pending_script.peek().as_ref().is_some_and(|request| {
                    check_script_route(owned, request, shell_mode, &address)
                        .is_err_and(|error| error != ReplyError::Busy)
                });
                if stale {
                    if let Some(request) = pending_script.take() {
                        let _ = owned.client.reply(
                            request.stamp,
                            request.id,
                            Err(ReplyError::Cancelled),
                        );
                    }
                }
                let mut terminal_status = None;
                for _ in 0..8 {
                    let Some(envelope) = owned.client.take_event() else {
                        break;
                    };
                    match envelope.message {
                        Control::PageRequest {
                            id,
                            caller,
                            channel,
                            payload,
                        } => {
                            let graph = channel == GRAPH_SNAPSHOT_CHANNEL;
                            let graph_authority = if graph
                                && !state.peek().blocks_live_qap_lease(
                                    "Browser",
                                    CapabilityLeaseScope::QualiaQ42Storage,
                                )
                                && !caller.private_browsing
                                && payload == "{}"
                                && allowed_graph_origin.as_deref() == Some(caller.origin.as_str())
                            {
                                try_consume_context::<AdoptedQualiaFoundation>().filter(|host| {
                                    host.graph_grant()
                                        .is_some_and(|grant| grant.grant.rights.read)
                                })
                            } else {
                                None
                            };
                            let scalar_allowed = channel
                                == HybridWebCapabilityChannel::GovernedVibeScriptEval
                                    .spec()
                                    .channel_id
                                && !state.peek().blocks_live_qap_lease(
                                    "Browser",
                                    CapabilityLeaseScope::GovernedVibeScriptEval,
                                )
                                && !caller.private_browsing
                                && !payload.is_empty()
                                && payload.len() <= MAX_GOVERNED_VIBESCRIPT_CELL_BYTES
                                && payload.bytes().all(|byte| (b' '..=b'~').contains(&byte))
                                && session().is_some();
                            if graph_authority.is_none() && !scalar_allowed {
                                let _ =
                                    owned
                                        .client
                                        .reply(envelope.stamp, id, Err(ReplyError::Denied));
                                continue;
                            }
                            let request = PendingScript {
                                stamp: envelope.stamp,
                                id,
                                origin: caller.origin.clone(),
                                payload,
                                expires_at: Instant::now() + Duration::from_secs(25),
                                graph_caller: graph.then_some(caller),
                                graph_authority,
                                sequence: envelope.sequence,
                            };
                            if check_script_route(owned, &request, shell_mode, &address)
                                .is_err_and(|error| error != ReplyError::Busy)
                            {
                                let _ = owned.client.reply(
                                    request.stamp,
                                    request.id,
                                    Err(ReplyError::Denied),
                                );
                            } else if pending_script.peek().is_some() || owned.graph_read.is_some()
                            {
                                let _ = owned.client.reply(
                                    request.stamp,
                                    request.id,
                                    Err(ReplyError::Busy),
                                );
                            } else {
                                script_notice.set(None);
                                pending_script.set(Some(request));
                            }
                        }
                        Control::Cancel { id, .. } => {
                            let cancelled = pending_script.peek().as_ref().is_some_and(|request| {
                                request.id == id && request.stamp == envelope.stamp
                            });
                            if cancelled {
                                pending_script.set(None);
                            }
                            if owned.graph_read.as_ref().is_some_and(|active| {
                                active.request.id == id && active.request.stamp == envelope.stamp
                            }) {
                                owned.graph_read = None;
                            }
                        }
                        Control::Invalidate { .. } => {
                            let previous = owned.observed_stamp;
                            let reason = if owned.manual_reload_generation
                                == Some(envelope.stamp.generation)
                            {
                                owned.manual_reload_generation = None;
                                "Manual route invalidation / worker acknowledgement"
                            } else if previous
                                .is_some_and(|stamp| stamp.viewport != envelope.stamp.viewport)
                            {
                                "Worker acknowledged viewport resize"
                            } else if previous
                                .is_some_and(|stamp| stamp.generation != envelope.stamp.generation)
                            {
                                "Worker acknowledged navigation"
                            } else {
                                "Worker acknowledged route invalidation"
                            };
                            state
                                .write()
                                .chora_supervisor
                                .invalidate_route(envelope.stamp.view, reason);
                            owned.observed_stamp = Some(envelope.stamp);
                            owned.last_published = None;
                            owned.graph_read = None;
                            pending_script.set(None);
                            script_notice.set(None);
                        }
                        Control::Failed { reason } => {
                            terminal_status = Some(format!("Worker stopped: {reason:?}"));
                            break;
                        }
                        Control::Stopped => {
                            terminal_status = Some(String::from("Worker stopped"));
                            break;
                        }
                        _ => {}
                    }
                }
                if owned.client.has_exited()
                    || matches!(owned.client.stamp(), Err(ProtocolError::Closed))
                {
                    terminal_status.get_or_insert_with(|| String::from("Worker connection closed"));
                }
                if let Some(message) = terminal_status {
                    if let Some(dead) = guard.take() {
                        dead.client.shutdown();
                    }
                    drop(guard);
                    widget_attr.set(None);
                    pending_script.set(None);
                    state.write().chora_supervisor.clear_worker(&message);
                    route_detail.set(String::new());
                    status.set(message);
                    return;
                }
                if owned.client.is_ready() {
                    let next = match traditional_address(&address) {
                        Err(message) => Some(message.to_owned()),
                        Ok(address) if owned.sent_address.as_deref() == Some(address.as_str()) => {
                            Some(String::from("Browser request sent to copied-frame worker"))
                        }
                        Ok(address) => match owned.client.navigate(address.clone()) {
                            Ok(_) => {
                                owned.sent_address = Some(address);
                                owned.manual_reload_generation = None;
                                owned.graph_read = None;
                                pending_script.set(None);
                                script_notice.set(None);
                                Some(String::from("Browser request sent to copied-frame worker"))
                            }
                            Err(ProtocolError::Busy) => None,
                            Err(error) => Some(format!("Worker navigation failed: {error}")),
                        },
                    };
                    if let Some(next) = next {
                        if *status.peek() != next {
                            status.set(next);
                        }
                    }
                }
                if let Some(notice) = poll_graph_read(owned, shell_mode, &address) {
                    script_notice.set(Some(notice));
                }
                if let Some(stamp) = refresh_worker_supervisor(
                    state,
                    owned,
                    &status.peek(),
                    pending_script.peek().is_some(),
                ) {
                    let detail = format!(
                        "{} · {} · {}",
                        stamp.generation, stamp.document, stamp.viewport
                    );
                    if *route_detail.peek() != detail {
                        route_detail.set(detail);
                    }
                }
            }
        });

        let on_start = Rc::clone(&running);
        let on_stop = Rc::clone(&running);
        let approve_worker = Rc::clone(&running);
        let deny_worker = Rc::clone(&running);
        let invalidate_worker = Rc::clone(&running);
        let invalidate_window = Arc::clone(&window);
        let requested_script = pending_script.read().clone();
        let start_window = Arc::clone(&window);
        let start_path = slot.clone();
        let object_attr = widget_attr.read().clone();
        let can_invalidate = state.read().chora_supervisor.view_id != 0 && object_attr.is_some();
        let route_detail_text = route_detail.read().clone();
        rsx! {
            section {
                class: "browser-worker-surface",
                p { role: "status", "{status}" }
                WorkerSupervisorPanel {
                    state, mode, worker_slot: slot.clone(), inactive_reason: "", can_invalidate,
                    route_detail: route_detail_text,
                    oninvalidate: move |_| {
                        let mut guard = invalidate_worker.borrow_mut();
                        let Some(worker) = guard.as_mut() else { return; };
                        let Some(address) = worker.sent_address.clone() else {
                            script_notice.set(Some("The worker has no current page to invalidate.".into()));
                            return;
                        };
                        match worker.client.navigate(address) {
                            Ok(stamp) => {
                                pending_script.set(None);
                                script_notice.set(None);
                                state.write().chora_supervisor.invalidate_route(stamp.view, "Manual route invalidation / navigation boundary");
                                worker.observed_stamp = Some(stamp);
                                worker.manual_reload_generation = Some(stamp.generation);
                                worker.last_published = None;
                                worker.graph_read = None;
                                invalidate_window.request_redraw();
                            }
                            Err(error) => script_notice.set(Some(format!("Could not invalidate the worker route: {error}"))),
                        }
                    },
                }
                if let Some(attr) = object_attr {
                    if let Some(request) = requested_script {
                        {
                            let origin = request.origin.clone();
                            let payload = request.payload.clone();
                            let expected = request.id;
                            let stamp = request.stamp;
                            let graph_request = request.graph_caller.is_some();
                            let graph_context = request.graph_authority.as_ref()
                                .and_then(|host| host.graph_grant())
                                .map(|grant| grant.grant.context)
                                .unwrap_or_default();
                            let approval_label = if graph_request { "Share Snapshot Once" } else { "Evaluate Once" };
                            rsx! {
                                section {
                                    class: "id-list",
                                    if graph_request {
                                        h5 { "Qualia graph snapshot request from {origin}" }
                                        p { "Share one read of signed project graph context {graph_context} with this origin." }
                                    } else {
                                        h5 { "VibeScript page request from {origin}" }
                                        p { "Allow one isolated scalar evaluation for this page." }
                                        pre { "{payload}" }
                                    }
                                    button {
                                        r#type: "button", class: "ribbon-btn",
                                        onclick: move |_| {
                                            let selected = pending_script.peek().as_ref().filter(|request| {
                                                request.id == expected && request.stamp == stamp
                                            }).cloned();
                                            let Some(request) = selected else { return; };
                                            let mut guard = approve_worker.borrow_mut();
                                            let Some(worker) = guard.as_mut() else { return; };
                                            let shell = state.peek();
                                            match check_script_route(worker, &request, shell.browser_mode, &shell.browser_url) {
                                                Ok(()) => pending_script.set(None),
                                                Err(ReplyError::Busy) => {
                                                    script_notice.set(Some(String::from("The worker is busy. Retry this expression.")));
                                                    return;
                                                }
                                                Err(error) => {
                                                    pending_script.set(None);
                                                    let _ = worker.client.reply(request.stamp, request.id, Err(error));
                                                    script_notice.set(Some(String::from("The original page request has expired.")));
                                                    return;
                                                }
                                            }
                                            let scope = if request.graph_caller.is_some() {
                                                CapabilityLeaseScope::QualiaQ42Storage
                                            } else {
                                                CapabilityLeaseScope::GovernedVibeScriptEval
                                            };
                                            if shell.blocks_live_qap_lease("Browser", scope) {
                                                let _ = worker.client.reply(request.stamp, request.id, Err(ReplyError::Denied));
                                                script_notice.set(Some("The Browser capability lease blocks this request.".into()));
                                                return;
                                            }
                                            if request.graph_caller.is_some() {
                                                let started = start_graph_read(worker, request.clone());
                                                let note = match started {
                                                    Ok(()) => "Reading the signed graph scope…".to_owned(),
                                                    Err(error) => {
                                                        let _ = worker.client.reply(request.stamp, request.id, Err(error));
                                                        format!("Graph snapshot request refused: {error:?}")
                                                    },
                                                };
                                                script_notice.set(Some(note));
                                                return;
                                            }
                                            let Some(adopted) = session() else {
                                                let _ = worker.client.reply(request.stamp, request.id, Err(ReplyError::Unavailable));
                                                return;
                                            };
                                            let receipt = evaluate_hybrid_page_request(
                                                shell.browser_mode,
                                                &adopted.identifiers,
                                                &request.origin,
                                                HybridWebCapabilityChannel::GovernedVibeScriptEval,
                                                &request.payload,
                                            );
                                            let notice = if receipt.permitted {
                                                let delivered = serde_json::to_string(&receipt).is_ok_and(|reply| {
                                                    worker.client.reply(request.stamp, request.id, Ok(reply)).is_ok()
                                                });
                                                if delivered {
                                                    format!("VibeScript result sent · NQuin48 parity {:016x}", receipt.sealed_nquin_parity)
                                                } else {
                                                    String::from("The page could not receive the VibeScript result.")
                                                }
                                            } else {
                                                let _ = worker.client.reply(request.stamp, request.id, Err(ReplyError::Denied));
                                                receipt.reply_payload
                                            };
                                            script_notice.set(Some(notice));
                                        },
                                        "{approval_label}"
                                    }
                                    button {
                                        r#type: "button", class: "ribbon-btn",
                                        onclick: move |_| {
                                            let matches = pending_script.peek().as_ref().is_some_and(|request| {
                                                request.id == expected && request.stamp == stamp
                                            });
                                            if matches {
                                                if let (Some(worker), Some(request)) = (deny_worker.borrow().as_ref(), pending_script.take()) {
                                                    let _ = worker.client.reply(request.stamp, request.id, Err(ReplyError::Denied));
                                                }
                                            }
                                        },
                                        "Decline"
                                    }
                                }
                            }
                        }
                    }
                    if let Some(notice) = script_notice.read().as_ref() {
                        p { role: "status", "{notice}" }
                    }
                    object {
                        class: "chora-frame-object",
                        style: "display: block; width: 100%; height: 350px;",
                        tabindex: "0",
                        "aria-label": "Interactive traditional web view",
                        data: attr,
                    }
                    button {
                        r#type: "button", class: "ribbon-btn",
                        onclick: move |_| {
                            if let Some(owned) = on_stop.borrow_mut().take() { owned.client.shutdown(); }
                            widget_attr.set(None);
                            pending_script.set(None);
                            script_notice.set(None);
                            state.write().chora_supervisor.clear_worker("Stopped by the shell");
                            route_detail.set(String::new());
                            status.set(String::from("Worker stopped"));
                        },
                        "Stop Worker"
                    }
                } else {
                    p { "Start the installed worker only for pages you choose to trust. It uses a separate process and has no OS containment." }
                    button {
                        r#type: "button", class: "ribbon-btn",
                        onclick: move |_| {
                            if on_start.borrow().is_some() { return; }
                            if let Err(message) = traditional_address(&state.peek().browser_url) {
                                status.set(message.to_owned());
                                return;
                            }
                            match ChoraClient::launch(Path::new(&start_path), ExecutionProfile::TrustedEmbedding, false) {
                                Ok(client) => {
                                    let id = NEXT_WORKER_WIDGET.fetch_add(1, Ordering::Relaxed);
                                    widget_attr.set(Some(CustomWidgetAttr::with_stable_id(id, ChoraFrameWidget::new(client.clone(), peniko::ImageAlphaType::Alpha))));
                                    *on_start.borrow_mut() = Some(WorkerSession { client, sent_address: None, observed_stamp: None, manual_reload_generation: None, last_published: None, graph_read: None });
                                    let mut shell = state.write();
                                    shell.chora_supervisor.clear_worker("New worker started");
                                    shell.chora_supervisor.worker_binary_path = start_path.clone();
                                    shell.chora_supervisor.worker_state_label = "Starting copied-frame worker".into();
                                    drop(shell);
                                    status.set(String::from("Starting copied-frame worker"));
                                    start_window.request_redraw();
                                }
                                Err(error) => status.set(format!("Worker launch failed: {error}")),
                            }
                        },
                        "Start Trusted Worker"
                    }
                    BridgePreview { mode, slot: slot.clone() }
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn script_approval_matches_an_exact_https_origin() {
            assert!(admitted_https_origin(
                "https://example.org/a",
                "https://example.org"
            ));
            assert!(admitted_https_origin(
                "https://example.org:8443/a",
                "https://example.org:8443"
            ));
            for (address, origin) in [
                ("http://example.org/a", "http://example.org"),
                ("https://example.org:8443/a", "https://example.org"),
                ("https://example.org/a", "https://example.org.evil"),
                ("https://example.org/a", "null"),
            ] {
                assert!(!admitted_https_origin(address, origin));
            }
        }

        #[test]
        fn worker_address_accepts_only_bounded_legacy_routes() {
            assert_eq!(
                traditional_address(" https://example.org ").unwrap(),
                "https://example.org/"
            );
            assert_eq!(traditional_address("about:blank").unwrap(), "about:blank");
            for address in [
                "qualia://chora/universe",
                "file:///tmp/file",
                "https://user:secret@example.org",
                "http://#",
                "about:config",
            ] {
                assert!(traditional_address(address).is_err());
            }
            assert!(traditional_address(&format!(
                "https://example.org/{}",
                "a".repeat(MAX_URL_BYTES)
            ))
            .is_err());
        }
    }
}

#[cfg(all(feature = "chora", not(target_arch = "wasm32")))]
use traditional::TrustedBrowserSurface;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_snapshot_probe_separates_local_fixture_from_worker_grants() {
        use aether_qualia_host::chora_protocol::{ReplyError, MAX_REPLY_BYTES};
        assert_eq!(
            graph_snapshot_probe(BrowserExecutionMode::TraditionalWeb),
            Err(ReplyError::Denied)
        );
        assert_eq!(
            graph_snapshot_probe(BrowserExecutionMode::QualiaNative),
            Err(ReplyError::Denied)
        );
        let body = graph_snapshot_probe(BrowserExecutionMode::HybridAugmented).unwrap();
        assert!(body.len() <= MAX_REPLY_BYTES);
        let doc: serde_json::Value = serde_json::from_str(&body).unwrap();
        let row = doc["quins"][0].as_array().unwrap();
        assert_eq!(row.len(), 6);
        assert!(row.iter().all(|lane| lane
            .as_str()
            .is_some_and(|word| word.parse::<u64>().is_ok())));
    }

    #[test]
    fn uniform_is_two_lanes_and_caps_the_harmonic_and_raymarch_budgets() {
        let mut view = HoloViewportControl::new(u8::MAX, false);
        view.raymarch_steps = u32::MAX;
        view.mode = HoloShaderVisualizationMode::AnatomicalRadialDepth;
        let block = uniform_data(&view, 320, 240, 0.125);
        assert_eq!(block.len(), 32);
        let lanes = block
            .chunks_exact(4)
            .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
            .collect::<Vec<_>>();
        assert_eq!(
            &lanes[3..6],
            &[
                0.125_f32.to_bits(),
                320.0_f32.to_bits(),
                240.0_f32.to_bits()
            ]
        );
        assert_eq!(lanes[7] & 63, 32);
        assert_eq!((lanes[7] >> 6) & 7, 4);
        assert_eq!((lanes[7] >> 9) & 3, 2);
        assert_eq!(lanes[6] & 1023, 716);
    }

    #[test]
    fn controls_keep_projection_finite_and_reset_the_camera() {
        let mut view = HoloViewportControl::new(2, true);
        let initial = view.project(CORNERS[0]);
        view.orbit(80.0, -200.0);
        view.zoom_by(2000.0);
        assert!(view.project(CORNERS[0]).into_iter().all(f32::is_finite));
        assert_ne!(view.project(CORNERS[0]), initial);
        view.reset_camera();
        assert_eq!(view.project(CORNERS[0]), initial);
        assert_eq!(PROXY_TRIANGLES.len(), 12);
    }
}
