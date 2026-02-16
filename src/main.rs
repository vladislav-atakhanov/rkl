mod layout;
mod transform;

use layout::Layout;

use argh::FromArgs;

#[derive(FromArgs, Debug)]
/// kdl - keymap definition language
struct Args {
    /// input file path
    #[argh(positional)]
    file: String,

    /// apply keymap to vial
    #[argh(switch)]
    vial: bool,

    /// generate kanata config
    #[argh(option)]
    kanata: Option<String>,
}

fn main() -> Result<(), String> {
    env_logger::init();

    let args: Args = argh::from_env();

    let content = std::fs::read_to_string(args.file).map_err(|e| e.to_string())?;
    let layout: Layout = content.parse()?;

    if args.vial {
        layout.vial(None)?;
    } else if let Some(a) = args.kanata {
        let text = layout.kanata()?;
        match a.as_str() {
            "-" => println!("{}", text),
            filename => {
                std::fs::write(filename, text).map_err(|e| e.to_string())?;
                println!("Wrote to {}", filename)
            }
        }
    } else {
        eprintln!("Keymap not applied")
    }

    Ok(())
}
