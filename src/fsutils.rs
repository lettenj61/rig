use std::fs;
use std::io;
use std::path::Path;

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
