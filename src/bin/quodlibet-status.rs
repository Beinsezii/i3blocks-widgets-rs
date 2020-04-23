use clap::Clap;
use quickshell::shell;

/// Quod Libet controller for i3blocks. Best used with quodlibet-volume.
#[derive(Clap)]
#[clap(author = "Beinsezii")]
struct Opts {
    /// `$button` arg optionally passed from i3blocks
    button: Option<i32>,

    #[clap(long, default_value = "#dc322f")]
    color_error: String,

    #[clap(long = "play", default_value = "▶")]
    icon_play: String,

    #[clap(long = "pause", default_value = "⏸️")]
    icon_pause: String,

    #[clap(long = "stop", default_value = "⏹️")]
    icon_stop: String,

    /// Format string for the long view
    #[clap(long, short, default_value="<title><artist| / <artist>><album| / <album>>")]
    long: String,

    /// Format string for the short view
    #[clap(long, short, default_value="<title>")]
    short: String,
}

fn main() {
    // console args.
    let opts: Opts = Opts::parse();

    let error_c = opts.color_error;
    let fail = |text: &str| {
        //long format
        println!("{}", text);
        //short format
        println!("{}", text);
        //color
        println!("{}", error_c);
    };

    match opts.button {
        // 1 = LMB, 2 = MMB, 3 = RMB, 4 = ScrollUp, 5 = ScrollDown
        Some(num) => match num {
            1 => shell("quodlibet", &["--play-pause"]),
            2 => shell("quodlibet", &["--toggle-window"]),
            3 => shell("quodlibet", &["--stop"]),
            4 => shell("quodlibet", &["--previous"]),
            5 => shell("quodlibet", &["--next"]),
            _ => {
                fail("Invalid button.");
                return;
            }
        },
        None => None,
    };

    // it takes about 0.1 to 0.15 seconds for a single run of quodlibet, so threads.
    let long_in = opts.long;
    let short_in = opts.short;
    let status_t = std::thread::spawn(|| shell("quodlibet", &["--status"]));
    let long_t = std::thread::spawn(move || shell("quodlibet", &["--print-playing", &long_in]));
    let short_t = std::thread::spawn(move || shell("quodlibet", &["--print-playing", &short_in]));

    let status = match status_t.join().unwrap() {
        None => {
            fail("Quodlibet failed.");
            return;
        }
        Some(result) => result,
    };
    let long = match long_t.join().unwrap() {
        None => "Long String Error".to_string(),
        Some(result) => result,
    };
    let short = match short_t.join().unwrap() {
        None => "Short String Error".to_string(),
        Some(result) => result,
    };

    // if somehow status doesn't quit the app cause it's blank, but it's *also* not playing or
    // paused, icon is "?" aka "idk what the hell's going on this shouldn't be possible"
    let mut icon = String::from("?");
    if status.starts_with("playing") {
        icon = opts.icon_play;
    } else if status.starts_with("paused") {
        // quodlibet displays "paused" for both pause and stop states, so it simply checks if the
        // song is 'on 0.000' seconds aka hasn't started yet.
        if status.ends_with("on 0.000") {
            icon = opts.icon_stop;
        } else {
            icon = opts.icon_pause;
        }
    }

    // long format
    println!("{} {}", icon, long);
    // short format
    println!("{} {}", icon, short);
}
