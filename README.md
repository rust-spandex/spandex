[![Build Status](https://travis-ci.org/tforgione/spandex.svg?branch=master)](https://travis-ci.org/tforgione/spandex)

# SpanDeX

## Install

As always in rust, it's super simple to install.

First, install rust

``` sh
curl https://sh.rustup.rs -sSf | sh
```

Then, install SpanDeX

``` sh
cargo install --git https://github.com/tforgione/spandex
```

## Usage

For the moment, only two commands are available:
  - `spandex init <name>`: creates a directory for a SpanDeX document with a
    `spandex.toml` and an initial `main.md` files. If no name is specified, the
    name of the current directory will be used instead.

  - `spandex build`: triggers the build of SpanDeX, and generates an
    `output.pdf` file.

## Build the examples

To build one of the examples, go to the example directory and run `cargo run -- build`.

## Default fonts
  - CMU Serif BoldItalic
  - CMU Serif Extra BoldSlanted
  - CMU Bright Oblique
  - CMU Bright Roman
  - CMU Bright SemiBoldOblique
  - CMU Bright SemiBold
  - CMU Typewriter Text Light
  - CMU Typewriter Text LightOblique
  - CMU Serif Bold
  - CMU Classical Serif Italic
  - CMU Typewriter Text Italic
  - CMU Concrete BoldItalic
  - CMU Concrete Bold
  - CMU Concrete Roman
  - CMU Concrete Italic
  - CMU Serif Roman
  - CMU Sans Serif Oblique
  - CMU Serif Extra RomanSlanted
  - CMU Sans Serif BoldOblique
  - CMU Sans Serif Demi Condensed DemiCondensed
  - CMU Sans Serif Medium
  - CMU Sans Serif Bold
  - CMU Typewriter Text Bold
  - CMU Serif Italic
  - CMU Typewriter Text Regular
  - CMU Typewriter Text BoldItalic
  - CMU Serif Upright Italic UprightItalic
  - CMU Typewriter Text Variable Width Italic
  - CMU Typewriter Text Variable Width Medium
