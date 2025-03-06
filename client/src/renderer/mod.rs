mod renderthing;

use std::sync::{Arc, Mutex, RwLock};

use libgame::{board::TileState, pos::Position};
use renderthing::{frame::RenderFrame, window::RendererWindowConfig, Renderer};
use winit::event::{MouseButton, WindowEvent};

use crate::State;

pub fn run(state_arc: Arc<RwLock<State>>) {
    let renderer_state = RendererState {
        global_state: state_arc,
        mouse_tile_pos: None,
        mouse_pressed: false,
        width: 0,
        height: 0,
    };

    let renderer_state_arc = Arc::new(Mutex::new(renderer_state));
    let draw_state_arc = renderer_state_arc.clone();
    let event_state_arc = renderer_state_arc.clone();

    let renderer = Renderer::new(RendererWindowConfig {
        title: "ml-life-killer".to_owned(),
        width: 480,
        height: 480,
        target_fps: 30,
        draw_callback: Box::new(move |frame| {
            let mut state = draw_state_arc.lock().unwrap();
            draw(&mut state, frame);
        }),
        event_callback: Some(Box::new(move |event| {
            let mut state = event_state_arc.lock().unwrap();
            on_event(&mut state, event);
        })),
    })
    .unwrap();

    renderer.run().unwrap();
}

fn draw(state: &mut RendererState, mut frame: RenderFrame) {
    state.width = frame.width;
    state.height = frame.height;

    let global_state = state.global_state.read().unwrap();

    let tile_width = frame.width / global_state.game.board.width as u32;
    let tile_height = frame.height / global_state.game.board.height as u32;

    const HALF_TILE_MARGIN: u32 = 1;

    frame.fill([10, 10, 10, 255]);

    for (tile_pos, tile) in global_state.game.board.enumerate_tiles() {
        let tile_screen_x = tile_pos.x as u32 * tile_width;
        let tile_screen_y = tile_pos.y as u32 * tile_height;

        let color = match tile {
            TileState::Alive => [255; 4],
            TileState::Dead => [0, 0, 0, 255],
        };

        frame.draw_square(
            tile_screen_x + HALF_TILE_MARGIN,
            tile_screen_y + HALF_TILE_MARGIN,
            tile_width - HALF_TILE_MARGIN * 2,
            tile_height - HALF_TILE_MARGIN * 2,
            color,
        );
    }
}

fn on_event(state: &mut RendererState, event: &WindowEvent) {
    let click = match event {
        WindowEvent::MouseInput {
            state: mouse_state,
            button,
            ..
        } => {
            if *button == MouseButton::Left {
                state.mouse_pressed = mouse_state.is_pressed();
                state.mouse_pressed
            } else {
                false
            }
        }
        WindowEvent::CursorMoved { position, .. } => {
            let mouse_pos = position.cast::<u32>();

            let global_state = state.global_state.read().unwrap();
            let tile_pos = Position {
                x: (mouse_pos.x * global_state.game.board.width as u32 / state.width) as usize,
                y: (mouse_pos.y * global_state.game.board.height as u32 / state.height) as usize,
            };

            let prev_tile_pos = state.mouse_tile_pos;
            state.mouse_tile_pos = Some(tile_pos);

            state.mouse_pressed && prev_tile_pos != state.mouse_tile_pos
        }
        _ => false,
    };

    if click && let Some(mouse_tile_pos) = state.mouse_tile_pos {
        let mut global_state = state.global_state.write().unwrap();

        let clicked_tile = global_state.game.board.tile_mut(mouse_tile_pos);

        if let Some(clicked_tile) = clicked_tile {
            *clicked_tile = match clicked_tile {
                TileState::Alive => TileState::Dead,
                TileState::Dead => TileState::Alive,
            };
        }
    }
}

struct RendererState {
    global_state: Arc<RwLock<State>>,
    mouse_tile_pos: Option<Position>,
    mouse_pressed: bool,
    width: u32,
    height: u32,
}
