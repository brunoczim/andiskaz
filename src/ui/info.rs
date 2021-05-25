//! An INFO dialong: just shows a message.

use crate::{
    color::{BasicColor, Color, Color2},
    coord::Coord,
    error::ServicesOff,
    event::{Event, Key, KeyEvent},
    screen::Screen,
    string::TermString,
    style::Style,
    terminal::Terminal,
};

/// An info dialog, with just an Ok option.
#[derive(Debug, Clone)]
pub struct InfoDialog {
    /// Title to be shown.
    pub title: TermString,
    /// Long text message to be shown.
    pub message: TermString,
    /// Settings such as margin and alignment.
    pub style: Style<Color2>,
    /// Colors shown with the title.
    pub title_colors: Color2,
    /// Colors shown with the selected option.
    pub selected_colors: Color2,
    /// Position of the title in height.
    pub title_y: Coord,
    /// Color of the background.
    pub bg: Color,
}

impl InfoDialog {
    /// Creates a dialog with default style settings.
    pub fn new(title: TermString, message: TermString) -> Self {
        Self {
            title,
            message,
            style: Style::new()
                .align(1, 2)
                .colors(Color2::default())
                .top_margin(4)
                .bottom_margin(2),
            title_colors: Color2::default(),
            selected_colors: !Color2::default(),
            title_y: 1,
            bg: BasicColor::Black.into(),
        }
    }

    /// Runs this dialog showing it to the user, awaiting OK!
    pub async fn run(&self, term: &mut Terminal) -> Result<(), ServicesOff> {
        self.render(term.lock_now().await?.screen());

        loop {
            let mut session = term.listen().await?;
            match session.event() {
                Some(Event::Key(KeyEvent {
                    main_key: Key::Enter,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }))
                | Some(Event::Key(KeyEvent {
                    main_key: Key::Esc,
                    ctrl: false,
                    alt: false,
                    shift: false,
                })) => break Ok(()),

                Some(Event::Resize(_)) => self.render(session.screen()),

                _ => (),
            }
        }
    }

    /// Renders the whole dialog.
    fn render(&self, screen: &mut Screen) {
        screen.clear(self.bg);
        self.render_title(screen);
        let pos = self.render_message(screen);
        self.render_ok(screen, pos);
    }

    /// Renders the title of the dialog.
    fn render_title(&self, screen: &mut Screen) {
        let style = Style::new()
            .align(1, 2)
            .colors(self.title_colors)
            .top_margin(self.title_y);
        screen.styled_text(&self.title, style);
    }

    /// Renders the message of the dialog.
    fn render_message(&self, screen: &mut Screen) -> Coord {
        screen.styled_text(&self.message, self.style)
    }

    /// Renders the OK button.
    fn render_ok(&self, screen: &mut Screen, pos: Coord) {
        let ok_string = tstring!["> OK <"];
        let style = Style::new()
            .align(1, 2)
            .colors(self.selected_colors)
            .top_margin(pos + 2);
        screen.styled_text(&ok_string, style);
    }
}
