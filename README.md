# Rig

A tool to generate new project from templates hosted in git repository, is mostly a port to Scala's [giter8][g8]

[![Build Status](https://travis-ci.org/lettenj61/rig.svg?branch=master)](https://travis-ci.org/lettenj61/rig)

__IMPORTANT__ Currently this tool is under early development stage, and not all the tests provided,
so unimplemented features & bugs everywhere.

## Installation

I haven't published it to [Crates.io][cratesio], anyway Cargo can download it from github repo.

_NOTE_: You need to install `CMake` first, as `cargo install` will build the binary from scratch, and git2-rs (one of our dependency) needs that.

```
cargo install --git https://github.com/lettenj61/rig
```

## Usage

If installed successfully, this usage text spams your console when you hit `rig -h`

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

Important thing is `<repository>` argument, which locates the git repo contains template.

`<repository>` should be one of these formats:
- `owner/project` format, like `lettenj61/rig`, which will be expanded to `https://github.com/lettenj61/rig`.
- Valid URL of git repository, currently supports only `https` scheme.

The example output generating new project:

```
lettenj61:~/sandbox $ rig typesafehub/scala-sbt.g8 --giter8
description [Scala Project Using sbt]:Some project made with Rig
name [Scala Project]:My First Project
version [0.1-SNAPSHOT]:0.1.0
scala_version [2.10.2]:2.11.8
organization [org.example]:com.github.myself
Project successfully generated: "/home/ubuntu/sandbox/my-first-project"
lettenj61:~/sandbox $ tree ./my-first-project/
./my-first-project/
├── README
├── project
│   └── MyFirstProjectBuild.scala
└── src
    └── main
        └── scala
            └── com
                └── github
                    └── myself
                        └── MyFirstProject.scala

7 directories, 3 files
```

I'm working for more documentation, so please wait for a moment.

## License
The software is dual-licensed under Apache license and MIT license, see LICENSE-APACHE and LICENSE-MIT for details.


<!-- links -->
[cratesio]:https://crates.io/
[g8]:https://github.com/foundweekends/giter8
