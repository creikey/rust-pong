use raylib::prelude::*;

// TODO create ImUI struct that manages UI rect logic and draws in one function. This would allow
// stuff that modifies raylib data like setting the clipboard, as the immediate mode UI logic
// would be done in process

fn rect_new_ex(position: Vector2, size: Vector2) -> Rectangle {
    Rectangle::new(position.x, position.y, size.x, size.y)
}

fn rect_pos(r: &Rectangle) -> Vector2 {
    Vector2::new(r.x, r.y)
}

fn rect_size(r: &Rectangle) -> Vector2 {
    Vector2::new(r.width, r.height)
}

/* tabling for now to use raylib rectangle
struct Rectangle {
    upper_left_corner: Vector2,
    size: Vector2,
}

impl Rectangle {
    fn has_point(&self, point: Vector2) -> bool {
        point.x >= self.upper_left_corner.x
            && point.x <= self.upper_left_corner.x + self.size.x
            && point.y >= self.upper_left_corner.y
            && point.y <= self.upper_left_corner.y + self.size.y
    }
}*/

pub fn button(d: &mut RaylibDrawHandle, upper_left_corner: Vector2, size: Vector2, text: &str) -> bool {
    let font_size = 50.0;
    let bounding_box = rect_new_ex(upper_left_corner, size);
    let hovered = bounding_box.check_collision_point_rec(d.get_mouse_position());

    let background_color = if hovered {
        Color::new(255, 255, 255, 255)
    } else {
        Color::new(170, 170, 170, 255)
    };

    d.draw_rectangle_v(
        rect_pos(&bounding_box),
        rect_size(&bounding_box),
        background_color,
    );
    let text_size = measure_text_ex(d.get_font_default(), text, font_size, 1.0);
    d.draw_text_ex(
        d.get_font_default(),
        text,
        upper_left_corner + size / 2.0 - text_size / 2.0,
        font_size,
        1.0,
        Color::BLACK,
    );

    d.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) && hovered
}