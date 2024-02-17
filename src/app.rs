use std::io;

use crate::nord::{self, City, Country, Nord, Status};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use rand::Rng;
use ratatui::layout::Layout;
use ratatui::{prelude::*, widgets::*};

enum Pane {
    Country,
    City,
}

impl Pane {
    fn first() -> Self {
        Pane::Country
    }

    fn next(&self) -> Self {
        use Pane::*;
        match *self {
            Country => City,
            City => Country,
        }
    }

    fn prev(&self) -> Self {
        use Pane::*;
        match *self {
            Country => City,
            City => Country,
        }
    }
}

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

pub struct App {
    status: Status,
    pane: Pane,
    countries: StatefulList<Country>,
    cities: StatefulList<City>,
    ui: Ui,
    help: bool,
}

impl App {
    pub fn new<'a>(nord: Nord) -> App {
        return App {
            status: nord.status,
            pane: Pane::first(),
            countries: StatefulList::with_items(nord.countries),
            cities: StatefulList::with_items(vec![]),
            ui: Ui::default(),
            help: false,
        };
    }
}

impl App {
    pub fn run(&mut self, mut terminal: Terminal<impl Backend>) -> io::Result<()> {
        self.set_ui(Ui::init(terminal.size().expect("Terminal error")));
        loop {
            self.draw(&mut terminal)?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    use KeyCode::*;
                    match key.code {
                        Char('q') | Esc => {
                            if self.help {
                                self.help = !self.help
                            } else {
                                return Ok(());
                            }
                        }
                        Char('l') | Right | Tab => self.next_pane(),
                        Char('h') | Left => self.prev_pane(),
                        Char('r') => self.reload_status(),
                        Char('R') => self.connect_random(),
                        Char('c') | Enter => self.connect_selected(),
                        Char('d') => self.disconnect(),
                        Char('?') => self.help = !self.help,
                        _ => {}
                    }

                    match self.pane {
                        Pane::Country => match key.code {
                            Char('j') | Down => {
                                self.countries.next();
                                self.reload_cities();
                            }
                            Char('k') | Up => {
                                self.countries.previous();
                                self.reload_cities();
                            }
                            Char('g') => {
                                self.countries.first();
                                self.reload_cities();
                            }
                            Char('G') => self.countries.last(),
                            _ => {}
                        },
                        Pane::City => match key.code {
                            Char('j') | Down => {
                                self.cities.next();
                            }
                            Char('k') | Up => {
                                self.cities.previous();
                            }
                            Char('g') => self.countries.first(),
                            Char('G') => self.countries.last(),
                            _ => {}
                        },
                    };
                }
            }
        }
    }

    fn set_ui(&mut self, ui: Ui) {
        self.ui = ui;
    }

    fn reload_status(&mut self) {
        self.status = nord::get_status();
    }

    fn reload_cities(&mut self) {
        let idx = match self.countries.state.selected() {
            Some(i) => i,
            None => 0,
        };
        let country = self.countries.items.get(idx).expect("");

        self.cities = StatefulList::with_items(country.cities.clone());
    }

    fn connect_selected(&mut self) {
        match self.pane {
            Pane::Country => nord::connect(
                &self
                    .countries
                    .items
                    .get(match self.countries.state.selected() {
                        Some(i) => i,
                        None => 0,
                    })
                    .expect("")
                    .name,
            ),
            Pane::City => nord::connect(
                &self
                    .cities
                    .items
                    .get(match self.cities.state.selected() {
                        Some(i) => i,
                        None => 0,
                    })
                    .expect("")
                    .name,
            ),
        }

        self.reload_status();
    }

    fn connect_random(&mut self) {
        let random = rand::thread_rng().gen_range(0..self.countries.items.len());
        nord::connect(&self.countries.items.get(random).expect("").name);
        self.reload_status();
    }

    fn disconnect(&mut self) {
        nord::disconnect();
        self.reload_status();
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> io::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }

    fn next_pane(&mut self) {
        self.pane = self.pane.next();
    }

    fn prev_pane(&mut self) {
        self.pane = self.pane.prev();
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut ui: Ui = Ui::init(area);

        if self.help {
            self.render_help(area, buf);
        } else {
            self.render_title(ui.get_title(), buf);
            self.render_status(ui.get_header(), buf);
            self.render_countries(ui.get_country(), buf);
            self.render_cities(ui.get_city(), buf);
            self.render_footer(ui.get_footer(), buf);
        }
    }
}

#[derive(Default)]
struct Ui {
    area: Rect,
    main_frame: Layout,
    body: Layout,
    body_frame: Layout,
    details: Layout,
}

impl Ui {
    fn init(area: Rect) -> Ui {
        return Ui {
            area,
            main_frame: Layout::vertical([
                Constraint::Length(2),
                Constraint::Min(0),
                Constraint::Length(2),
            ]),
            body_frame: Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(60),
                Constraint::Fill(1),
            ]),
            body: Layout::vertical([Constraint::Max(12), Constraint::Max(25)]),
            details: Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]),
        };
    }

    fn get_title(&mut self) -> Rect {
        let [title_area, _body_area, _footer_area] = self.main_frame.areas(self.area);
        return title_area;
    }

    fn get_header(&mut self) -> Rect {
        let [_title_area, body_area, _footer_area] = self.main_frame.areas(self.area);
        let [_left_filler, body_area, _right_filler] = self.body_frame.areas(body_area);
        let [header_area, _details_area] = self.body.areas(body_area);
        return header_area;
    }

    fn get_footer(&mut self) -> Rect {
        let [_title_area, _body_area, footer_area] = self.main_frame.areas(self.area);
        return footer_area;
    }

    fn get_country(&mut self) -> Rect {
        let [_title_area, body_area, _footer_area] = self.main_frame.areas(self.area);
        let [_left_filler, body_area, _right_filler] = self.body_frame.areas(body_area);
        let [_header_area, details_area] = self.body.areas(body_area);
        let [country_column, _city_column] = self.details.areas(details_area);
        return country_column;
    }

    fn get_city(&mut self) -> Rect {
        let [_title_area, body_area, _footer_area] = self.main_frame.areas(self.area);
        let [_left_filler, body_area, _right_filler] = self.body_frame.areas(body_area);
        let [_header_area, details_area] = self.body.areas(body_area);
        let [_country_column, city_column] = self.details.areas(details_area);
        return city_column;
    }
}

impl App {
    fn render_help(&self, area: Rect, buf: &mut Buffer) {
        let help_text = "
    Navigation:
        ↓/↑ - Select list item
        ←/→ - Move between panels
        g   - Move to the top of current panel
        G   - Move to the bottom of current panel

    Functionality:
        c   - Connect to selected option
        d   - Disconnect
        r   - Refresh
        R   - Connect to random option

        ?   - Help
        q   - Quit
            ";
        let block = Block::default().title("Help").borders(Borders::ALL);
        let para = Paragraph::new(help_text).block(block).bg(Color::default());
        let area = popup(55, 18, area);
        para.render(area, buf);
    }

    fn render_title(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Ratatui List Example")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("\nUse ↓↑ to move, ← → to change panel, g/G to go top/bottom, ? for help")
            .centered()
            .render(area, buf);
    }

    fn render_status(&mut self, area: Rect, buf: &mut Buffer) {
        let sub_v_layout = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]);
        let column_split =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let column_layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ]);

        let sub_v_area = sub_v_layout.split(area);
        let columns = column_split.split(sub_v_area[1]);
        let left_col = column_layout.split(columns[0]);
        let right_col = column_layout.split(columns[1]);

        let fg = if &self.status.status == "Connected" {
            Color::Green
        } else {
            Color::Red
        };

        Paragraph::new(String::from(&self.status.status))
            .block(Self::get_block("Status"))
            .alignment(Alignment::Center)
            .fg(fg)
            .render(sub_v_area[0], buf);
        Paragraph::new(String::from(&self.status.country))
            .block(Self::get_block("Country"))
            .render(left_col[0], buf);
        Paragraph::new(String::from(&self.status.city))
            .block(Self::get_block("City"))
            .render(left_col[1], buf);
        Paragraph::new(String::from(&self.status.uptime))
            .block(Self::get_block("Uptime"))
            .render(left_col[2], buf);
        Paragraph::new(String::from(&self.status.ip))
            .block(Self::get_block("IP"))
            .render(right_col[0], buf);
        Paragraph::new(String::from(&self.status.transfer.down))
            .block(Self::get_block("Download"))
            .render(right_col[1], buf);
        Paragraph::new(String::from(&self.status.transfer.up))
            .block(Self::get_block("Upload"))
            .render(right_col[2], buf);
    }

    fn get_block(title: &str) -> Block<'static> {
        Block::default()
            .borders(Borders::all())
            .title(String::from(title))
            .title_alignment(Alignment::Center)
    }

    fn render_countries(&mut self, area: Rect, buf: &mut Buffer) {
        let fg = match self.pane {
            Pane::Country => Color::LightCyan,
            _ => Color::default(),
        };

        let outer_block = Block::default()
            .borders(Borders::NONE)
            .title("Countries")
            .style(Style::default().fg(fg))
            .title_alignment(Alignment::Center);
        let inner_block = Block::default()
            .borders(Borders::all())
            .style(Style::default().remove_modifier(Modifier::BOLD));

        let outer_area = area;
        let inner_area = outer_block.inner(outer_area);

        outer_block.render(outer_area, buf);

        let items: Vec<ListItem> = self
            .countries
            .items
            .iter()
            .enumerate()
            .map(|(_i, country)| country_to_list_item(country))
            .collect();

        let items = List::new(items)
            .block(inner_block)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED),
            )
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(items, inner_area, buf, &mut self.countries.state);
    }

    fn render_cities(&mut self, area: Rect, buf: &mut Buffer) {
        let fg = match self.pane {
            Pane::City => Color::LightCyan,
            _ => Color::default(),
        };

        let outer_block = Block::default()
            .borders(Borders::NONE)
            .title("Cities")
            .style(Style::default().fg(fg))
            .title_alignment(Alignment::Center);
        let inner_block = Block::default().borders(Borders::all());

        let outer_area = area;
        let inner_area = outer_block.inner(outer_area);

        outer_block.render(outer_area, buf);
        let items: Vec<ListItem> = self
            .cities
            .items
            .iter()
            .enumerate()
            .map(|(_i, city)| city_to_list_item(city))
            .collect();

        let items = List::new(items)
            .block(inner_block)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED),
            )
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(items, inner_area, buf, &mut self.cities.state);
    }
}

fn city_to_list_item(city: &City) -> ListItem<'_> {
    return ListItem::new(String::from(&city.name));
}

fn country_to_list_item(country: &Country) -> ListItem<'_> {
    return ListItem::new(String::from(&country.name));
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        if self.items.len() > 0 {
            let i = match self.state.selected() {
                Some(i) => {
                    if i >= self.items.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    pub fn previous(&mut self) {
        if self.items.len() > 0 {
            let i = match self.state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.items.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    pub fn first(&mut self) {
        self.state.select(Some(0));
    }

    pub fn last(&mut self) {
        self.state.select(Some(self.items.len() - 1));
    }
}

fn popup(x: u16, y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(y),
        Constraint::Fill(1),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(x),
        Constraint::Fill(1),
    ])
    .split(popup_layout[1])[1]
}
