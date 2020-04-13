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

    /// Shows 'discsubtitle' tag preceding the title.
    #[clap(long = "dst")]
    discsubtitle: bool,
}

fn main() {
    // console args.
    let opts: Opts = Opts::parse();

    let fail = |text: &str| {
        //long format
        println!("{}", text);
        //short format
        println!("{}", text);
        //color
        println!("{}", opts.color_error);
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

    // it takes about 0.1 to 0.15 seconds for a single run of quodlibet.
    // multiply that by 5 and it can take up to whole-ass 2/3 second after a click to update the
    // screen. Idk why it's so slow to get a single string but here we are multi-threading prints
    // because apparently getting the formatted text takes like .15 real seconds of CPU time.
    let status_t = std::thread::spawn(|| shell("quodlibet", &["--status"]));
    let title_t = std::thread::spawn(|| shell("quodlibet", &["--print-playing", "<title>"]));
    let artist_t = std::thread::spawn(|| shell("quodlibet", &["--print-playing", "<artist>"]));
    let album_t = std::thread::spawn(|| shell("quodlibet", &["--print-playing", "<album>"]));
    // would like to only spawn this if enabled but haven't thought of a pretty way to do so.
    // one option would be spawning a dummy thread that just echos I guess. Rust ain't smart
    // enough to figure out that disc_t will always be initialized if I put both this and the
    // thread join in `if opts.discsubtitle` blocks.
    // Just worry about the performance hit on low-core systems
    let disc_t = std::thread::spawn(|| shell("quodlibet", &["--print-playing", "<discsubtitle>"]));

    let status = match status_t.join().unwrap() {
        None => {
            fail("Quodlibet failed.");
            return;
        }
        Some(result) => result,
    };
    let disc = match disc_t.join().unwrap() {
            None => "".to_string(),
            Some(result) => {
                if opts.discsubtitle { result + " - " }
                else {"".to_string()}
            },
        };
    let title = match title_t.join().unwrap() {
        // Technically quodlibet should return "foo.mp3 [Unknown]" if no title is found but
        // just in case.
        None => "Unknown Title".to_string(),
        Some(result) => result,
    };
    let artist = match artist_t.join().unwrap() {
        None => "Unknown Artist".to_string(),
        Some(result) => result,
    };
    let album = match album_t.join().unwrap() {
        None => "Unknown Album".to_string(),
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
    println!("{} {}{} / {} / {}", icon, disc, title, artist, album);
    // short format
    println!("{} {}", icon, title);
}
