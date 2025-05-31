// Color constants for terminal output
pub const C_FG_BLACK: &str = "\x1b[30;1m";
pub const C_FG_RED: &str = "\x1b[31;1m";
pub const C_FG_GREEN: &str = "\x1b[32;1m";
pub const C_FG_YELLOW: &str = "\x1b[33;1m";
pub const C_FG_BLUE: &str = "\x1b[34;1m";
pub const C_FG_MAGENTA: &str = "\x1b[35;1m";
pub const C_FG_CYAN: &str = "\x1b[36;1m";
pub const C_FG_WHITE: &str = "\x1b[37;1m";
pub const C_FG_B_BLACK: &str = "\x1b[90;1m";
pub const C_FG_B_WHITE: &str = "\x1b[97;1m";
pub const COLOR_RESET: &str = "\x1b[m";

// ASCII art for vendors (short logos only for now)
const ASCII_AMD: &str = "\
$C2          '###############             \n\
$C2             ,#############            \n\
$C2                      .####            \n\
$C2              #.      .####            \n\
$C2            :##.      .####            \n\
$C2           :###.      .####            \n\
$C2           #########.   :##            \n\
$C2           #######.       ;            \n\
$C1                                       \n\
$C1    ###     ###      ###   #######     \n\
$C1   ## ##    #####  #####   ##     ##   \n\
$C1  ##   ##   ### #### ###   ##      ##  \n\
$C1 #########  ###  ##  ###   ##      ##  \n\
$C1##       ## ###      ###   ##     ##   \n\
$C1##       ## ###      ###   #######     \n";

const ASCII_INTEL_NEW: &str = "\
$C1  MMM                 oddl                   MMN   \n\
$C1  MMM                 dMMN                   MMN   \n\
$C1  ...  ....   ...     dMMM..      .cc.       NMN   \n\
$C1  MMM  :MMMdWMMMMMX.  dMMMMM,  .XMMMMMMNo    MMN   \n\
$C1  MMM  :MMMp    dMMM  dMMX   .NMW      WMN.  MMN   \n\
$C1  MMM  :MMM      WMM  dMMK   kMMXooooooNMMx  MMN   \n\
$C1  MMM  :MMM      NMM  dMMK   dMMX            MMN   \n\
$C1  MMM  :MMM      NMM  dMMMoo  OMM0....:Nx.   MMN   \n\
$C1  MMM  :WWW      XWW   lONMM   'xXMMMMNOc    MMN   \n";

const ASCII_ARM: &str = "\
$C1   #####  ##   # #####  ## ####  ######   \n\
$C1 ###    ####   ###      ####  ###   ###   \n\
$C1###       ##   ###      ###    ##    ###  \n\
$C1 ###    ####   ###      ###    ##    ###  \n\
$C1  ######  ##   ###      ###    ##    ###  \n";

const ASCII_NVIDIA: &str = "\
$C1               'cccccccccccccccccccccccccc   \n\
$C1               ;oooooooooooooooooooooooool   \n\
$C1           .:::.     .oooooooooooooooooool   \n\
$C1      .:cll;   ,c:::.     cooooooooooooool   \n\
$C1   ,clo'      ;.   oolc:     ooooooooooool   \n\
$C1.cloo    ;cclo .      .olc.    coooooooool   \n\
oooo   :lo,    ;ll;    looc    :oooooooool      \n\
 oooc   ool.   ;oooc;clol    :looooooooool      \n\
  :ooc   ,ol;  ;oooooo.   .cloo;     loool      \n\
    ool;   .olc.       ,:lool        .lool      \n\
      ool:.    ,::::ccloo.        :clooool      \n\
         oolc::.            ':cclooooooool      \n\
               ;oooooooooooooooooooooooool      \n\
                                                \n\
$C2######.  ##   ##  ##  ######   ##    ###     \n\
$C2##   ##  ##   ##  ##  ##   ##  ##   #: :#    \n\
$C2##   ##   ## ##   ##  ##   ##  ##  #######   \n\
$C2##   ##    ###    ##  ######   ## ##     ##  \n";

const ASCII_POWERPC: &str = "\
$C1     //////                                   //////    /////  \n\
$C1    //// /// ,//// /// ///  /// /////  ///// /// ////////      \n\
$C1   */////// /// ///////////// /// /// ///// ////////////       \n\
$C1   ///     /// /// ///////// ///     ///   ///        ////.    \n\
$C1  ///      /////   //  ///     //// ///   ///          /////   \n";

const ASCII_APPLE: &str = "\
$C1                    'c.                     \n\
$C2                 ,xNMM.                     \n\
$C3               .OMMMMo                      \n\
$C4               OMMM0,                       \n\
$C5     .;loddo:' loolloddol;.                 \n\
$C6   cKMMMMMMMMMMNWMMMMMMMMMM0:               \n\
$C7 .KMMMMMMMMMMMMMMMMMMMMMMMWd.               \n\
$C1 XMMMMMMMMMMMMMMMMMMMMMMMX.                 \n\
$C2;MMMMMMMMMMMMMMMMMMMMMMMM:                  \n\
$C3:MMMMMMMMMMMMMMMMMMMMMMMM:                  \n\
$C4.MMMMMMMMMMMMMMMMMMMMMMMMX.                 \n\
$C5 kMMMMMMMMMMMMMMMMMMMMMMMMWd.               \n\
$C6 .XMMMMMMMMMMMMMMMMMMMMMMMMMMk              \n\
$C7  .XMMMMMMMMMMMMMMMMMMMMMMMMK.              \n\
$C1    kMMMMMMMMMMMMMMMMMMMMMMd                \n\
$C2     ;KMMMMMMMWXXWMMMMMMMk.                 \n\
$C3       .cooc,.    .,coo:.                   \n";

fn logo_lines_for_vendor(vendor_id: &str) -> Option<Vec<String>> {
    let (raw_logo, colors): (&str, &[&str]) = match vendor_id {
        "AuthenticAMD" | "amd" => (ASCII_AMD, &[C_FG_WHITE, C_FG_RED]),
        "GenuineIntel" | "intel" => (ASCII_INTEL_NEW, &[C_FG_CYAN]),
        "ARM" | "arm" => (ASCII_ARM, &[C_FG_CYAN]),
        "NVIDIA" | "nvidia" => (ASCII_NVIDIA, &[C_FG_GREEN, C_FG_WHITE]),
        "PowerPC" | "powerpc" => (ASCII_POWERPC, &[C_FG_YELLOW]),
        "Apple" | "apple" => (ASCII_APPLE, &[C_FG_RED, C_FG_YELLOW, C_FG_GREEN, C_FG_CYAN, C_FG_BLUE, C_FG_MAGENTA, C_FG_WHITE]),
        _ => return None,
    };
    let mut processed_logo = raw_logo.to_string();
    for (i, color) in colors.iter().enumerate() {
        let placeholder = format!("$C{}", i + 1);
        processed_logo = processed_logo.replace(&placeholder, color);
    }
    processed_logo = processed_logo.replace("$CR", COLOR_RESET);
    let lines: Vec<String> = processed_logo.lines().map(|l| l.to_string()).collect();
    Some(lines)
}

pub fn get_logo_lines_for_vendor(vendor_id: &str) -> Option<Vec<String>> {
    logo_lines_for_vendor(vendor_id)
}
