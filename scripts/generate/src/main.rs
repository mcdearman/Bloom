//! Writes `src/Cases.mw` for anstyle.
//!
//! ```text
//! cargo run --release -- <package root>
//! ```
//!
//! Inputs, with what the crate makes of them: styles rendered, effect sets
//! combined, palette colours converted, and the sixteen colours brightened.
//! The library is ported by hand into `src/`, and the crate's source is
//! fingerprinted.

use anstyle::{Ansi256Color, AnsiColor, Color, Effects, RgbColor, Style};
use std::fmt::Write as _;
use std::path::PathBuf;

/// The crate version pinned in `Cargo.toml`.
const UPSTREAM_VERSION: &str = "1.0.14";

/// The fingerprint of the crate's source, which `src/` ports.
const SOURCES: u64 = 0xb4c1_374d_cbc5_3f17;

fn main() {
    let root = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "../..".into()));

    let print = fingerprint(include_str!(concat!(env!("OUT_DIR"), "/sources.rs.txt")));
    if print != SOURCES {
        eprintln!(
            "error: anstyle is not the version src/ ports.\n\
             Compare its source in {} with the previous version, carry any change\n\
             into src/, then set SOURCES in scripts/generate/src/main.rs to\n\
             {print:#x}",
            env!("UPSTREAM_DIR")
        );
        std::process::exit(1);
    }

    let cases = cases();
    let path = root.join("src/Cases.mw");
    std::fs::write(&path, &cases).unwrap();
    eprintln!("wrote {} ({} bytes)", path.display(), cases.len());
}

/// FNV-1a: stable across builds, which `DefaultHasher` does not promise.
fn fingerprint(text: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

// --- encoding -----------------------------------------------------------------------

/// A number as `digits` base-64 digits, most significant first, each digit the
/// character `'0' + d`: `'0'` to `'o'`, one contiguous run of ASCII.
fn digits(out: &mut String, value: u64, digits: u32) {
    assert!(
        value < 1 << (6 * digits),
        "{value} does not fit in {digits} digits"
    );
    for k in (0..digits).rev() {
        out.push(char::from(b'0' + ((value >> (6 * k)) & 63) as u8));
    }
}

/// A string, as its length in bytes (3 digits) and then its bytes.
fn text(out: &mut String, s: &str) {
    digits(out, s.len() as u64, 3);
    out.push_str(s);
}

/// `text` as one Meadow string literal, broken with `\`-newline every `width`
/// characters. Only printable ASCII is written raw; a space that would start a
/// line is `\x20`, since a continuation drops leading whitespace.
fn long_literal(text: &str, width: usize) -> String {
    let mut out = String::with_capacity(text.len() + text.len() / width * 4 + 2);
    out.push('"');
    for (i, c) in text.chars().enumerate() {
        let line_start = i > 0 && i % width == 0;
        if line_start {
            out.push_str("\\\n    ");
        }
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '$' => out.push_str("\\$"),
            ' ' if line_start => out.push_str("\\x20"),
            ' '..='~' => out.push(c),
            _ => {
                let _ = write!(out, "\\u{{{:X}}}", u32::from(c));
            }
        }
    }
    out.push('"');
    out
}

// --- inputs -------------------------------------------------------------------------

/// A small deterministic generator, so that the cases are the same on every run.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

const ANSI: [AnsiColor; 16] = [
    AnsiColor::Black,
    AnsiColor::Red,
    AnsiColor::Green,
    AnsiColor::Yellow,
    AnsiColor::Blue,
    AnsiColor::Magenta,
    AnsiColor::Cyan,
    AnsiColor::White,
    AnsiColor::BrightBlack,
    AnsiColor::BrightRed,
    AnsiColor::BrightGreen,
    AnsiColor::BrightYellow,
    AnsiColor::BrightBlue,
    AnsiColor::BrightMagenta,
    AnsiColor::BrightCyan,
    AnsiColor::BrightWhite,
];

const EFFECTS: [Effects; 12] = [
    Effects::BOLD,
    Effects::DIMMED,
    Effects::ITALIC,
    Effects::UNDERLINE,
    Effects::DOUBLE_UNDERLINE,
    Effects::CURLY_UNDERLINE,
    Effects::DOTTED_UNDERLINE,
    Effects::DASHED_UNDERLINE,
    Effects::BLINK,
    Effects::INVERT,
    Effects::HIDDEN,
    Effects::STRIKETHROUGH,
];

/// A colour, perhaps, written as `Tests.mw` reads it: a kind (none, one of
/// the sixteen, palette, RGB) and its numbers.
fn random_color(rng: &mut Rng, out: &mut String) -> Option<Color> {
    let kind = rng.below(4);
    digits(out, kind, 1);
    match kind {
        0 => None,
        1 => {
            let i = rng.below(16);
            digits(out, i, 1);
            Some(Color::Ansi(ANSI[i as usize]))
        }
        2 => {
            let i = rng.below(256);
            digits(out, i, 2);
            Some(Color::Ansi256(Ansi256Color(i as u8)))
        }
        _ => {
            let [r, g, b] = [rng.below(256), rng.below(256), rng.below(256)];
            for c in [r, g, b] {
                digits(out, c, 2);
            }
            Some(Color::Rgb(RgbColor(r as u8, g as u8, b as u8)))
        }
    }
}

/// A set of effects, written as its bits.
fn random_effects(rng: &mut Rng, out: &mut String) -> Effects {
    let bits = match rng.below(4) {
        0 => 0,
        1 => 1 << rng.below(12),
        _ => rng.below(1 << 12),
    };
    digits(out, bits, 2);
    EFFECTS
        .iter()
        .enumerate()
        .filter(|(i, _)| bits & (1 << i) != 0)
        .fold(Effects::new(), |acc, (_, e)| acc | *e)
}

/// The bits of a set of effects, which the crate keeps private.
fn bits(e: Effects) -> u64 {
    EFFECTS
        .iter()
        .enumerate()
        .filter(|(_, x)| e.contains(**x))
        .map(|(i, _)| 1 << i)
        .sum()
}

// --- cases --------------------------------------------------------------------------

fn cases() -> String {
    let mut rng = Rng(0xa575_7e1e_c0ff_ee42);
    let mut body = String::new();
    let mut counts = [0usize; 3];

    // Kind 0: styles, and their effects combined with others.
    for _ in 0..3000 {
        counts[0] += 1;
        digits(&mut body, 0, 1);
        let fg = random_color(&mut rng, &mut body);
        let bg = random_color(&mut rng, &mut body);
        let underline = random_color(&mut rng, &mut body);
        let effects = random_effects(&mut rng, &mut body);
        let other = random_effects(&mut rng, &mut body);
        let style = Style::new()
            .fg_color(fg)
            .bg_color(bg)
            .underline_color(underline)
            .effects(effects);
        assert_eq!(style.to_string(), style.render().to_string());
        text(&mut body, &style.render().to_string());
        text(&mut body, &style.render_reset().to_string());
        text(&mut body, &format!("{style:#}"));
        digits(&mut body, u64::from(style.is_plain()), 1);
        text(&mut body, &format!("{effects:?}"));
        digits(&mut body, u64::from(effects.contains(other)), 1);
        digits(&mut body, bits(effects.insert(other)), 2);
        digits(&mut body, bits(effects.remove(other)), 2);
        digits(&mut body, bits((style | other).get_effects()), 2);
        digits(&mut body, bits((style - other).get_effects()), 2);
        let names: Vec<String> = effects.iter().map(|e| format!("{e:?}")).collect();
        text(&mut body, &names.join(","));
    }

    // Kind 1: palette colours as one of the sixteen.
    for i in 0..=255u8 {
        counts[1] += 1;
        digits(&mut body, 1, 1);
        digits(&mut body, u64::from(i), 2);
        let ansi = Ansi256Color(i).into_ansi();
        digits(&mut body, ansi.map_or(16, |a| a as u64), 1);
    }

    // Kind 2: the sixteen, brightened and dimmed, and on a background.
    for (i, a) in ANSI.iter().enumerate() {
        counts[2] += 1;
        digits(&mut body, 2, 1);
        digits(&mut body, i as u64, 1);
        digits(&mut body, a.bright(true) as u64, 1);
        digits(&mut body, a.bright(false) as u64, 1);
        digits(&mut body, u64::from(a.is_bright()), 1);
        let bg = ANSI[15 - i];
        text(&mut body, &a.on(bg).render().to_string());
        text(&mut body, &a.on_default().render().to_string());
        text(&mut body, &Ansi256Color::from(*a).render_fg().to_string());
    }

    let mut out = String::new();
    let _ = writeln!(
        out,
        "-- GENERATED by scripts/generate.sh from anstyle {UPSTREAM_VERSION}.
-- Do not edit: run the script again instead.
--
-- Inputs, with what the crate makes of them, for `Tests.mw`: {} styles, {}
-- palette colours and {} of the sixteen colours.
--
-- Copyright the anstyle contributors, and the Meadow port's authors.
-- Dual-licensed under Apache-2.0 or MIT: see COPYRIGHT.

-- Each case starts with its kind (1 base-64 digit), and its fields follow in
-- the order `Tests.mw` reads them. A string is its length in bytes (3 digits)
-- and then its bytes; a colour is a kind (none, one of the sixteen, palette,
-- RGB) and its numbers; a set of effects is its bits (2 digits).
@cfg(test)
@pub(pkg) def cases =
  {}",
        counts[0],
        counts[1],
        counts[2],
        long_literal(&body, 96)
    );
    out
}
