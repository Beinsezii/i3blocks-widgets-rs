use clap::Clap;
use regex::Regex;
use quickshell::shell;

/// Quod Libet volume controller for i3blocks. Best used with quodlibet-status.
#[derive(Clap)]
#[clap(author = "Beinsezii")]
struct Opts {
    /// $button arg optionally passed from i3blocks
    button: Option<i32>,

    #[clap(long = "high", default_value = "🔊")]
    icon_high: String,

    #[clap(long = "normal", default_value = "🔉")]
    icon_normal: String,

    #[clap(long = "low", default_value = "🔈")]
    icon_low: String,
}

fn main() {
    // console args.
    let opts: Opts = Opts::parse();

    match opts.button {
        Some(button) => match button {
            // 1 = LMB, 2 = MMB, 3 = RMB, 4 = ScrollUp, 5 = ScrollDown
            1 | 4 => shell("quodlibet", &["--volume-up"]),
            3 | 5 => shell("quodlibet", &["--volume-down"]),
            _ => None,
        },
        None => None,
    };

    let status = match shell("quodlibet", &["--status"]) {
        Some(val) => val,
        None => {
            // long
            println!(" ");
            // short
            println!(" ");
            // quodlbet-status will print "Quodlibet failed.", so volume prints a single space to keep
            // the separator.
            return;
        },
    };

    // I don't know how an unwrap would fail on the new, considering the regex is hardcoded.
    let re = Regex::new(r"\d\.\d{3}").unwrap().find(status.as_str());

    // this looks wrong. I wanna return from main on None option, else use the regex match.
    // would be functionally identical to an if statement. Is one more proper? Is there a better way?
    let re = match re {
        None => return,
        _ => re.unwrap(),
    };

    // unwrap physically can't fail since getting here means regexing \d digits
    // ...right?
    let volume: f32 = status[re.start()..re.end()].parse::<f32>().unwrap() * 100.0;

    let icon: String;
    if volume > 66.0 {
        icon = opts.icon_high;
    } else if volume < 34.0 {
        icon = opts.icon_low;
    } else {
        icon = opts.icon_normal;
    }

    // long format
    println!("{} {:.0}%", icon, volume);
    // short format
    println!("{} {:.0}%", icon, volume);
}
