# bloom

Styles for terminal text — colours and effects — and the ANSI escape sequences
that turn them on and off, for [Meadow](https://github.com/mcdearman/meadow).

This package is a port of Rust's [`anstyle`](https://github.com/rust-cli/anstyle)
1.0.14 and writes exactly the same sequences. Like the crate, it does not
decide whether a terminal can show colour. It only builds the sequences.

## Install

```sh
meadow add mcdearman/Bloom
```

## Use

```meadow
use Bloom (style, bold, withFg, Ansi, Red, paint, render, renderReset)

def warning = bold (withFg (Just (Ansi Red)) style)

def main =
  ( paint warning "careful",                           -- "\u{1b}[1m\u{1b}[31mcareful\u{1b}[0m"
    render warning ++ "careful" ++ renderReset warning  -- the same
  )
```

### Colours

A `Color` is one of three kinds:

- `Ansi c`: one of the sixteen `AnsiColor`s, `Black` to `BrightWhite`;
- `Ansi256 n`: a colour from the xterm palette, `n` from 0 to 255;
- `Rgb r g b`: each component from 0 to 255.

`renderFg`, `renderBg` and `renderUnderline` give the sequence that selects a
colour. For the sixteen:

- `bright yes c` makes a colour bright, or plain;
- `isBright` tells which kind it is;
- `ansiIndex` gives its place in the palette;
- `ansiFromIndex` converts back, and is `None` above 15.

### Effects

An `Effect` is one of `Bold`, `Dimmed`, `Italic`, `Underline`,
`DoubleUnderline`, `CurlyUnderline`, `DottedUnderline`, `DashedUnderline`,
`Blink`, `Invert`, `Hidden` and `Strikethrough`. An `Effects` value is a set of
them.

| function | |
|---|---|
| `plain`, `effectSet e`, `effectsOf [e, …]` | build a set |
| `hasEffect`, `containsEffects`, `isPlainEffects` | test one |
| `insertEffects`, `removeEffects`, `setEffects es other enable` | change one |
| `effectsList es` | its effects, in order |
| `renderEffects es` | the sequences that turn them on |
| `effectsDebug es` | `"Effects(BOLD \| ITALIC)"`, as the crate prints it |

### Styles

A `Style` is a record with three optional colours, `fg`, `bg` and `underline`,
and a set of `effects`. To build one:

- start from `style`, which is plain;
- add colours with `withFg`, `withBg` and `withUnderlineColor`;
- set effects with `withEffects`, `addEffects` or `subEffects`, or with the
  shortcuts `bold`, `dimmed`, `italic`, `underlined`, `blink`, `invert`,
  `hidden` and `strikethrough`;
- or begin with `on fg bg`, `onDefault fg` or `styleOfEffects`.

To write one out:

- `render s` turns the style on;
- `renderReset s` turns it off again, and is empty for a plain style;
- `reset` is the bare off sequence;
- `paint s text` wraps `text` in both.

The crate's `Style::underline` is called `underlined` here, so that it is not
confused with the `underline` field.

## How it's made

`src/Color.mw`, `src/Effects.mw` and `src/Style.mw` are hand translations of
the crate's source. **`src/Cases.mw`** is generated test data:

- 3,000 random styles, each with its rendering and its effects combined with
  another set;
- all 256 palette colours converted to the sixteen;
- the sixteen colours brightened, dimmed and set on backgrounds.

Every expected result comes from calling the crate. Run `scripts/generate.sh`
to regenerate; it needs a Rust toolchain.

## Licence

Dual-licensed under [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your
option, like the crate. See [COPYRIGHT](COPYRIGHT).
