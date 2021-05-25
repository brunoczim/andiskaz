use crate::{
    color::{BasicColor, Color, Color2},
    coord,
    coord::{Coord, Coord2},
    error::ServicesOff,
    event::{Event, Key, KeyEvent, ResizeEvent},
    screen::Screen,
    string::TermString,
    style::Style,
    terminal::Terminal,
};
use std::mem;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputDialogItem {
    Ok,
    Cancel,
}

/// A dialog asking for user input, possibly filtered.
pub struct InputDialog<F>
where
    F: FnMut(char) -> bool,
{
    /// Filter of the valid characters for the input dialog's box.
    pub filter: F,
    /// The title of the input dialog.
    pub title: TermString,
    /// Initial buffer of the input dialog.
    pub buffer: String,
    /// Maximum size of the input box.
    pub max: Coord,
    /// Colors of the title.
    pub title_colors: Color2,
    /// Selected option colors.
    pub selected_colors: Color2,
    /// Unselected option colors.
    pub unselected_colors: Color2,
    /// Input box's cursor colors.
    pub cursor_colors: Color2,
    /// Input box colors.
    pub box_colors: Color2,
    /// Background of non-text areas.
    pub bg: Color,
    /// Position of the title.
    pub title_y: Coord,
    /// Padding lines inserted after the title.
    pub pad_after_title: Coord,
    /// Padding lines inserted after the box.
    pub pad_after_box: Coord,
    /// Padding lines inserted after the OK option.
    pub pad_after_ok: Coord,
}

impl<F> InputDialog<F>
where
    F: FnMut(char) -> bool,
{
    /// Creates a new input dialog, with the given title, initial buffer,
    /// maximum input size, and filter function.
    pub fn new(
        title: TermString,
        buffer: String,
        max: Coord,
        filter: F,
    ) -> Self {
        Self {
            title,
            buffer,
            filter,
            max,
            title_colors: Color2::default(),
            selected_colors: !Color2::default(),
            unselected_colors: Color2::default(),
            cursor_colors: Color2::default(),
            box_colors: !Color2::default(),
            bg: BasicColor::Black.into(),
            title_y: 1,
            pad_after_title: 2,
            pad_after_box: 2,
            pad_after_ok: 1,
        }
    }

    /// Gets user input without possibility of canceling it.
    pub async fn select(
        &mut self,
        term: &mut Terminal,
    ) -> Result<TermString, ServicesOff> {
        let mut selector = Selector::without_cancel(self, 0);
        selector.run(term).await?;
        Ok(selector.result())
    }

    /// Gets user input with the user possibly canceling it.
    pub async fn select_with_cancel(
        &mut self,
        term: &mut Terminal,
    ) -> Result<Option<TermString>, ServicesOff> {
        let mut selector = Selector::with_cancel(self, 0, InputDialogItem::Ok);
        selector.run(term).await?;
        Ok(selector.result_with_cancel())
    }
}

struct Selector<'dialog, F>
where
    F: FnMut(char) -> bool,
{
    dialog: &'dialog mut InputDialog<F>,
    buffer: Vec<char>,
    joined: String,
    cursor: usize,
    selected: InputDialogItem,
    has_cancel: bool,
    actual_max: Coord,
    valid_size: bool,
}

impl<'dialog, F> Selector<'dialog, F>
where
    F: FnMut(char) -> bool,
{
    fn new(
        dialog: &'dialog mut InputDialog<F>,
        cursor: usize,
        selected: InputDialogItem,
        has_cancel: bool,
    ) -> Self {
        Self {
            buffer: dialog.buffer.chars().collect(),
            joined: String::new(),
            cursor,
            selected,
            has_cancel,
            actual_max: 0,
            valid_size: true,
            dialog,
        }
    }

    fn without_cancel(
        dialog: &'dialog mut InputDialog<F>,
        cursor: usize,
    ) -> Self {
        Self::new(dialog, cursor, InputDialogItem::Ok, false)
    }

    fn with_cancel(
        dialog: &'dialog mut InputDialog<F>,
        cursor: usize,
        selected: InputDialogItem,
    ) -> Self {
        Self::new(dialog, cursor, selected, true)
    }

    async fn run(&mut self, term: &mut Terminal) -> Result<(), ServicesOff> {
        self.init_run(term).await?;

        loop {
            let mut session = term.listen().await?;
            let event = session.event();
            let screen = session.screen();

            match event {
                Some(Event::Key(keys)) if self.valid_size => match keys {
                    KeyEvent {
                        main_key: Key::Esc,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } if self.has_cancel => {
                        self.selected = InputDialogItem::Cancel;
                        break;
                    },

                    KeyEvent {
                        main_key: Key::Left,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => self.key_left(screen),

                    KeyEvent {
                        main_key: Key::Right,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => self.key_right(screen),

                    KeyEvent {
                        main_key: Key::Up,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => self.key_up(screen),

                    KeyEvent {
                        main_key: Key::Down,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => self.key_down(screen),

                    KeyEvent {
                        main_key: Key::Enter,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => break,

                    KeyEvent {
                        main_key: Key::Backspace,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => self.key_backspace(screen),

                    KeyEvent {
                        main_key: Key::Char(ch),
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => self.key_char(screen, ch),

                    _ => (),
                },

                Some(Event::Resize(evt)) => self.resized(evt, screen),

                _ => (),
            }
        }

        Ok(())
    }

    fn result(&mut self) -> TermString {
        let buffer = mem::take(&mut self.buffer);
        tstring![buffer.into_iter().collect::<String>()]
    }

    fn result_with_cancel(&mut self) -> Option<TermString> {
        match self.selected {
            InputDialogItem::Ok => Some(self.result()),
            InputDialogItem::Cancel => None,
        }
    }

    async fn init_run(
        &mut self,
        term: &mut Terminal,
    ) -> Result<(), ServicesOff> {
        let mut session = term.lock_now().await?;
        self.selected = InputDialogItem::Ok;
        self.buffer = self.dialog.buffer.chars().collect::<Vec<_>>();
        self.cursor = 0;
        self.joined = String::new();
        self.update_actual_max(session.screen().size());
        self.render(session.screen());
        Ok(())
    }

    fn update_actual_max(&mut self, screen_size: Coord2) {
        self.actual_max = self.dialog.max.min(screen_size.x);
        let max_index = coord::to_index(self.actual_max).saturating_sub(1);
        self.cursor = self.cursor.min(max_index);
        self.buffer.truncate(coord::to_index(self.actual_max));
    }

    fn key_up(&mut self, screen: &mut Screen) {
        if self.has_cancel {
            self.selected = InputDialogItem::Ok;
            self.render_item(screen, InputDialogItem::Ok);
            self.render_item(screen, InputDialogItem::Cancel);
        }
    }

    fn key_down(&mut self, screen: &mut Screen) {
        if self.has_cancel {
            self.selected = InputDialogItem::Cancel;
            self.render_item(screen, InputDialogItem::Ok);
            self.render_item(screen, InputDialogItem::Cancel);
        }
    }

    fn key_left(&mut self, screen: &mut Screen) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.render_input_box(screen);
        }
    }

    fn key_right(&mut self, screen: &mut Screen) {
        if self.cursor < self.buffer.len() {
            self.cursor += 1;
            self.render_input_box(screen);
        }
    }

    fn key_backspace(&mut self, screen: &mut Screen) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.buffer.remove(self.cursor);
            self.render_input_box(screen);
        }
    }

    fn key_char(&mut self, screen: &mut Screen, ch: char) {
        if (self.dialog.filter)(ch) {
            self.joined.clear();
            self.joined.push('a');
            self.joined.push(ch);
            if self.joined.graphemes(true).count() > 1 {
                self.joined.clear();
                self.joined.extend(self.buffer.iter());
                self.joined.push(ch);
                let length = self.joined.graphemes(true).count() as Coord;
                if length <= self.actual_max {
                    self.buffer.insert(self.cursor, ch);
                    self.cursor += 1;
                    self.render_input_box(screen);
                }
            }
        }
    }

    fn resized(&mut self, evt: ResizeEvent, screen: &mut Screen) {
        self.valid_size = match evt.size {
            Some(size) => {
                self.update_actual_max(size);
                self.render(screen);
                true
            },
            None => false,
        };
    }

    fn render(&self, screen: &mut Screen) {
        screen.clear(self.dialog.bg);
        self.render_title(screen);
        self.render_input_box(screen);
        self.render_item(screen, InputDialogItem::Ok);
        if self.has_cancel {
            self.render_item(screen, InputDialogItem::Cancel);
        }
    }

    fn render_title(&self, screen: &mut Screen) {
        let style = Style::new()
            .left_margin(1)
            .right_margin(1)
            .align(1, 2)
            .max_height(self.dialog.pad_after_title.saturating_add(1))
            .top_margin(self.dialog.title_y);
        screen.styled_text(&self.dialog.title, style);
    }

    fn render_input_box(&self, screen: &mut Screen) {
        let mut field = self.buffer.iter().collect::<String>();
        let additional = self.actual_max as usize - self.buffer.len();
        field.reserve(additional);

        for _ in 0 .. additional {
            field.push_str(" ");
        }

        let style = Style::new()
            .align(1, 2)
            .top_margin(self.y_of_box())
            .colors(self.dialog.box_colors);
        let string = tstring![&field];
        screen.styled_text(&string, style);

        let width = screen.size().x;
        let correction = (self.dialog.max % 2 + width % 2 + 1) as usize;
        let length = field.graphemes(true).count() - correction % 2;

        field.clear();

        for i in 0 .. length + 1 {
            if i == self.cursor {
                field.push('¯')
            } else {
                field.push(' ')
            }
        }

        let style = Style::new()
            .align(1, 2)
            .top_margin(self.y_of_box() + 1)
            .left_margin(1)
            .colors(self.dialog.cursor_colors);
        let string = tstring![&field];
        screen.styled_text(&string, style);
    }

    fn render_item(&self, screen: &mut Screen, item: InputDialogItem) {
        let (option, y) = match item {
            InputDialogItem::Ok => ("> OK <", self.y_of_ok()),
            InputDialogItem::Cancel => ("> CANCEL <", self.y_of_cancel()),
        };
        let colors = if item == self.selected {
            self.dialog.selected_colors
        } else {
            self.dialog.unselected_colors
        };

        let style = Style::new().align(1, 2).colors(colors).top_margin(y);
        let string = tstring![option];
        screen.styled_text(&string, style);
    }

    fn y_of_box(&self) -> Coord {
        self.dialog.title_y + 1 + self.dialog.pad_after_title
    }

    fn y_of_ok(&self) -> Coord {
        self.y_of_box() + 2 + self.dialog.pad_after_box
    }

    fn y_of_cancel(&self) -> Coord {
        self.y_of_ok() + 1 + self.dialog.pad_after_ok
    }
}
