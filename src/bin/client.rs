use crossterm::{
    self,
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use kittui_racer::ui::{
    draw::draw_ui_from_layout,
    models::{App, UiMessage},
    websocket_handler,
};

use std::{
    error::Error,
    io,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    time::Duration,
};

use tui::{
    backend::{Backend, CrosstermBackend},
    Frame, Terminal,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Do an api call and get the quote
    let quote = "How is it possible that a being with such sensitive jewels as the eyes, such enchanted musical instruments as the ears, and such fabulous arabesque of nerves as the brain can experience itself anything less than a god.".to_string();

    // create app and run it
    let app = Arc::new(Mutex::new(App::new(quote)));
    let (sender, receiver) = mpsc::channel::<UiMessage>();

    let app_clone = app.clone();
    tokio::spawn(async move {
        websocket_handler::event_handler(app.clone(), receiver).await;
    });

    let res = run_app(&mut terminal, app_clone, sender);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: Arc<Mutex<App>>,
    sender: Sender<UiMessage>,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(100);

    let mut last_tick = std::time::Instant::now();
    loop {
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| std::time::Duration::from_secs(0));

        let cloned_app = app.clone();
        terminal.draw(move |f| ui(f, cloned_app))?;

        if crossterm::event::poll(timeout).unwrap() {
            if let crossterm::event::Event::Key(key_event) = crossterm::event::read()? {
                if let KeyCode::Esc = key_event.code {
                    return Ok(());
                } else {
                    // Let the handler handle it
                    sender.send(UiMessage::Input(key_event.code)).unwrap();
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = std::time::Instant::now();
        }
    }
}

fn ui<B: Backend>(frame: &mut Frame<B>, app: Arc<Mutex<App>>) {
    let layouts = kittui_racer::ui::layout_divider::divide_frame(frame.size());
    draw_ui_from_layout(app, layouts, frame);
}
