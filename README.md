# YouTube Summarizer

Creates a brief summary of a YouTube video so that you can decide on watching

## What it's good for

Good for checking if you'd be interested in watching

- getting a high level overview of content to see if you know it well already
- finding out why some influencer hates Rust for some reason and then leaving

## What it's not for

The purpose of this project is **not** to

- plagiarize videos with AI slop
- replace the original content. If the summary looks good, the original content can give you much more details (and less AI hallucinations 😛)

## Development

### Dependencies

| Dependency  | Installation                                                             |
| ----------- | ------------------------------------------------------------------------ |
| Rust        | [Install Rust](https://doc.rust-lang.org/book/ch01-01-installation.html) |
| cargo-watch | `cargo install cargo-watch`                                              |
| just        | `cargo install just`                                                     |
| npm         | [Install NodeJS](https://nodejs.org/en/download)                         |
| lefthook    | [Install lefthook](https://lefthook.dev/installation/index.html)         |

### Commands

run `just --list` to see a list of available commands

### Example workflow

`just dev` and `just example` can be run side by side to build an run the server then make an example request against it
