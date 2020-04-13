use clap::Clap;
use quickshell::shell;
use regex::Regex;

/// Pulse Audio controller for i3blocks.
#[derive(Clap)]
#[clap(author = "Beinsezii")]
struct Opts {
    /// `$button` arg optionally passed from i3blocks
    button: Option<i32>,

    /// Work on sources (input devices) instead of sinks (output devices)
    #[clap(long)]
    source: bool,

    /// Device ID to work on. Uses default device otherwise. Currently only supports numbers.
    #[clap(long)]
    id: Option<String>,

    #[clap(long, default_value = "#dc322f")]
    color_error: String,

    #[clap(long, default_value = "#b58900")]
    color_high: String,

    #[clap(long, default_value = "#268bd2")]
    color_low: String,

    #[clap(long, default_value = "#859900")]
    color_mute: String,

    #[clap(long = "high", default_value = "🔊")]
    icon_high: String,

    #[clap(long = "low", default_value = "🔉")]
    icon_normal: String,

    #[clap(long = "off", default_value = "🔈")]
    icon_low: String,

    #[clap(long = "mute", default_value = "🔇")]
    icon_mute: String,

    /// [mic]rophone ic[on]. I'm proud of this one.
    #[clap(long, default_value = "🎙️")]
    micon: String,
}

fn main() {
    // console args.
    let opts: Opts = Opts::parse();

    // seems to capture Opts as a whole? idk.
    // Assinging a variable outside the closure works so it stays.
    let color_error = opts.color_error;
    let fail = |text: &str| {
        //long format
        println!("{}", text);
        //short format
        println!("{}", text);
        //color
        println!("{}", color_error);
    };

    let mutecmd: &str;
    let volumecmd: &str;
    let listcmd: &str;
    let mut micon = String::new();

    if !opts.source {
        mutecmd = "set-sink-mute";
        volumecmd = "set-sink-volume";
        listcmd = "sinks";
    } else {
        mutecmd = "set-source-mute";
        volumecmd = "set-source-volume";
        listcmd = "sources";
        micon = opts.micon;
    };

    let device: &str;
    let info: String;

    match opts.id {
        // see if the ID given is numeric
        Some(id) => match id.parse::<i32>() {
            //if it's a numeric ID, use the source/sink of that number
            Ok(_) => {
                info = match shell("pactl", &["list", "short", listcmd]) {
                    Some(val) => val,
                    None => {
                        fail(&format!("pactl list short {} failed", listcmd));
                        return;
                    }
                };
                // convert numbered sink to same format as default
                let re = Regex::new(format!("(?m)^{}.+?$", id).as_str())
                    .unwrap()
                    .find(info.as_str());
                let re = match re {
                    Some(val) => val,
                    None => {
                        fail("Numeric ID not found");
                        return;
                    }
                };
                // get()'s re match, splits to second word.
                device = info
                    .get(re.start()..re.end())
                    .unwrap()
                    .split_whitespace()
                    .collect::<Vec<&str>>()[1];
            }
            // TODO if it's not numeric, search for it
            Err(_) => {
                fail("Device name search unimplemented");
                return;
            }
        },

        None => {
            // gets status using pactl
            info = match shell("pactl", &["info"]) {
                Some(val) => val,
                None => {
                    fail("pactl info failed.");
                    return;
                }
            };
            // simply searches `pactl info` for a default sink or source.
            match listcmd {
                "sinks" => {
                    let re = Regex::new(r"Default Sink: [^\n]+")
                        .unwrap()
                        .find(info.as_str())
                        .unwrap();
                    // +14 to ignore "Default Sink: "
                    device = info.get(re.start() + 14..re.end()).unwrap();
                }
                "sources" => {
                    let re = Regex::new(r"Default Source: [^\n]+")
                        .unwrap()
                        .find(info.as_str())
                        .unwrap();
                    // +14 to ignore "Default Source: "
                    device = info.get(re.start() + 16..re.end()).unwrap();
                }
                _ => unreachable!(),
            }
        }
    };

    match opts.button {
        Some(button) => match button {
            // 1 = LMB, 2 = MMB, 3 = RMB, 4 = ScrollUp, 5 = ScrollDown
            1 => shell("pactl", &[mutecmd, device, "toggle"]),
            3 => shell("pactl", &[volumecmd, device, "100%"]),
            4 => shell("pactl", &[volumecmd, device, "+1dB"]),
            5 => shell("pactl", &[volumecmd, device, "-1dB"]),
            _ => None,
        },
        None => None,
    };

    let mut status = match shell("pactl", &["list", listcmd]) {
        Some(val) => val,
        None => {
            fail("Pactl list failed");
            return;
        }
    };

    // finds the device in the current status list.
    status = match status.find(device) {
        Some(val) => status.get(val..).unwrap().to_string(),
        None => {
            fail("Failed to find device.");
            return;
        }
    };

    let re = Regex::new(".+?Volume: .+?dB")
        .unwrap()
        .find(status.as_str());

    let mut volume: &str;
    match re {
        Some(val) => {
            volume = status.get(val.start()..val.end()).unwrap();
            volume = volume.get(volume.find("/").unwrap() + 2..).unwrap().trim();
        }
        None => {
            fail("Couldn't find device volume.");
            return;
        }
    };

    let re = Regex::new(".+?Mute: .+?\n").unwrap().find(status.as_str());
    let mut mute: &str;
    match re {
        Some(val) => {
            mute = status.get(val.start()..val.end()).unwrap();
            mute = mute
                .get(mute.find(":").unwrap() + 2..mute.len() - 1)
                .unwrap()
                .trim()
        }
        None => {
            fail("Couldn't find device mute status");
            return;
        }
    }

    // get volume as int
    let intvol: i32 = volume
        .get(..volume.find("%").unwrap())
        .unwrap()
        .parse()
        .unwrap();

    let icon: String;
    let mut color = String::from("");
    if mute == "yes" {
        icon = opts.icon_mute;
        color = opts.color_mute;
    } else {
        if intvol > 100 {
            color = opts.color_high;
            icon = opts.icon_high;
        } else if intvol < 100 {
            color = opts.color_low;
            icon = opts.icon_low;
        } else {
            icon = opts.icon_normal;
        }
    }

    // long format
    println!("{}{} {}", micon, icon, volume);
    // short format
    println!("{}{} {}%", micon, icon, intvol);

    if color != "" {
        println!("{}", color);
    }
}
