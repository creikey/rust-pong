use raylib::prelude::*;

fn clamp(d: f32, min: f32, max: f32) -> f32 {
    let t;
    if d < min {
        t = min;
    } else {
        t = d;
    }
    if t > max {
        return max;
    } else {
        return t;
    }
}

fn key_strength(rl: &RaylibHandle, key: KeyboardKey) -> f32 {
    if rl.is_key_down(key) {
        return 1.0;
    } else {
        return 0.0;
    }
}

fn main() {
    let screen_size = Vector2::new(1000.0, 800.0);
    let (mut rl, thread) = raylib::init()
        .size(screen_size.x as i32, screen_size.y as i32)
        .title("Rust Pong")
        .build();

    let paddle_size = Vector2::new(25.0, 175.0);
    let ball_size = 20.0;
    let ball_speed = paddle_size.x * 10.0;

    let mut l_paddle_pos = 0.0;
    let mut r_paddle_pos = 0.0;
    let mut ball_pos = screen_size / 2.0;
    let mut ball_direction = Vector2::new(1.0, -1.0).normalized();

    while !rl.window_should_close() {
        let l_vertical =
            key_strength(&rl, KeyboardKey::KEY_S) - key_strength(&rl, KeyboardKey::KEY_W);
        l_paddle_pos += l_vertical * rl.get_frame_time() * paddle_size.y * 1.5;
        l_paddle_pos = clamp(l_paddle_pos, 0.0, screen_size.y - paddle_size.y);

        let r_vertical =
            key_strength(&rl, KeyboardKey::KEY_K) - key_strength(&rl, KeyboardKey::KEY_I);
        r_paddle_pos += r_vertical * rl.get_frame_time() * paddle_size.y * 1.5;
        r_paddle_pos = clamp(r_paddle_pos, 0.0, screen_size.y - paddle_size.y);

        ball_pos += ball_direction * rl.get_frame_time() * ball_speed;
        let ball_hit;
        let paddle_pos_to_check;
        let ball_snap_to_x_pos;
        if ball_pos.x + ball_size > screen_size.x - paddle_size.x {
            ball_hit = true;
            paddle_pos_to_check = r_paddle_pos;
            ball_snap_to_x_pos = screen_size.x - paddle_size.x - ball_size;
        } else if ball_pos.x - ball_size < paddle_size.x {
            ball_hit = true;
            paddle_pos_to_check = l_paddle_pos;
            ball_snap_to_x_pos = paddle_size.x + ball_size;
        } else {
            ball_hit = false;
            paddle_pos_to_check = 0.0;
            ball_snap_to_x_pos = 0.0;
        }
        if ball_hit
            && ball_pos.y >= paddle_pos_to_check
            && ball_pos.y <= paddle_pos_to_check + paddle_size.y
        {
            ball_direction.x *= -1.0;
            ball_pos.x = ball_snap_to_x_pos;
        }
        if ball_pos.y - ball_size < 0.0 || ball_pos.y + ball_size > screen_size.y {
            ball_direction.y *= -1.0;
        }

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        d.draw_rectangle(
            0,
            l_paddle_pos as i32,
            paddle_size.x as i32,
            paddle_size.y as i32,
            Color::BLACK,
        );
        d.draw_rectangle(
            (screen_size.x - paddle_size.x) as i32,
            r_paddle_pos as i32,
            paddle_size.x as i32,
            paddle_size.y as i32,
            Color::BLACK,
        );
        d.draw_circle_v(ball_pos, ball_size, Color::RED);
    }
}
