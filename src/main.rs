use std::time::Duration;

use nannou::{
    lyon::geom::euclid::num::Floor,
    prelude::{rgb::Rgb, MouseButton, *},
};
use nannou_egui::{self, egui, Egui};

fn main() {
    nannou::app(model).update(update).run();
}

struct Settings {
    width: u32,
    height: u32,
    delay: u64,
}

struct Model {
    egui: Egui,
    settings: Settings,
    field: Vec<bool>,
    active: bool,
    mouse_pos: Vec2,
    current_step: Duration,
}

fn model(app: &App) -> Model {
    // Create window
    let window_id = app
        .new_window()
        .view(view)
        .raw_event(raw_window_event)
        .mouse_moved(mouse_moved)
        .mouse_released(mouse_released)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);

    let settings = Settings {
        width: 120,
        height: 80,
        delay: 200,
    };

    let mut field: Vec<bool> = Vec::new();
    for _ in 0..(settings.width * settings.height) {
        field.push(false);
    }

    Model {
        egui,
        field,
        settings,
        active: false,
        mouse_pos: vec2(0.0, 0.0),
        current_step: Duration::ZERO,
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;
    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    egui::Window::new("Settings").show(&ctx, |ui| {
        let clicked = ui
            .button(match model.active {
                true => "Stop",
                false => "Start",
            })
            .clicked();
        if clicked {
            model.active = !model.active;
        }

        ui.label("Delay (ms):");
        ui.add(egui::Slider::new(&mut model.settings.delay, 100..=2000));

        let clear_clicked = ui.button("Clear").clicked();
        if clear_clicked {
            for i in 0..model.field.len() {
                model.field[i] = false;
            }
        }
    });

    if !model.active {
        return;
    }

    model.current_step += update.since_last;
    if model.current_step > Duration::from_millis(model.settings.delay) {
        model.field = update_field(&mut model.field, &mut model.settings);
        model.current_step = Duration::ZERO;
    }
}

fn update_field(field: &mut Vec<bool>, settings: &mut Settings) -> Vec<bool> {
    let mut copy = field.clone();

    for i in 0..(settings.height * settings.width) {
        let x = (i % settings.width).floor();
        let y = (i / settings.width).floor();

        let mut alive_neighbours = 0;

        for dy in 0..3 {
            for dx in 0..3 {
                // skip tile itself
                if dx == 1 && dy == 1 {
                    continue;
                }

                // skip if out of bounds
                let x_n = (x + dx).to_i32().unwrap() - 1;
                let y_n = (y + dy).to_i32().unwrap() - 1;

                if x_n < 0
                    || x_n >= settings.width.to_i32().unwrap()
                    || y_n < 0
                    || y_n >= settings.height.to_i32().unwrap()
                {
                    continue;
                }

                let thing = field[(x_n + (y_n * settings.width.to_i32().unwrap()))
                    .to_usize()
                    .unwrap()];
                if thing {
                    alive_neighbours += 1;
                }
            }
        }

        let tile: &mut bool = &mut copy[(x + (y * settings.width)).to_usize().unwrap()];
        if *tile {
            // tile is alive
            if alive_neighbours != 2 && alive_neighbours != 3 {
                *tile = false;
            }
        } else {
            // tile is dead
            if alive_neighbours == 3 {
                *tile = true;
            }
        }
    }

    return copy;
}

fn mouse_moved(app: &App, model: &mut Model, pos: Point2) {
    let bounds = app.window_rect();
    let x = pos.x - bounds.left();
    let y = pos.y - bounds.bottom();
    let position = vec2(x, y);
    model.mouse_pos = position;
}

fn mouse_released(app: &App, model: &mut Model, _button: MouseButton) {
    if _button != MouseButton::Left {
        return;
    }

    let bounds = app.window_rect();
    let x_step = bounds.w() / model.settings.width.to_f32().unwrap();
    let y_step = bounds.h() / model.settings.height.to_f32().unwrap();

    let x = ((model.mouse_pos.x) / x_step).floor();
    let y = ((model.mouse_pos.y) / y_step).floor();

    let index = (x + (y * model.settings.width.to_f32().unwrap()).floor())
        .to_usize()
        .unwrap();
    model.field[index] = !model.field[index];
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    let bound = app.window_rect();

    draw_tiles(&bound, model, &draw);
    draw_grid(&bound, model, &draw);

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}

fn draw_tiles(bounds: &Rect, model: &Model, draw: &Draw) {
    let height = bounds.h() / model.settings.height.to_f32().unwrap();
    let width = bounds.w() / model.settings.width.to_f32().unwrap();

    for i in 0..(model.settings.width * model.settings.height) {
        let x = i % model.settings.width;
        let y = i / model.settings.width;
        let tile = model.field[i.to_usize().unwrap()];

        let color: Rgb = match tile {
            true => Rgb::new(0.6, 0.6, 0.6),
            false => Rgb::new(0.15, 0.15, 0.15),
        };

        draw.rect()
            .width(width)
            .height(height)
            .x_y(
                bounds.left() + (width / 2.0) + x.to_f32().unwrap() * width,
                bounds.bottom() + (height / 2.0) + y.to_f32().unwrap() * height,
            )
            .color(color);
    }
}

fn draw_grid(bounds: &Rect, model: &Model, draw: &Draw) {
    let color: Rgb = Rgb::new(0.4, 0.4, 0.4);
    let y_offset = bounds.h() / model.settings.height.to_f32().unwrap();
    for y in 0..model.settings.height {
        draw.line()
            .start(vec2(
                bounds.left(),
                bounds.bottom() + y_offset * y.to_f32().unwrap(),
            ))
            .end(vec2(
                bounds.right(),
                bounds.bottom() + y_offset * y.to_f32().unwrap(),
            ))
            .color(color)
            .weight(1.0);
    }

    let x_offset = bounds.w() / model.settings.width.to_f32().unwrap();
    for x in 0..model.settings.width {
        draw.line()
            .start(vec2(
                bounds.left() + x_offset * x.to_f32().unwrap(),
                bounds.bottom(),
            ))
            .end(vec2(
                bounds.left() + x_offset * x.to_f32().unwrap(),
                bounds.top(),
            ))
            .color(color)
            .weight(1.0);
    }
}
