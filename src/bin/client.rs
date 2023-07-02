/// This is the Terminal User Interface for a player (kitten ) to practice / play with other kittens
///
/// Every player is a kitten
/// The Goal of a kitten is to sharpen his paws to fight with other kitties
///
/// Each client tries to connect to a server ( master Cat ) for communication
/// The communication channel is web sockets
///
/// If the connection is successful, then game areana can be entered
/// If connection fails, only practice section will be available
///
/// The Clients are always in sync about all the players currently online
/// and available to play
///
/// To enter a game, a kitten has to challenge the other kitten
/// Once the opponent kitten accepts the challenge, game begins
///
///
use crossterm::{
    self,
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use kittui_racer::ui::{
    draw::draw_ui_from_layout,
    input_handler,
    types::{App, UiMessage},
    websocket_handler,
};

use std::{
    error::Error,
    io,
    sync::{Arc, Mutex},
    time::Duration,
};

use tui::{
    backend::{Backend, CrosstermBackend},
    Frame, Terminal,
};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // A channel to communicate events to websocket handler.
    // Few events like challenging a player, progress update of a game
    // has to be relayed to all users who are currently online
    // these events are communicated by the application via this channel to webwocket handler
    let (sender, receiver) = tokio::sync::mpsc::channel::<UiMessage>(32);
    let app = Arc::new(Mutex::new(App::new(sender)));

    let app_clone = app.clone();

    // Handle the websocket events in a separate thread
    std::thread::spawn(move || {
        let single_threaded_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .unwrap();

        // The created runtime is run on the current thread
        single_threaded_runtime.block_on(websocket_handler::event_handler(app_clone, receiver))
    });

    let res = run_app(&mut terminal, app);

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: Arc<Mutex<App>>) -> io::Result<()> {
    let tick_rate = Duration::from_millis(100);

    let mut last_tick = std::time::Instant::now();
    loop {
        // If there is some waiting time from the last iteration
        // Let's say the last iteration took only 20ms, for the current iteration
        // the poll timeout will be 80ms
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| std::time::Duration::from_secs(0));

        let cloned_app = app.clone();

        // The terminal is drawn either
        // - after `timeout` duration, if there is no input
        // - if any key is pressed
        terminal.draw(move |f| ui(f, cloned_app))?;

        // Why use poll?
        // Read is a blocking call, the current thread is blocked untill the event is available
        // When it is blocked, the thread is taken off the CPU and some other thread can be executed
        //
        // The poll function will poll for the given time duration and
        // if no input is available, then it will return Ok(false)
        //
        // So when is this useful?
        //
        // Let's say we want to execute some code after waiting for input
        // with `read()`, the function would not return ( it would be blocked ) if there is no input,
        // so we cannot proceed further without input
        // In our case, we want the `terminal.draw` to be executed even if there was no input
        // as the contents of terminal might be updated by websocket events.
        //
        // But with `poll(x)`, we wait for x amount of time for input
        // if no input is read from the input stream after x time, the `poll()` function will return Ok(false)
        if crossterm::event::poll(timeout).unwrap() {
            // If `poll()` returns Ok(false), this `read()` call is non blocking
            if let crossterm::event::Event::Key(key_event) = crossterm::event::read()? {
                if let KeyCode::Esc = key_event.code {
                    return Ok(());
                } else {
                    input_handler::handle_input(app.clone(), key_event.code);
                }
            }
        }

        // Update the timeout only if it is fully exhausted
        if last_tick.elapsed() >= tick_rate {
            last_tick = std::time::Instant::now();
        }
    }
}

fn ui<B: Backend>(frame: &mut Frame<B>, app: Arc<Mutex<App>>) {
    let layouts = kittui_racer::ui::layout_divider::divide_frame(frame.size());
    draw_ui_from_layout(app, layouts, frame);
}
