# epub_textractor

This is a tool for extracting Japanese text from EPUB books.
It is meant for generating a base for a corpus of written Japanese.
Features:

- Outputs nicely formatted plaintext.
- Detects higher-level structures (e.g., chapters), allowing auxiliary texts
  (e.g., table of contents, copyright notices) to be skipped.
- Parses ruby annotations and outputs them separately.
- Detects gaiji images and supports annotation-based conversion to text.

## My philosophy

This is a piece of software that I write for fun, not for work.
Therefore, there are a bunch of idiosyncratic things.

- It's a script, but written in Rust simply because I enjoy it.
- The dependencies are minimal; I enjoy re-inventing the wheel, coding my own:
  - ZIP file parsing (however, [miniz_oxide](https://github.com/Frommi/miniz_oxide/) is used for DEFLATE decompression)
  - XHTML parsing
  - Hidden Markov Model-based inference (Viterbi algorithm etc.)
- Error handling is "succeed or die" style, with `死!` and `即死!` macros.
  If you don't enjoy those words, you are welcome not to look at the code.

If you use this software,
I am by no means responsible or eligible for anything you do with it,
including answering questions or technical support.
This project is licensed under MIT/Apache-2.0.
You are welcome to send pull requests,
but I make no guarantees about accepting them.

## Usage sample:

Build the tool with [Rust 1.85 or later](https://www.rust-lang.org/learn/get-started) using:

```sh
cargo build --release
```

Run the tool on a sample .epub file (not included in the repo) like this:

```sh
./target/release/epub_textractor ラノベ(サンプル文庫).epub
```

It generates the following outputs:

- `./ラノベ(サンプル文庫)/` _(directory named after the .epub file)_
- `./ラノベ(サンプル文庫)/chapters.txt` _(an index of books / chapters the .epub file contains)_
- `./ラノベ(サンプル文庫)/gaiji.txt` _(an index of gaiji; editable for fixing gaiji by annotation)_
- `./ラノベ(サンプル文庫)/gaiji_001.jpg` _(multiple image files that were used as gaiji)_
- `./ラノベ(サンプル文庫)/ラノベ.txt` _(main output, named after the inferred book name)_
- `./ラノベ(サンプル文庫)/ラノベ.ruby.yomi` _(the ruby (kanji readings) contained in the .epub)_

## TODO:

### Chapters

- ensure that spine, toc and manifest and their indices/keys are bug-free and compatible
- detect and ignore 合本版 (left: セット)
- improve chapter detection accuracy/fix bugs with current heuristics
- try HMM-based chapter detection
- try contents-based chapter detection
- try using title tag as input for book names
- try using TOC as input for book names
- try using headers as input for book names
- try using headers as input for chapter names
- re-enable 合本版, generate multiple outputs

### Contents

- output gaiji pictures for easier tagging
- fix/redesign xhtml iteration API
- improve contents detection accuracy/fix bugs with current heuristics
