# Rig

A toolkit to generate new project from templates hosted in git repository. Mostly a port to Scala's [giter8][g8]

__IMPORTANT__ Currently this tool is under early development stage, and not all the tests provided,
so unimplemented features & bugs everywhere.

## Installation

I haven't published it to [Crates.io][cratesio], Cargo can download it via github repos.

_NOTE_: You need `CMake` to be installed, as `cargo install` will build the binary from scratch.

```
cargo install --git cargo install --git https://github.com/lettenj61/rig
```

## Usage

Successfully installed, this usage text spams your console when you hit `rig -h`

```
Rig - Generate new project by cloning templates from git repository.

*NOTE* This software is under early development, most of its features are not yet supported:
  - Currently it can only use templates that hosted on GitHub
  - giter8 compatibility features (e.g. maven directive) are not implemented.

Usage:
    rig <repository> [options]
    rig (-h | --help)
    rig (-V | --version)

Options:
    -h, --help              Show help message
    -V, --version           Show version
    --name NAME             Specify project name (overrides default if any)
    --output PATH           Specify output directory
    --package PATH          Specify project directory tree (mainly used in giter8 project)
    --verbatim EXTENSION    Space separeted list of file exts exclude from template processing
    -Y, --confirm           Use template default value to all parameters (Yes-To-All)
    --dry-run               Show generation process to STDOUT, without producing any files
    --giter8                Expects that the template is a giter8 template
    --no-logo               Supress logo
```

I'm working for more documentation, so please wait for a moment.

## License
The software is dual-licensed under Apache license and MIT license, see LICENSE-Apache and LICENSE-MIT for details.


<!-- links -->
[cratesio]:https://crates.io/
[g8]:https://github.com/foundweekends/giter8
