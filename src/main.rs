use edited_text::EditedText;
use speedy2d::color::Color;
use speedy2d::dimen::{UVec2, Vec2};
use speedy2d::font::{Font, FormattedTextBlock, TextLayout, TextOptions};
use speedy2d::shape::Rect;
use speedy2d::window::{
    KeyScancode, VirtualKeyCode, WindowHandler, WindowHelper, WindowStartupInfo,
};
use speedy2d::{Graphics2D, Window};

mod edited_text;

const TEXT_SCALE: f32 = 64.0;

fn main() {
    let window = Window::new_centered("Speedy2D: Input Callbacks Example", (640, 480)).unwrap();

    let font = Font::new(include_bytes!("../assets/fonts/NotoSans-Regular.ttf")).unwrap();

    let mut large_text = String::new();
    let mut lines = vec![];

    for y in 0..30000 {
        let mut text = String::new();
        for x in 0..25 {
            let str = format!("{},{}|", x, y);
            large_text.push_str(str.as_str());
            text.push_str(str.as_str());
        }

        large_text.push_str("\n");
        lines.push(text);
    }

    // 30000 * 25 * 4 = 3,000,000
    // that's 3 megabytes of text

    window.run_loop(MyWindowHandler {
        scroll_pos: Vec2::ZERO,
        font,
        edited_text: large_text.into(),
        lines,
        cursor_pos: UVec2::ZERO,
    })
}

struct MyWindowHandler {
    font: Font,
    scroll_pos: Vec2,
    edited_text: EditedText,
    lines: Vec<String>,
    cursor_pos: UVec2,
}

impl MyWindowHandler {
    fn cursor_pos_to_index(&self) -> usize {
        let mut index = 0;
        for i in 0..self.cursor_pos.y as usize {
            index += self.lines[i].len() + 1;
        }
        index += self.cursor_pos.x as usize;
        index
    }

    fn insert_char(&mut self, c: char) {
        let index = self.cursor_pos_to_index();
        self.edited_text.add_char(c, index);
        self.lines = self.edited_text.get_lines();
        self.cursor_pos.x += 1;
    }

    fn remove_char(&mut self) {
        let index = self.cursor_pos_to_index();
        if index == 0 {
            return;
        }
        self.edited_text.remove_char(index - 1);
        self.lines = self.edited_text.get_lines();
        if self.cursor_pos.x > 0 {
            self.cursor_pos.x -= 1;
        } else if self.cursor_pos.y > 0 {
            self.cursor_pos.y -= 1;
            self.cursor_pos.x = self.lines[self.cursor_pos.y as usize].len() as u32;
        }
    }

    fn new_line(&mut self) {
        let index = self.cursor_pos_to_index();
        self.edited_text.add_char('\n', index);
        self.lines = self.edited_text.get_lines();
        self.cursor_pos.y += 1;
        self.cursor_pos.x = 0;
    }

    fn fmt_text(&self, lines: (usize, usize)) -> FormattedTextBlock {
        let text = self.lines[lines.0..lines.1].join("\n");
        self.font.layout_text(&text, TEXT_SCALE, TextOptions::new())
    }

    fn get_lineheight(&self) -> f32 {
        self.font
            .layout_text("a", TEXT_SCALE, TextOptions::new())
            .height()
    }

    fn get_cursor_pos(&self) -> Vec2 {
        let layout = self.font.layout_text(
            &self.lines[self.cursor_pos.y as usize][0..self.cursor_pos.x as usize],
            TEXT_SCALE,
            TextOptions::new(),
        );
        let mut size = layout.size();
        let line_height = self.get_lineheight();
        size.y = line_height * self.cursor_pos.y as f32;
        size
    }

    /// Returns the start and end line of the lines that are visible on the screen
    /// as well as the y position on the screen of the start of the first visible line
    fn get_visible_lines(&self, helper: &WindowHelper) -> ((usize, usize), f32) {
        let line_height = self.get_lineheight();
        let window_size = helper.get_size_pixels();
        let mut start_line = (-self.scroll_pos.y / line_height) as usize;
        let mut end_line = start_line + (window_size.y as f32 / line_height) as usize + 2;
        let y_pos = self.scroll_pos.y % line_height;
        if end_line > self.lines.len() {
            end_line = self.lines.len();
        }
        if start_line > self.lines.len() {
            start_line = self.lines.len();
        }
        ((start_line, end_line), y_pos)
    }
}

impl WindowHandler for MyWindowHandler {
    fn on_start(&mut self, helper: &mut WindowHelper, _info: WindowStartupInfo) {
        helper.set_cursor_visible(false);
    }

    fn on_draw(&mut self, helper: &mut WindowHelper, graphics: &mut Graphics2D) {
        // Clear the screen
        graphics.clear_screen(Color::from_rgb(0.8, 0.9, 1.0));

        // Draw the cursor
        let cursor_pos = self.get_cursor_pos();
        let cursor_size = Vec2::new(2.0, TEXT_SCALE);

        {
            let mut offset_cursor_pos = cursor_pos + self.scroll_pos;
            while offset_cursor_pos.y > helper.get_size_pixels().y as f32 - cursor_size.y * 2.0 {
                self.scroll_pos.y -= helper.get_size_pixels().y as f32 / 2.0;
                offset_cursor_pos = cursor_pos + self.scroll_pos;
            }
            while self.scroll_pos.y != 0. && offset_cursor_pos.y < cursor_size.y {
                self.scroll_pos.y += helper.get_size_pixels().y as f32 / 2.0;
                self.scroll_pos.y = self.scroll_pos.y.min(0.0);
                offset_cursor_pos = cursor_pos + self.scroll_pos;
            }

            while offset_cursor_pos.x > helper.get_size_pixels().x as f32 - cursor_size.y * 2.0 {
                self.scroll_pos.x -= helper.get_size_pixels().x as f32 / 2.0;
                offset_cursor_pos = cursor_pos + self.scroll_pos;
            }
            while self.scroll_pos.x != 0. && offset_cursor_pos.x < cursor_size.y {
                self.scroll_pos.x += helper.get_size_pixels().x as f32 / 2.0;
                self.scroll_pos.x = self.scroll_pos.x.min(0.0);
                offset_cursor_pos = cursor_pos + self.scroll_pos;
            }
        }
        graphics.draw_rectangle(
            Rect::new(
                cursor_pos + self.scroll_pos,
                cursor_pos + self.scroll_pos + cursor_size,
            ),
            Color::BLACK,
        );

        // Draw the text
        let ((start_line, end_line), y_pos) = self.get_visible_lines(helper);
        let text = self.fmt_text((start_line, end_line));
        graphics.draw_text(Vec2::new(self.scroll_pos.x, y_pos), Color::BLACK, &text);
    }

    fn on_key_down(
        &mut self,
        helper: &mut WindowHelper,
        virtual_key_code: Option<VirtualKeyCode>,
        _scancode: KeyScancode,
    ) {
        if let Some(key) = virtual_key_code {
            match key {
                VirtualKeyCode::Escape => {
                    // Exit the program when escape is pressed
                    std::process::exit(0);
                }
                VirtualKeyCode::Backspace => {
                    // Remove the last character from the text
                    self.remove_char();
                    helper.request_redraw();
                }
                VirtualKeyCode::Return => {
                    // Add a newline to the text
                    self.new_line();
                    helper.request_redraw();
                }
                VirtualKeyCode::Home => {
                    // Move the cursor to the start of the line
                    self.cursor_pos.x = 0;
                    helper.request_redraw();
                }
                VirtualKeyCode::End => {
                    // Move the cursor to the end of the line
                    self.cursor_pos.x = self.lines[self.cursor_pos.y as usize].len() as u32;
                    helper.request_redraw();
                }
                VirtualKeyCode::PageUp => {
                    // Move the cursor to the center of the document
                    self.cursor_pos.y = self.lines.len() as u32 / 2;
                    self.cursor_pos.x = self.lines[self.cursor_pos.y as usize].len() as u32 / 2;
                    helper.request_redraw();
                }
                VirtualKeyCode::PageDown => {
                    // Move the cursor to the end of the document
                    self.cursor_pos.y = self.lines.len() as u32 - 1;
                    self.cursor_pos.x = self.lines[self.cursor_pos.y as usize].len() as u32;
                    helper.request_redraw();
                }
                VirtualKeyCode::Left => {
                    // Move the cursor left
                    if self.cursor_pos.x > 0 {
                        self.cursor_pos.x -= 1;
                        helper.request_redraw();
                    } else if self.cursor_pos.y > 0 {
                        self.cursor_pos.y -= 1;
                        self.cursor_pos.x = self.lines[self.cursor_pos.y as usize].len() as u32;
                        helper.request_redraw();
                    }
                }
                VirtualKeyCode::Right => {
                    // Move the cursor right
                    if self.cursor_pos.x < self.lines[self.cursor_pos.y as usize].len() as u32 {
                        self.cursor_pos.x += 1;
                        helper.request_redraw();
                    } else if self.cursor_pos.y < self.lines.len() as u32 - 1 {
                        self.cursor_pos.y += 1;
                        self.cursor_pos.x = 0;
                        helper.request_redraw();
                    }
                }
                VirtualKeyCode::Down => {
                    println!("text: {:?}", self.edited_text);
                }
                _ => {}
            }
        }
    }

    fn on_keyboard_char(&mut self, helper: &mut WindowHelper, unicode_codepoint: char) {
        if (unicode_codepoint as u32) < 32 {
            return;
        }
        // Add the character to the text
        self.insert_char(unicode_codepoint);
        helper.request_redraw();
    }
}
