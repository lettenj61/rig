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
    -Y, --confirm           Use template default value to all parameters (Yes-To-All)
    --name NAME             Specify project name (overrides default if any)
    --output PATH           Specify output directory
    --giter8                Expects that the template is a giter8 template
```

Important is `<repository>` argument, which is a location of git repo contains template.

`<repository>` should be one of the formats:
- `owner/project` style, like `lettenj61/rig`, which will expand to `https://github.com/lettenj61/rig`.
- Proper URL locates git repository, currently supports only `https` scheme.

I'm working for more documentation, so please wait for a moment.

## License
The software is dual-licensed under Apache license and MIT license, see LICENSE-Apache and LICENSE-MIT for details.


<!-- links -->
[cratesio]:https://crates.io/
[g8]:https://github.com/foundweekends/giter8
