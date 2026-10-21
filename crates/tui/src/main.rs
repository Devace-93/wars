//! wars-tui: the console client. Everything is driven from the keyboard; the map is
//! drawn as coloured glyphs and the bottom panel always lists the actions available
//! for the current selection with their shortcut letters.

mod actions;
mod app;
mod game;
mod net_ui;
mod view;

use app::App;
use clap::Parser;
use crossterm::event::{self, Event, KeyEventKind};
use std::time::Duration;
use wars_client::wars_mapgen::{generate, MapParams, MapSize, MapType};
use wars_client::wars_sim::ids::PlayerId;
use wars_client::{LocalMatch, Session};

#[derive(Parser, Debug)]
#[command(name = "wars-tui", about = "Wars console client", version)]
struct Cli {
    /// Create a local match, step 200 ticks without a terminal and exit 0 (for CI).
    #[arg(long)]
    headless_smoke: bool,
    /// Skip the menu: start a single-player match right away.
    #[arg(long)]
    quick: bool,
    /// Map seed for --quick / --headless-smoke.
    #[arg(long, default_value_t = 1)]
    seed: u64,
    /// Map type for --quick / --headless-smoke (arabia, black forest, islands, ...).
    #[arg(long, default_value = "arabia")]
    map: String,
    /// Map size for --quick / --headless-smoke (tiny, small, medium, large, huge).
    #[arg(long, default_value = "tiny")]
    size: String,
}

fn params(cli: &Cli) -> MapParams {
    let size = MapSize::ALL
        .iter()
        .copied()
        .find(|s| s.name().eq_ignore_ascii_case(&cli.size))
        .unwrap_or(MapSize::Tiny);
    MapParams {
        map_type: MapType::from_name(&cli.map).unwrap_or(MapType::Arabia),
        size,
        seed: cli.seed,
        ..Default::default()
    }
}

fn headless_smoke(cli: &Cli) -> i32 {
    let data = wars_client::wars_data::data();
    let world = generate(data, &params(cli));
    let me = PlayerId(1);
    let mut g = game::Game::new(Session::Local(LocalMatch::new(world, me)));
    g.set_view_size(80, 24);
    // Select every villager and send it to the nearest tree through the controller.
    let villagers: Vec<_> = g
        .session
        .world()
        .entities_of(me)
        .filter(|e| data.object(e.ty).is_villager)
        .map(|e| e.id)
        .collect();
    g.ctrl.select(villagers.clone(), false);
    let tree = g
        .session
        .world()
        .iter()
        .filter(|e| data.object(e.ty).yields.is_some() && e.owner.is_gaia())
        .min_by_key(|e| {
            let v = g.session.world().get(villagers[0]).unwrap();
            e.pos.dist_sq(v.pos)
        })
        .map(|e| (e.id, e.pos));
    if let Some((id, pos)) = tree {
        let cmd = g
            .ctrl
            .context_command(g.session.world(), data, me, pos, Some(id));
        if let Some(c) = cmd {
            g.session.issue(c);
        }
    }
    let Session::Local(m) = &mut g.session else {
        unreachable!()
    };
    for _ in 0..200 {
        m.step_once(data);
    }
    let w = &m.world;
    let acts = actions::actions_for(w, data, me, &villagers);
    let tc = w
        .entities_of(me)
        .find(|e| data.object(e.ty).is_town_center)
        .map(|e| e.id);
    let tc_acts = tc
        .map(|id| actions::actions_for(w, data, me, &[id]))
        .unwrap_or_default();
    println!(
        "smoke ok: tick {} entities {} villager actions {} tc actions {} wood {}",
        w.tick,
        w.count(),
        acts.len(),
        tc_acts.len(),
        w.player(me).resources.wood
    );
    if w.tick != 200 || acts.is_empty() || tc_acts.is_empty() {
        return 1;
    }
    0
}

fn run(terminal: &mut ratatui::DefaultTerminal, cli: &Cli) -> std::io::Result<()> {
    let mut app = App::new();
    if cli.quick {
        let world = generate(wars_client::wars_data::data(), &params(cli));
        app.start_local(world);
    }
    loop {
        app.tick();
        terminal.draw(|f| app.draw(f))?;
        let timeout = if app.in_game() {
            Duration::from_millis(45)
        } else {
            Duration::from_millis(120)
        };
        let mut first = true;
        // Handle every pending key before the next frame so fast typing stays snappy.
        while event::poll(if first { timeout } else { Duration::ZERO })? {
            first = false;
            match event::read()? {
                Event::Key(k) if k.kind == KeyEventKind::Press => app.key(k),
                Event::Resize(..) => break,
                _ => {}
            }
            if app.quit {
                break;
            }
        }
        if app.quit {
            return Ok(());
        }
    }
}

fn main() {
    let cli = Cli::parse();
    if cli.headless_smoke {
        std::process::exit(headless_smoke(&cli));
    }
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &cli);
    ratatui::restore();
    if let Err(e) = result {
        eprintln!("wars-tui: {e}");
        std::process::exit(1);
    }
}
