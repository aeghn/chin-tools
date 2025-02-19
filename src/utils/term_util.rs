use std::io::{Read, Write};

use crossterm::terminal::WindowSize;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode, window_size};

use crate::wrapper::anyhow::AResult;

pub fn get_window_size_px() -> AResult<WindowSize> {
    let mut window_size = window_size()?;

    if window_size.width == 0 || window_size.height == 0 {
        // send the command code to get the terminal window size
        print!("\x1b[14t");
        std::io::stdout().flush()?;

        // we need to enable raw mode here since this bit of output won't print a newline; it'll
        // just print the info it wants to tell us. So we want to get all characters as they come
        enable_raw_mode()?;

        // read in the returned size until we hit a 't' (which indicates to us it's done)
        let input_vec = std::io::stdin()
            .bytes()
            .flat_map(|b| b.ok())
            .take_while(|b| *b != b't')
            .collect::<Vec<_>>();

        // and then disable raw mode again in case we return an error in this next section
        disable_raw_mode()?;

        let input_line = String::from_utf8(input_vec)?;

        if input_line.starts_with("\x1b[4;") {
            // it should input it to us as `\e[4;<height>;<width>t`, so we need to split to get the h/w
            let mut splits = input_line.split([';', 't']);
            // ignore the first val
            _ = splits.next();

            window_size.height = splits
                .next()
                .ok_or_else(|| {
                    anyhow::anyhow!(format!(
                        "Terminal responded with unparseable size response '{input_line}'"
                    ))
                })?
                .parse::<u16>()?;

            window_size.width = splits
                .next()
                .ok_or_else(|| {
                    anyhow::anyhow!(format!(
                        "Terminal responded with unparseable size response '{input_line}'"
                    ))
                })?
                .parse::<u16>()?;
        } else {
            anyhow::bail!("Your terminal is falsely reporting a window size of 0; tdf needs an accurate window size to display graphics");
        }
    }

    Ok(window_size)
}
