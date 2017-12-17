use io::TextRenderer;
use ui::Size;
use resource::Point;

pub struct BufferedTextRenderer {
    buffer: Vec<char>,
    cursor_pos: Point,
    size: Size,
}

impl BufferedTextRenderer {
    pub fn new(size: Size) -> BufferedTextRenderer {
        BufferedTextRenderer {
            buffer: vec![' '; (size.product() as usize)],
            cursor_pos: Point::as_zero(),
            size: size,
        }
    }

    fn set_char_at_cursor(&mut self, c: char) {
        *self.buffer.get_mut((self.cursor_pos.x + self.cursor_pos.y *
                              self.size.width) as usize).unwrap() = c;
        self.cursor_pos.add_x(1);
    }

    fn cursor_pos_valid(&self) -> bool {
        self.cursor_pos.in_bounds(self.size.width, self.size.height)
    }

    pub fn clear(&mut self) {
        self.buffer.iter_mut().for_each(|c| *c = ' ');
    }

    pub fn get_line(&self, y: i32) -> String {
        let s = &self.buffer[(y * self.size.width) as usize..
            ((y + 1) * self.size.width) as usize];

        let out: String = s.iter().collect();
        out
    }
}

impl TextRenderer for BufferedTextRenderer {
    fn render_char(&mut self, c: char) {
        if self.cursor_pos_valid() {
            self.set_char_at_cursor(c);
        }
    }

    fn render_string(&mut self, s: &str) {
        for c in s.chars() {
            self.render_char(c);
        }
    }

    fn render_chars(&mut self, cs: &[char]) {
        for c in cs.iter() {
            self.render_char(*c);
        }
    }

    fn set_cursor_pos(&mut self, x: i32, y: i32) {
        self.cursor_pos.set(x, y);
    }
}