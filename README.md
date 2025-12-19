# Rebar

**A Git clone implemented from scratch in Rust**

## Overview

Rebar is a Version Control System (VCS) similar to Git, built from the ground up using Rust. It aims to fix some of my gripes with Git while updating it to what modern systems support, like changing to SHA-256 hashes.

## Features

- **SHA-256 Hashing**: Uses SHA-256 for object hashing, reasoning below.
- **Simplified Commands**: Streamlined command set for ease of use.
- **Improved Performance**: Optimized for speed and efficiency, by redefining how diffs and patches are handled.
- **User-Friendly by Design**: Simplifies Git ideas for better usability.

## Download

Pre-built binaries are available as artifacts from the latest [GitHub Actions workflow runs](https://github.com/actuallyatiger/rebar/actions/workflows/ci-cd.yml?query=branch%3Amain+event%3Apush), which track the `main` branch.

Eventually I'll get around to proper version releases.

## Building from Source

To build Rebar from source, ensure you have Rust installed. Then clone the repository and build the project:

```bash
cd rebar
cargo build --release
```

Then add the binary to your PATH:

```bash
export PATH="$PATH:$(pwd)/target/release"
```

Put it in your `.bashrc`, `.zshrc`, or equivalent shell configuration file to make it permanent.

## FAQ

### Why SHA-256?

Linus Torvalds rejected switching from SHA-1 in [this mailing list post](https://public-inbox.org/git/Pine.LNX.4.58.0504291221250.18901@ppc970.osdl.org/), but that was back in 2005 and a lot in cryptography has changed since then. Since then, they have [resolved to move to SHA-256](https://marc.info/?l=git&m=148787042322920&w=2) but AFAIK this is still an opt-in feature at repo initialisation. That brings me on to my next point:

Git has a lot of legacy baggage. It needs to support old versions of itself, so when building a modern VCS from scratch we might as well take advantage of having a fresh start.

As for why 256-bit specifically, I figured it would be a good balance of hash length while still being small enought for objects to fit inside modern file system path length limits. Storing them as hex strings gives 64 characters, which is very manageable.

### Project Plans

First I want the core functionality, which will take the shape of the underlying commands like `hash-object`, `cat-file`, etc. This means the underlying storage system will need to be implemented. From there, build up to user-facing commands like `commit`, `branch`, `merge`, etc.

Once it's in a usable state, I'll look into distribution functionality like remotes, pushing, pulling, etc.
