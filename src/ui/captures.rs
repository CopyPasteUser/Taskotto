//! Optional, genuine egui paint-output capture for release documentation.
//! No display server, mock process entries, external renderer, or OS assets.
use super::*;
use eframe::egui::{ColorImage, ImageData, TextureId, epaint::Primitive};
use std::{collections::HashMap, fs, path::Path, thread, time::Duration};

const WIDTH: usize = 1280;
const HEIGHT: usize = 800;
const CLASSIC98_WIDTH: usize = 650;
const CLASSIC98_HEIGHT: usize = 510;

#[test]
#[ignore = "run explicitly with TASKOTTO_SCREENSHOT_DIR=assets/screenshots"]
fn generate_headless_ui_screenshots() {
    let directory = std::env::var_os("TASKOTTO_SCREENSHOT_DIR")
        .expect("set TASKOTTO_SCREENSHOT_DIR to the intended screenshot directory");
    let directory = Path::new(&directory);
    fs::create_dir_all(directory).expect("create screenshot directory");

    // Every pixel is painted from Taskotto's actual egui primitives; every
    // displayed process/metric comes from the container's real Linux /proc.
    let mut sampler = Monitor::default();
    let mut history = History::default();
    let mut latest = None;
    for _ in 0..24 {
        let snapshot = sampler.sample().expect("read live Linux procfs");
        history.record(&snapshot);
        latest = Some(snapshot);
        thread::sleep(Duration::from_millis(150));
    }
    let snapshot = latest.expect("at least one sample");

    for (theme, page, file) in [
        (
            Theme::Windows98,
            Page::Processes,
            "taskotto-win98-close-program.png",
        ),
        (
            Theme::Windows98,
            Page::Applications,
            "taskotto-win98-process-report.png",
        ),
        (
            Theme::Windows98,
            Page::Performance,
            "taskotto-win98-system-monitor.png",
        ),
        (
            Theme::Windows98,
            Page::Networking,
            "taskotto-win98-network.png",
        ),
        (
            Theme::Windows98,
            Page::Details,
            "taskotto-win98-information.png",
        ),
        (
            Theme::Windows7,
            Page::Applications,
            "taskotto-win7-applications.png",
        ),
        (
            Theme::Windows7,
            Page::Processes,
            "taskotto-win7-processes.png",
        ),
        (
            Theme::Windows7,
            Page::Services,
            "taskotto-win7-services.png",
        ),
        (
            Theme::Windows7,
            Page::Performance,
            "taskotto-win7-performance.png",
        ),
        (
            Theme::Windows7,
            Page::Networking,
            "taskotto-win7-networking.png",
        ),
        (Theme::Windows7, Page::Users, "taskotto-win7-users.png"),
        (
            Theme::Windows7,
            Page::Details,
            "taskotto-win7-properties.png",
        ),
        (
            Theme::Windows10,
            Page::Processes,
            "taskotto-win10-processes.png",
        ),
        (
            Theme::Windows10,
            Page::Processes,
            "taskotto-win10-vertical.png",
        ),
        (
            Theme::Windows10,
            Page::Performance,
            "taskotto-win10-performance.png",
        ),
        (
            Theme::Windows10,
            Page::AppHistory,
            "taskotto-win10-app-history.png",
        ),
        (
            Theme::Windows10,
            Page::Startup,
            "taskotto-win10-startup.png",
        ),
        (
            Theme::Windows10,
            Page::Services,
            "taskotto-win10-services.png",
        ),
        (Theme::Windows10, Page::Users, "taskotto-win10-users.png"),
        (
            Theme::Windows10,
            Page::Details,
            "taskotto-win10-details.png",
        ),
        (
            Theme::XpLuna,
            Page::Applications,
            "taskotto-xp-luna-applications.png",
        ),
        (
            Theme::XpLuna,
            Page::Applications,
            "taskotto-xp-luna-new-task.png",
        ),
        (
            Theme::XpLuna,
            Page::Processes,
            "taskotto-xp-luna-processes.png",
        ),
        (
            Theme::XpLuna,
            Page::Performance,
            "taskotto-xp-luna-performance.png",
        ),
        (
            Theme::XpLuna,
            Page::Networking,
            "taskotto-xp-luna-networking.png",
        ),
        (Theme::XpLuna, Page::Details, "taskotto-xp-luna-details.png"),
        (Theme::Macos, Page::Processes, "taskotto-macos-cpu.png"),
        (Theme::Macos, Page::Performance, "taskotto-macos-memory.png"),
        (Theme::Macos, Page::MacEnergy, "taskotto-macos-energy.png"),
        (Theme::Macos, Page::MacDisk, "taskotto-macos-disk.png"),
        (Theme::Macos, Page::Networking, "taskotto-macos-network.png"),
        (Theme::Macos, Page::Processes, "taskotto-macos-info.png"),
    ] {
        if let Ok(filter) = std::env::var("TASKOTTO_SCREENSHOT_FILTER")
            && !file.contains(&filter)
        {
            continue;
        }
        let (width, height) = if theme == Theme::Windows98 {
            (CLASSIC98_WIDTH, CLASSIC98_HEIGHT)
        } else {
            (WIDTH, HEIGHT)
        };
        let ctx = egui::Context::default();
        let settings = Settings {
            theme,
            refresh_ms: 10_000,
            layout_mode: if file.contains("vertical") {
                LayoutMode::Sidebar
            } else {
                LayoutMode::TopTabs
            },
            ..Settings::default()
        };
        apply_theme(&ctx, &settings);
        let mut app = TaskottoApp {
            worker: Worker::start(10_000, ctx.clone()),
            settings,
            snapshot: Some(snapshot.clone()),
            history: History {
                cpu: history.cpu.clone(),
                cores: history.cores.clone(),
                memory: history.memory.clone(),
                received: history.received.clone(),
                sent: history.sent.clone(),
                disk_read: history.disk_read.clone(),
                disk_write: history.disk_write.clone(),
            },
            page,
            performance_focus: PerformanceFocus::Cpu,
            query: String::new(),
            focus_search: false,
            selected: if page == Page::Details || file.contains("macos-info") {
                snapshot.processes.first().map(|p| (p.pid, p.start_ticks))
            } else {
                None
            },
            pending: None,
            notification: None,
            theme_accent_input: String::new(),
            theme_surface_input: String::new(),
            theme_text_input: String::new(),
            my_uid: unsafe { libc::geteuid() },
            users: load_users(),
            system_pages: rices::SystemPages::default(),
            new_task: NewTaskDialog::default(),
            mac_info_open: file.contains("macos-info"),
        };
        if file.contains("new-task") {
            app.new_task.open = true;
        }
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                Vec2::new(width as f32, height as f32),
            )),
            ..Default::default()
        };
        let mut textures = HashMap::new();
        let mut pixels = None;
        // egui intentionally estimates panel dimensions on the first frame.
        // Render several consecutive frames so navigation/topbar clipping
        // matches the settled native GUI layout; retain incremental atlas data.
        for frame in 0..3 {
            let output = ctx.run(input.clone(), |ctx| {
                app.draw_contents(ctx);
            });
            if page == Page::Services && frame == 0 {
                // The real systemctl query runs asynchronously; give the test
                // capture a bounded opportunity to show its actual result or
                // permission/error state instead of the initial spinner.
                thread::sleep(Duration::from_millis(350));
            }
            pixels = Some(
                paint_egui(&ctx, output, &mut textures, frame == 2, width, height)
                    .expect("rasterize original egui primitives"),
            );
        }
        let pixels = pixels.expect("rendered at least one frame");
        save_png(&directory.join(file), &pixels, width, height).expect("write screenshot");
    }
}

fn save_png(
    path: &Path,
    pixels: &[u8],
    width: usize,
    height: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::create(path)?;
    let mut encoder = png::Encoder::new(file, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(pixels)?;
    Ok(())
}

fn paint_egui(
    ctx: &egui::Context,
    output: egui::FullOutput,
    textures: &mut HashMap<TextureId, ColorImage>,
    render: bool,
    width: usize,
    height: usize,
) -> Result<Vec<u8>, String> {
    for (id, delta) in output.textures_delta.set {
        let ImageData::Color(image) = delta.image;
        if let Some([x, y]) = delta.pos {
            let destination = textures
                .get_mut(&id)
                .ok_or("partial texture without base")?;
            for row in 0..image.height() {
                let from = row * image.width();
                let to = (row + y) * destination.width() + x;
                destination.pixels[to..to + image.width()]
                    .copy_from_slice(&image.pixels[from..from + image.width()]);
            }
        } else {
            textures.insert(id, (*image).clone());
        }
    }
    if !render {
        return Ok(Vec::new());
    }
    let pixels_per_point = output.pixels_per_point;
    let primitives = ctx.tessellate(output.shapes, pixels_per_point);
    // Native eframe clears the framebuffer before painting. Some panel fills
    // are intentionally translucent: an all-white initial canvas would show
    // a spurious white strip underneath those panels in dark/classic themes.
    let fill = ctx.style().visuals.panel_fill;
    let mut image = [fill.r(), fill.g(), fill.b(), 255].repeat(width * height);
    for clipped in primitives {
        let Primitive::Mesh(mesh) = clipped.primitive else {
            return Err("headless screenshot encountered a paint callback".to_string());
        };
        let texture = textures
            .get(&mesh.texture_id)
            .ok_or("texture image not supplied")?;
        let clip = clipped.clip_rect;
        let clip_left = (clip.left() * pixels_per_point).floor().max(0.0) as usize;
        let clip_top = (clip.top() * pixels_per_point).floor().max(0.0) as usize;
        let clip_right = (clip.right() * pixels_per_point).ceil().min(width as f32) as usize;
        let clip_bottom = (clip.bottom() * pixels_per_point).ceil().min(height as f32) as usize;
        let (triangles, remainder) = mesh.indices.as_chunks::<3>();
        debug_assert!(remainder.is_empty());
        for indices in triangles {
            let vertices = [
                mesh.vertices[indices[0] as usize],
                mesh.vertices[indices[1] as usize],
                mesh.vertices[indices[2] as usize],
            ];
            paint_triangle(
                &mut image,
                vertices,
                texture,
                (clip_left, clip_top, clip_right, clip_bottom),
                pixels_per_point,
                width,
            );
        }
    }
    Ok(image)
}

fn paint_triangle(
    destination: &mut [u8],
    verts: [egui::epaint::Vertex; 3],
    texture: &ColorImage,
    clip: (usize, usize, usize, usize),
    scale: f32,
    width: usize,
) {
    let positions = verts.map(|vertex| egui::pos2(vertex.pos.x * scale, vertex.pos.y * scale));
    let area = edge(positions[0], positions[1], positions[2]);
    if area.abs() < f32::EPSILON {
        return;
    }
    let left = positions
        .iter()
        .map(|p| p.x)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(clip.0 as f32) as usize;
    let top = positions
        .iter()
        .map(|p| p.y)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(clip.1 as f32) as usize;
    let right = positions
        .iter()
        .map(|p| p.x)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil()
        .min(clip.2 as f32) as usize;
    let bottom = positions
        .iter()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil()
        .min(clip.3 as f32) as usize;
    if left >= right || top >= bottom {
        return;
    }
    for y in top..bottom {
        for x in left..right {
            let point = egui::pos2(x as f32 + 0.5, y as f32 + 0.5);
            let weights = [
                edge(positions[1], positions[2], point) / area,
                edge(positions[2], positions[0], point) / area,
                edge(positions[0], positions[1], point) / area,
            ];
            if weights.iter().any(|weight| *weight < -0.000_01) {
                continue;
            }
            let u = verts
                .iter()
                .zip(weights)
                .map(|(vertex, weight)| vertex.uv.x * weight)
                .sum();
            let v = verts
                .iter()
                .zip(weights)
                .map(|(vertex, weight)| vertex.uv.y * weight)
                .sum();
            let texture_rgba = sample_texture(texture, u, v);
            let mut color = [0.0_f32; 4];
            for (vertex, weight) in verts.iter().zip(weights) {
                let channels = [
                    vertex.color.r(),
                    vertex.color.g(),
                    vertex.color.b(),
                    vertex.color.a(),
                ];
                for channel in 0..4 {
                    color[channel] += channels[channel] as f32 / 255.0 * weight;
                }
            }
            let alpha = (color[3] * texture_rgba[3]).clamp(0.0, 1.0);
            let offset = (y * width + x) * 4;
            for channel in 0..3 {
                let foreground = (color[channel] * texture_rgba[channel]).clamp(0.0, 1.0);
                destination[offset + channel] = ((foreground
                    + destination[offset + channel] as f32 / 255.0 * (1.0 - alpha))
                    * 255.0)
                    .clamp(0.0, 255.0) as u8;
            }
            destination[offset + 3] = 255;
        }
    }
}

fn edge(a: egui::Pos2, b: egui::Pos2, point: egui::Pos2) -> f32 {
    (b.x - a.x) * (point.y - a.y) - (b.y - a.y) * (point.x - a.x)
}

fn sample_texture(texture: &ColorImage, u: f32, v: f32) -> [f32; 4] {
    let width = texture.width();
    let height = texture.height();
    let fx = (u * width as f32 - 0.5).clamp(0.0, (width - 1) as f32);
    let fy = (v * height as f32 - 0.5).clamp(0.0, (height - 1) as f32);
    let x0 = fx.floor() as usize;
    let y0 = fy.floor() as usize;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);
    let weights = [
        ((1.0 - fx.fract()) * (1.0 - fy.fract()), x0, y0),
        (fx.fract() * (1.0 - fy.fract()), x1, y0),
        ((1.0 - fx.fract()) * fy.fract(), x0, y1),
        (fx.fract() * fy.fract(), x1, y1),
    ];
    let mut result = [0.0; 4];
    for (weight, x, y) in weights {
        let texel = texture.pixels[y * width + x];
        for (index, value) in [texel.r(), texel.g(), texel.b(), texel.a()]
            .iter()
            .enumerate()
        {
            result[index] += weight * (*value as f32 / 255.0);
        }
    }
    result
}

#[test]
fn rasterizer_honors_triangle_bounds_and_clip_rect() {
    let mut destination = vec![0_u8; WIDTH * HEIGHT * 4];
    let vertex = |x, y| egui::epaint::Vertex {
        pos: egui::pos2(x, y),
        uv: egui::pos2(0.5, 0.5),
        color: Color32::RED,
    };
    let texture = ColorImage::filled([1, 1], Color32::WHITE);
    paint_triangle(
        &mut destination,
        [vertex(0.0, 0.0), vertex(16.0, 0.0), vertex(0.0, 16.0)],
        &texture,
        (4, 4, 8, 8),
        1.0,
        WIDTH,
    );
    assert_eq!(&destination[(5 * WIDTH + 5) * 4..][..3], &[255, 0, 0]);
    assert_eq!(&destination[(3 * WIDTH + 3) * 4..][..3], &[0, 0, 0]);
    assert_eq!(&destination[(8 * WIDTH + 8) * 4..][..3], &[0, 0, 0]);
}
