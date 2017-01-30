/// File system utilities copied from rustup-utils crate.
/// https://github.com/rust-lang-nursery/rustup.rs/tree/master/src/rustup-utils

use std::fs;
use std::io;
use std::path::Path;
use std::thread;
use std::time::Duration;

pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    fs::metadata(path).is_ok()
}

pub fn is_directory<P: AsRef<Path>>(path: P) -> bool {
    fs::metadata(path).ok().as_ref().map(fs::Metadata::is_dir) == Some(true)
}

pub fn read_file(path: &Path) -> io::Result<String> {
    let mut file = try!(fs::OpenOptions::new()
        .read(true)
        .open(path));

    let mut contents = String::new();
    try!(io::Read::read_to_string(&mut file, &mut contents));
    Ok(contents)
}

pub fn write_file(path: &Path, contents: &str) -> io::Result<()> {
    let mut file = try!(fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path));

    try!(io::Write::write_all(&mut file, contents.as_bytes()));
    try!(file.sync_data());
    Ok(())
}

pub fn copy_dir(src: &Path, dest: &Path) -> io::Result<()> {
    try!(fs::create_dir(dest));
    for entry in try!(src.read_dir()) {
        let entry = try!(entry);
        let kind = try!(entry.file_type());
        let src = entry.path();
        let dest = dest.join(entry.file_name());
        if kind.is_dir() {
            try!(copy_dir(&src, &dest));
        } else {
            try!(fs::copy(&src, &dest));
        }
    }
    Ok(())
}

pub fn remove_dir(path: &Path) -> io::Result<()> {
    if try!(fs::symlink_metadata(path)).file_type().is_symlink() {
        if cfg!(windows) {
            fs::remove_dir(path)
        } else {
            fs::remove_file(path)
        }
    } else {
        let mut result = Ok(());

        for _ in 0..5 {
            result = rm_rf(path);
            if !is_directory(path) {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(16));
        }
        result
    }
}

fn rm_rf(path: &Path) -> io::Result<()> {
    if path.exists() {
        for file in fs::read_dir(path).unwrap() {
            let file = try!(file);
            let is_dir = try!(file.file_type()).is_dir();
            let ref file = file.path();

            if is_dir {
                try!(rm_rf(file));
            } else {

                match fs::remove_file(file) {
                    Ok(()) => {}
                    Err(ref e) if cfg!(windows) && e.kind() == io::ErrorKind::PermissionDenied => {
                        let mut p = file.metadata().unwrap().permissions();
                        p.set_readonly(false);
                        fs::set_permissions(file, p).unwrap();
                        try!(fs::remove_file(file));
                    }
                    Err(e) => return Err(e),
                }
            }
        }
        fs::remove_dir(path)
    } else {
        Ok(())
    }
}
