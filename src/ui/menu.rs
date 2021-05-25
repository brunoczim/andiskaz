//! This module exports items related to menus in the UI, such as more extensive
//! menus, or just dialogs for OK/CANCEL.

use crate::{
    color::{BasicColor, Color, Color2},
    coord::{Coord, Coord2},
    error::ServicesOff,
    event::{Event, Key, KeyEvent, ResizeEvent},
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
        let mut selector = Selector::without_cancel(self, initial);
        selector.run(term).await?;
        Ok(selector.result())
    }

    /// Asks for the user to select an item of the menu with a cancel option.
    pub async fn select_with_cancel(
        &self,
        term: &mut Terminal,
    ) -> Result<Option<usize>, ServicesOff> {
        self.select_cancel_initial(term, 0, false).await
    }

    /// Asks for the user to select an item of the menu with a cancel option,
    /// and sets the initial chosen option to the given one, together with a
    /// paramter stating whether cancel option is currently chosen.
    pub async fn select_cancel_initial(
        &self,
        term: &mut Terminal,
        initial: usize,
        cancel: bool,
    ) -> Result<Option<usize>, ServicesOff> {
        let mut selector = Selector::with_cancel(self, initial, cancel);
        selector.run(term).await?;
        Ok(selector.result_with_cancel())
    }
}

/// A trait representing a menu option.
pub trait MenuOption {
    /// Returns the display name of this option.
    fn name(&self) -> TermString;
}

impl MenuOption for TermString {
    fn name(&self) -> TermString {
        self.clone()
    }
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

/// Menu selection runner.
#[derive(Debug)]
struct Selector<'menu, O>
where
    O: MenuOption,
{
    /// A reference to the original menu.
    menu: &'menu Menu<O>,
    /// First row currently shown.
    first_row: usize,
    /// Last row currently shown.
    last_row: usize,
    /// Row currently selected (or that was previously selected before the
    /// cancel being currently selected).
    selected: usize,
    /// Whether the cancel option is currently selected, IF cancel is `Some`.
    cancel: Option<bool>,
}

impl<'menu, O> Selector<'menu, O>
where
    O: MenuOption,
{
    /// Initializes this selector for a selection without cancel option.
    fn without_cancel(menu: &'menu Menu<O>, initial: usize) -> Self {
        Selector {
            menu,
            selected: initial,
            cancel: None,
            first_row: 0,
            last_row: 0,
        }
    }

    /// Initializes this selector for a selection with cancel option.
    fn with_cancel(menu: &'menu Menu<O>, initial: usize, cancel: bool) -> Self {
        Selector {
            menu,
            selected: initial,
            cancel: Some(cancel || menu.options.len() == 0),
            first_row: 0,
            last_row: 0,
        }
    }

    /// Gets the result for a selection without cancel.
    fn result(&self) -> usize {
        self.selected
    }

    /// Gets the result for a selection with cancel option.
    fn result_with_cancel(&self) -> Option<usize> {
        Some(self.selected).filter(|_| self.cancel != Some(true))
    }

    /// Runs this selector and uses the given terminal.
    async fn run(&mut self, term: &mut Terminal) -> Result<(), ServicesOff> {
        self.init_run(term).await?;

        loop {
            let mut session = term.listen().await?;
            let event = session.event();
            let screen = session.screen();

            match event {
                Some(Event::Key(KeyEvent {
                    main_key: Key::Esc,
                    ctrl: false,
                    alt: false,
                    shift: false,
                })) => break,

                Some(Event::Key(KeyEvent {
                    main_key: Key::Up,
                    ctrl: false,
                    alt: false,
                    shift: false,
                })) => self.key_up(screen),

                Some(Event::Key(KeyEvent {
                    main_key: Key::Down,
                    ctrl: false,
                    alt: false,
                    shift: false,
                })) => self.key_down(screen),

                Some(Event::Key(KeyEvent {
                    main_key: Key::Left,
                    ctrl: false,
                    alt: false,
                    shift: false,
                })) => self.key_left(screen),

                Some(Event::Key(KeyEvent {
                    main_key: Key::Right,
                    ctrl: false,
                    alt: false,
                    shift: false,
                })) => self.key_right(screen),

                Some(Event::Key(KeyEvent {
                    main_key: Key::Enter,
                    ctrl: false,
                    alt: false,
                    shift: false,
                })) => break,

                Some(Event::Resize(evt)) => self.resized(evt, screen),

                _ => (),
            }
        }

        Ok(())
    }

    /// Initializes the run of this selector.
    async fn init_run(
        &mut self,
        term: &mut Terminal,
    ) -> Result<(), ServicesOff> {
        let mut session = term.lock_now().await?;
        let screen = session.screen();
        self.render(screen);
        self.update_last_row(session.screen().size());
        Ok(())
    }

    /// Should be triggered when UP key is pressed.
    fn key_up(&mut self, screen: &mut Screen) {
        if self.is_cancelling() && self.menu.options.len() > 0 {
            self.cancel = Some(false);
            self.render(screen);
        } else if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.first_row {
                self.first_row -= 1;
                self.update_last_row(screen.size());
            }
            self.render(screen);
        }
    }

    /// Should be triggered when DOWN key is pressed.
    fn key_down(&mut self, screen: &mut Screen) {
        if self.selected + 1 < self.menu.options.len() {
            self.selected += 1;
            if self.selected >= self.last_row {
                self.first_row += 1;
                self.update_last_row(screen.size());
            }
            self.render(screen);
        } else if self.is_not_cancelling() {
            self.cancel = Some(true);
            self.render(screen);
        }
    }

    /// Should be triggered when LEFT key is pressed.
    fn key_left(&mut self, screen: &mut Screen) {
        if self.is_not_cancelling() {
            self.cancel = Some(true);
            self.render(screen);
        }
    }

    /// Should be triggered when RIGHT key is pressed.
    fn key_right(&mut self, screen: &mut Screen) {
        if self.is_cancelling() && self.menu.options.len() > 0 {
            self.cancel = Some(false);
            self.render(screen);
        }
    }

    /// Should be triggered when screen is resized.
    fn resized(&mut self, evt: ResizeEvent, screen: &mut Screen) {
        if let Some(size) = evt.size {
            self.render(screen);
            self.update_last_row(size);
        }
    }

    /// Returns if the selection is currently selecting the cancel option.
    fn is_cancelling(&self) -> bool {
        self.cancel == Some(true)
    }

    /// Returns if the selection is currently not selecting the cancel option
    /// AND the cancel option is enabled.
    fn is_not_cancelling(&self) -> bool {
        self.cancel == Some(false)
    }

    /// Updates the last row field from the computed end of the screen.
    fn update_last_row(&mut self, screen_size: Coord2) {
        self.last_row = self.screen_end(screen_size);
    }

    /// Returns the index of the last visible option in the screen.
    fn screen_end(&self, screen_size: Coord2) -> usize {
        let cancel = if self.cancel.is_some() { 4 } else { 0 };
        let mut available = screen_size.y - self.menu.title_y;
        available -= 2 * (self.menu.pad_after_title - 1) + cancel;
        let extra = available / (self.menu.pad_after_option + 1) - 2;
        self.first_row + extra as usize
    }

    /// Returns the range of the visible options in the screen.
    fn range_of_screen(&self, screen_size: Coord2) -> Range<usize> {
        self.first_row .. self.screen_end(screen_size)
    }

    /// Renders the whole menu.
    fn render(&self, screen: &mut Screen) {
        screen.clear(self.menu.bg);
        self.render_title(screen);

        let arrow_style =
            Style::new().align(1, 2).colors(self.menu.arrow_colors);

        let mut range = self.range_of_screen(screen.size());
        self.render_up_arrow(screen, arrow_style);
        self.render_down_arrow(screen, arrow_style, &mut range);

        self.render_options(screen, range);
        self.render_cancel(screen, screen.size().y);
    }

    /// Renders the title of the menu.
    fn render_title(&self, screen: &mut Screen) {
        let title_style = Style::new()
            .align(1, 2)
            .top_margin(self.menu.title_y)
            .colors(self.menu.title_colors)
            .max_height(self.menu.pad_after_title.saturating_add(1));
        screen.styled_text(&self.menu.title, title_style);
    }

    /// Renders the UP arrow.
    fn render_up_arrow(&self, screen: &mut Screen, style: Style<Color2>) {
        if self.first_row > 0 {
            let mut option_y = self.y_of_option(self.first_row);
            option_y -= self.menu.pad_after_option + 1;
            let style = style.top_margin(option_y);
            screen.styled_text(&tstring!["Ʌ"], style);
        }
    }

    /// Renders the DOWN arrow and updates the given range of the screen.
    fn render_down_arrow(
        &self,
        screen: &mut Screen,
        style: Style<Color2>,
        range: &mut Range<usize>,
    ) {
        if range.end < self.menu.options.len() {
            let option_y = self.y_of_option(range.end);
            let style = style.top_margin(option_y);
            screen.styled_text(&tstring!["V"], style);
        } else {
            range.end = self.menu.options.len();
        }
    }

    /// Renders all the options of the given range.
    fn render_options(&self, screen: &mut Screen, range: Range<usize>) {
        for (i, option) in self.menu.options[range.clone()].iter().enumerate() {
            let is_selected =
                range.start + i == self.selected && !self.is_cancelling();
            self.render_option(
                screen,
                option,
                self.y_of_option(range.start + i),
                is_selected,
            );
        }
    }

    /// Renders a single option.
    fn render_option(
        &self,
        screen: &mut Screen,
        option: &O,
        option_y: Coord,
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
        }

        buf = tstring_concat![tstring!["> "], buf, tstring![" <"]];

        let colors = if selected {
            self.menu.selected_colors
        } else {
            self.menu.unselected_colors
        };
        let style =
            Style::new().align(1, 2).colors(colors).top_margin(option_y);
        screen.styled_text(&buf, style);
    }

    /// Renders the cancel option, if any.
    fn render_cancel(&self, screen: &mut Screen, cancel_y: Coord) {
        if let Some(selected) = self.cancel {
            let colors = if selected {
                self.menu.selected_colors
            } else {
                self.menu.unselected_colors
            };
            let string = tstring!["> Cancel <"];

            let style = Style::new()
                .align(1, 3)
                .colors(colors)
                .top_margin(cancel_y - 2);
            screen.styled_text(&string, style);
        }
    }

    /// Gets the height of a given option (by index).
    fn y_of_option(&self, option: usize) -> Coord {
        let count = (option - self.first_row) as Coord;
        let before = (count + 1) * (self.menu.pad_after_option + 1);
        before + self.menu.pad_after_title + 1 + self.menu.title_y
    }
}
