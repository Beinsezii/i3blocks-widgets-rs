use clap::Clap;
use quickshell::shell;
use regex::Regex;

/// i3blocks display for NVIDIA GPUs.
#[derive(Clap)]
#[clap(author = "Beinsezii")]
struct Opts {
    /// Numeric id of GPU. Only relevant for multi-gpu systems.
    #[clap(long, default_value = "0")]
    id: i32,

    #[clap(long, default_value = "#dc322f")]
    color_error: String,

    #[clap(long, default_value = "#268bd2")]
    color_idle: String,

    // I want the text to turn a different color when the GPU is overworked and throttling. Would
    // that be "SW Thermal Slowdown" in nvidia-smi? There's a jillion throttle reasons.
    /// Unimplemented
    #[clap(long, default_value = "#b58900")]
    color_throttle: String,

    /// Disables showing utilization.
    #[clap(long)]
    no_util: bool,

    /// Disables showing memory.
    #[clap(long)]
    no_mem: bool,

    /// Disables showing temperature.
    #[clap(long)]
    no_temp: bool,

    /// Always shows memory as a % instead of MiB
    #[clap(long)]
    perc_mem: bool,

    /// Enables showing temperature in short as well as long.
    #[clap(long)]
    short_temp: bool,

    /// Shows GPU name in long.
    #[clap(long, short = "g")]
    gpu_name: bool,

    /// Shows video encoder utilization in long.
    #[clap(long, short = "e")]
    encode: bool,

    /// Shows video decoder utilization in long.
    #[clap(long, short = "d")]
    decode: bool,
}

fn main() {
    // console args.
    let opts: Opts = Opts::parse();

    let error = opts.color_error;
    let fail = |text: &str| {
        //long format
        println!("{}", text);
        //short format
        println!("{}", text);
        //color
        println!("{}", error);
    };

    let mut utilization = "";
    let mut memory: f32 = 0.0;
    let mut max_memory: f32 = 0.0;
    let mut temperature = "";
    // if name is a &str, it gets dropped after assigning from .replace()
    let mut name = String::from("");
    let mut encode = "";
    let mut decode = "";
    let mut idle = false;
    let mut throttle = false;

    // Uses `nvidia-smi` to get a status of the GPU id given. Has literally all the information.
    let status = match shell("nvidia-smi", &["-q", "-i", &opts.id.to_string()]) {
        Some(val) => val,
        None => {
            fail("Command `nvidia-smi -q -i 0` failed.");
            return;
        }
    };

    // Utilization
    if !opts.no_util {
        // Since there's no lookaround, I put the important text in a separate capture group.
        let caps = Regex::new(r"Gpu +: ([\d]+)").unwrap().captures(&status);
        // Match to whether or not the captures worked.
        utilization = match caps {
            // Update text if they did.
            Some(caps) => caps.get(1).unwrap().as_str(),
            None => {
                fail("Couldn't find utilization.");
                return;
            }
        }
    };

    // VRAM usage
    if !opts.no_mem {
        // this one grabs two groups, one for Total and one for Used
        let caps =
            Regex::new(r"FB Memory Usage[ \n]+Total +: ([\d]+) MiB[ \n]+Used +: ([\d]+) MiB")
                .unwrap()
                .captures(&status);
        match caps {
            Some(caps) => {
                // converts groups into floats for easy usage % later.
                // parse unwrap can't fail cause the groups only match to [\d]+
                max_memory = caps.get(1).unwrap().as_str().parse().unwrap();
                memory = caps.get(2).unwrap().as_str().parse().unwrap();
            }
            None => {
                fail("Couldn't find memory.");
                return;
            }
        }
    };

    // Temperature
    if !opts.no_temp {
        let caps = Regex::new(r"GPU Current Temp +: ([\d]+)")
            .unwrap()
            .captures(&status);
        temperature = match caps {
            Some(caps) => caps.get(1).unwrap().as_str(),
            None => {
                fail("Couldn't find temperature.");
                return;
            }
        }
    };

    // Product name, without the brand name in front.
    // Ex, product = GeForce GTX 1070; Brand = GeForce; Name = GTX 1070
    if opts.gpu_name {
        let caps = Regex::new(r"Product Name +: ([^\n]+)")
            .unwrap()
            .captures(&status);
        // Name now contains product name
        name = match caps {
            Some(caps) => String::from(caps.get(1).unwrap().as_str()),
            None => {
                fail("Couldn't find GPU name.");
                return;
            }
        };
        // get brand name
        let caps = Regex::new(r"Product Brand +: ([^\n]+)")
            .unwrap()
            .captures(&status);
        let brand = match caps {
            Some(caps) => caps.get(1).unwrap().as_str(),
            None => "",
        };
        // subtract brand from product and trim
        name = String::from(name.replace(brand, "").trim());
    };

    if opts.encode {
        let caps = Regex::new(r"Encoder +: ([\d]+)").unwrap().captures(&status);
        encode = match caps {
            Some(caps) => caps.get(1).unwrap().as_str(),
            None => {
                fail("Couldn't find encoder utilization.");
                return;
            }
        }
    };

    if opts.decode {
        let caps = Regex::new(r"Decoder +: ([\d]+)").unwrap().captures(&status);
        decode = match caps {
            Some(caps) => caps.get(1).unwrap().as_str(),
            None => {
                fail("Couldn't find decoder utilization.");
                return;
            }
        }
    };

    // "Throttle status -- Idle : Active" sets idle bool to true
    if opts.color_idle != "" {
        let caps = Regex::new(r"Idle +: ([[:alpha:]]+)")
            .unwrap()
            .captures(&status);
        let idle_status = match caps {
            Some(caps) => String::from(caps.get(1).unwrap().as_str()),
            None => {
                fail("Couldn't find idle status.");
                return;
            }
        };
        if idle_status == "Active" {
            idle = true;
        };
    };

    // is there a way to make RustFMT ignore a part? Everything below here looks better manually
    // formatted imo.
    let mut long = if opts.gpu_name { format!("{}: ", name) }
                   else { format!("GPU {}: ", opts.id) };
    if !opts.no_util { long += &format!("GPU {}%|", utilization) };
    if !opts.no_mem && !opts.perc_mem { long += &format!("{}/{}MiB|", memory, max_memory) };
    if !opts.no_mem && opts.perc_mem { long += &format!("MEM {:.0}%|", memory / max_memory * 100.0) };
    if !opts.no_temp { long += &format!("{}C|", temperature) };
    if opts.encode { long += &format!("ENC {}%|", encode) };
    if opts.decode { long += &format!("VID {}%", decode) };
    println!("{}", long.trim_end_matches("|"));

    let mut short = format!("GPU {}: ", opts.id);
    if !opts.no_util { short += &format!("GPU {}|", utilization) };
    if !opts.no_mem { short += &format!("MEM {:.0}|", memory / max_memory * 100.0) };
    if opts.short_temp && !opts.no_temp { short += &format!("{}C", temperature) };
    println!("{}", short.trim_end_matches("|"));

    if idle { println!("{}", opts.color_idle)
    } else if throttle { println!("{}", opts.color_throttle) };

}
