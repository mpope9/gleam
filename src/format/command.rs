use crate::error::{Error, FileIOAction, FileKind, StandardIOAction};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::str::FromStr;

pub fn run(stdin: bool, check: bool, files: Vec<String>) -> Result<(), Error> {
    if stdin {
        format_stdin(check)
    } else {
        format_files(files, check)
    }
}

pub fn format_files(files: Vec<String>, _check: bool) -> Result<(), Error> {
    for file_path in files {
        let path = PathBuf::from_str(&file_path).map_err(|e| Error::FileIO {
            action: FileIOAction::Open,
            kind: FileKind::File,
            path: PathBuf::from(file_path),
            err: Some(e.to_string()),
        })?;

        if path.is_dir() {
            for path in crate::project::gleam_files(&path).into_iter() {
                format_file(&path)?;
            }
        } else {
            format_file(&path)?;
        }
    }

    Ok(())
}

fn format_file(path: &PathBuf) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .open(path.clone())
        .map_err(|e| Error::FileIO {
            action: FileIOAction::Open,
            kind: FileKind::File,
            path: path.clone(),
            err: Some(e.to_string()),
        })?;

    let mut src = String::new();
    file.read_to_string(&mut src).map_err(|e| Error::FileIO {
        action: FileIOAction::Read,
        kind: FileKind::File,
        path: path.clone(),
        err: Some(e.to_string()),
    })?;

    let formatted = crate::format::pretty(src.as_ref()).map_err(|error| Error::Parse {
        path: path.clone(),
        src: src.clone(),
        error,
    })?;

    file.seek(SeekFrom::Start(0)).unwrap();
    file.set_len(0).unwrap();

    file.write(&mut formatted.as_bytes())
        .map_err(|e| Error::FileIO {
            action: FileIOAction::WriteTo,
            kind: FileKind::File,
            err: Some(e.to_string()),
            path: path.clone(),
        })?;

    Ok(())
}

pub fn format_stdin(_check: bool) -> Result<(), Error> {
    let mut src = String::new();
    std::io::stdin()
        .read_to_string(&mut src)
        .map_err(|e| Error::StandardIO {
            action: StandardIOAction::Read,
            err: Some(e.kind()),
        })?;

    let formatted = crate::format::pretty(src.as_ref()).map_err(|error| Error::Parse {
        path: PathBuf::from("<standard input>"),
        error,
        src,
    })?;

    print!("{}", formatted);
    Ok(())
}
