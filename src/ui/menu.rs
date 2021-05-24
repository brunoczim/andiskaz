use crate::{
    color::{BasicColor, Color, Color2},
    coord::{Coord, Coord2},
    error::ServicesOff,
    event::{Event, Key, KeyEvent},
    screen::Screen,
    string::{TermGrapheme, TermString},
    style::Style,
    terminal::Terminal,
};
use std::ops::Range;

/// A menu, with a list of options and potentially a cancel option.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Menu<O>
where
    O: MenuOption,
{
    /// The title shown above the menu.
    pub title: TermString,
    /// A list of options.
    pub options: Vec<O>,
    /// Colors for the title.
    pub title_colors: Color2,
    /// Colors for the arrows.
    pub arrow_colors: Color2,
    /// Colors for selected options.
    pub selected_colors: Color2,
    /// Colors for unselected options.
    pub unselected_colors: Color2,
    /// Color of the background of no text.
    pub bg: Color,
    /// Number of lines padded before the title.
    pub title_y: Coord,
    /// Number of lines padded after the title.
    pub pad_after_title: Coord,
    /// Number of lines padded after an option.
    pub pad_after_option: Coord,
}

impl<O> Menu<O>
where
    O: MenuOption,
{
    /// Creates a new menu with default styles.
    pub fn new(title: TermString, options: Vec<O>) -> Self {
        Self {
            title,
            options,
            title_colors: Color2::default(),
            arrow_colors: Color2::default(),
            selected_colors: !Color2::default(),
            unselected_colors: Color2::default(),
            bg: BasicColor::Black.into(),
            title_y: 1,
            pad_after_title: 2,
            pad_after_option: 1,
        }
    }

    /// Asks for the user to select an item of the menu without cancel option.
    pub async fn select(
        &self,
        term: &mut Terminal,
    ) -> Result<usize, ServicesOff> {
        self.select_with_initial(term, 0).await
    }

    /// Asks for the user to select an item of the menu without cancel option,
    /// but with a given initial chosen option.
    pub async fn select_with_initial(
        &self,
        term: &mut Terminal,
        initial: usize,
    ) -> Result<usize, ServicesOff> {
        let mut selected = initial;
        let mut start = 0;

        let mut last_row = {
            let mut session = term.lock_now().await?;
            self.render(session.screen(), start, Some(selected), false);
            self.screen_end(start, session.screen().size(), false)
        };

        loop {
            let mut session = term.listen().await?;

            match session.event() {
                Some(Event::Key(KeyEvent {
                    main_key: Key::Up,
                    alt: false,
                    ctrl: false,
                    shift: false,
                })) => {
                    if selected > 0 {
                        selected -= 1;
                        if selected < start {
                            start -= 1;
                            last_row = self.screen_end(
                                start,
                                session.screen().size(),
                                false,
                            );
                        }
                        self.render(
                            session.screen(),
                            start,
                            Some(selected),
                            false,
                        );
                    }
                },

                Some(Event::Key(KeyEvent {
                    main_key: Key::Down,
                    alt: false,
                    ctrl: false,
                    shift: false,
                })) => {
                    if selected + 1 < self.options.len() {
                        selected += 1;
                        if selected >= last_row {
                            start += 1;
                            last_row = self.screen_end(
                                start,
                                session.screen().size(),
                                false,
                            );
                        }
                        self.render(
                            session.screen(),
                            start,
                            Some(selected),
                            false,
                        );
                    }
                },

                Some(Event::Key(KeyEvent {
                    main_key: Key::Enter,
                    alt: false,
                    ctrl: false,
                    shift: false,
                })) => break,

                Some(Event::Resize(evt)) => {
                    if let Some(size) = evt.size {
                        self.render(
                            session.screen(),
                            start,
                            Some(selected),
                            false,
                        );
                        last_row = self.screen_end(start, size, false);
                    }
                },

                _ => (),
            }
        }

        Ok(selected)
    }

    /// Asks for the user to select an item of the menu with a cancel option.
    pub async fn select_with_cancel(
        &self,
        term: &mut Terminal,
    ) -> Result<Option<usize>, ServicesOff> {
        self.select_with_cancel_and_initial(term, Some(0)).await
    }

    /// Asks for the user to select an item of the menu with a cancel option,
    /// and sets the initial chosen option to the given one.
    pub async fn select_with_cancel_and_initial(
        &self,
        term: &mut Terminal,
        initial: Option<usize>,
    ) -> Result<Option<usize>, ServicesOff> {
        let mut selected = initial.unwrap_or(0);
        let mut is_cancel = initial.is_none();
        let mut start = 0;

        let mut last_row = {
            let mut session = term.lock_now().await?;
            self.render(
                session.screen(),
                start,
                Some(selected).filter(|_| !is_cancel),
                true,
            );
            self.screen_end(start, session.screen().size(), true)
        };

        let ret = loop {
            let mut session = term.listen().await?;

            match session.event() {
                Some(Event::Key(KeyEvent {
                    main_key: Key::Esc,
                    ctrl: false,
                    alt: false,
                    shift: false,
                })) => break None,

                Some(Event::Key(KeyEvent {
                    main_key: Key::Up,
                    ctrl: false,
                    alt: false,
                    shift: false,
                })) => {
                    if is_cancel && self.options.len() > 0 {
                        is_cancel = false;
                        self.render(
                            session.screen(),
                            start,
                            Some(selected),
                            true,
                        );
                    } else if selected > 0 {
                        selected -= 1;
                        if selected < start {
                            start -= 1;
                            last_row = self.screen_end(
                                start,
                                session.screen().size(),
                                true,
                            );
                        }
                        self.render(
                            session.screen(),
                            start,
                            Some(selected).filter(|_| !is_cancel),
                            true,
                        );
                    }
                },

                Some(Event::Key(KeyEvent {
                    main_key: Key::Down,
                    ctrl: false,
                    alt: false,
                    shift: false,
                })) => {
                    if selected + 1 < self.options.len() {
                        selected += 1;
                        if selected >= last_row {
                            start += 1;
                            last_row = self.screen_end(
                                start,
                                session.screen().size(),
                                true,
                            );
                        }
                        self.render(
                            session.screen(),
                            start,
                            Some(selected).filter(|_| !is_cancel),
                            true,
                        );
                    } else if !is_cancel {
                        is_cancel = true;
                        self.render(session.screen(), start, None, true);
                    }
                },

                Some(Event::Key(KeyEvent {
                    main_key: Key::Left,
                    ctrl: false,
                    alt: false,
                    shift: false,
                })) => {
                    if !is_cancel {
                        is_cancel = true;
                        self.render(session.screen(), start, None, true);
                    }
                },

                Some(Event::Key(KeyEvent {
                    main_key: Key::Right,
                    ctrl: false,
                    alt: false,
                    shift: false,
                })) => {
                    if is_cancel && self.options.len() > 0 {
                        is_cancel = false;
                        self.render(
                            session.screen(),
                            start,
                            Some(selected),
                            true,
                        );
                    }
                },

                Some(Event::Key(KeyEvent {
                    main_key: Key::Enter,
                    ctrl: false,
                    alt: false,
                    shift: false,
                })) => break if is_cancel { None } else { Some(selected) },

                Some(Event::Resize(evt)) => {
                    if let Some(size) = evt.size {
                        self.render(
                            session.screen(),
                            start,
                            Some(selected),
                            true,
                        );
                        last_row = self.screen_end(start, size, true);
                    }
                },

                _ => (),
            }
        };

        Ok(ret)
    }

    fn y_of_option(&self, start: usize, option: usize) -> Coord {
        let count = (option - start) as Coord;
        let before = (count + 1) * (self.pad_after_option + 1);
        before + self.pad_after_title + 1 + self.title_y
    }

    fn screen_end(
        &self,
        start: usize,
        screen_size: Coord2,
        cancel: bool,
    ) -> usize {
        let cancel = if cancel { 4 } else { 0 };
        let available = screen_size.y - self.title_y;
        let available = available - 2 * (self.pad_after_title - 1) - cancel;
        let extra = available / (self.pad_after_option + 1) - 2;
        start + extra as usize
    }

    fn range_of_screen(
        &self,
        start: usize,
        screen_size: Coord2,
        cancel: bool,
    ) -> Range<usize> {
        start .. self.screen_end(start, screen_size, cancel)
    }

    fn render(
        &self,
        screen: &mut Screen,
        start: usize,
        selected: Option<usize>,
        cancel: bool,
    ) {
        screen.clear(self.bg);
        let style = Style::new()
            .align(1, 2)
            .top_margin(self.title_y)
            .colors(self.title_colors)
            .max_height(self.pad_after_title.saturating_add(1));
        screen.styled_text(&self.title, style);

        let mut range = self.range_of_screen(start, screen.size(), cancel);
        if start > 0 {
            let y = self.y_of_option(start, start) - self.pad_after_option - 1;
            let style = Style::new()
                .align(1, 2)
                .colors(self.arrow_colors)
                .top_margin(y);
            screen.styled_text(&tstring!["Ʌ"], style);
        }
        if range.end < self.options.len() {
            let y = self.y_of_option(start, range.end);
            let style = Style::new()
                .align(1, 2)
                .colors(self.arrow_colors)
                .top_margin(y);
            screen.styled_text(&tstring!["V"], style);
        } else {
            range.end = self.options.len();
        }
        for (i, option) in self.options[range.clone()].iter().enumerate() {
            let is_selected = Some(range.start + i) == selected;
            self.render_option(
                screen,
                option,
                self.y_of_option(start, range.start + i),
                is_selected,
            );
        }

        if cancel {
            self.render_cancel(screen, screen.size().y, selected.is_none());
        }
    }

    fn render_option(
        &self,
        screen: &mut Screen,
        option: &O,
        y: Coord,
        selected: bool,
    ) {
        let mut buf = option.name();
        let mut len = buf.count_graphemes();
        let screen_size = screen.size();

        if len as Coord % 2 != screen_size.x % 2 {
            buf = tstring_concat![buf, TermGrapheme::space()];
            len += 1;
        }

        if screen_size.x - 4 < len as Coord {
            buf = tstring_concat![
                buf.index(.. len - 5),
                TermGrapheme::new_lossy("…")
            ];
            #[allow(unused_assignments)]
            {
                len -= 4;
            }
        }

        buf = tstring_concat![tstring!["> "], buf, tstring![" <"]];

        let colors = if selected {
            self.selected_colors
        } else {
            self.unselected_colors
        };
        let style = Style::new().align(1, 2).colors(colors).top_margin(y);
        screen.styled_text(&buf, style);
    }

    fn render_cancel(
        &self,
        screen: &mut Screen,
        cancel_y: Coord,
        selected: bool,
    ) {
        let colors = if selected {
            self.selected_colors
        } else {
            self.unselected_colors
        };
        let string = tstring!["> Cancel <"];

        let style =
            Style::new().align(1, 3).colors(colors).top_margin(cancel_y - 2);
        screen.styled_text(&string, style);
    }
}

/// A trait representing a menu option.
pub trait MenuOption {
    /// Returns the display name of this option.
    fn name(&self) -> TermString;
}

/// An item of a prompt about a dangerous action.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DangerPromptOption {
    /// Returned when user cancels this action.
    Cancel,
    /// Returned when user confirms this action.
    Ok,
}

impl DangerPromptOption {
    /// Creates a menu over a dangerous prompt.
    pub fn menu(title: TermString) -> Menu<Self> {
        Menu::new(
            title,
            vec![DangerPromptOption::Ok, DangerPromptOption::Cancel],
        )
    }
}

impl MenuOption for DangerPromptOption {
    fn name(&self) -> TermString {
        let string = match self {
            DangerPromptOption::Cancel => "CANCEL",
            DangerPromptOption::Ok => "OK",
        };

        tstring![string]
    }
}
