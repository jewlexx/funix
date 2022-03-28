extern crate directories;
extern crate git2;
extern crate spinners;

use directories::UserDirs;
use git2::Repository;
use spinners::{Spinner, Spinners};
use std::{
    fs,
    path::Path,
    process::{exit, Command},
};

#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::symlink;
#[cfg(target_os = "windows")]
use std::{io, path::PathBuf};
#[cfg(target_os = "windows")]
fn symlink(from: &PathBuf, to: &PathBuf) -> io::Result<()> {
    match fs::copy(from, to) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(target_os = "windows")]
const EXEC_NAME: &str = "flutter.bat";
#[cfg(not(target_os = "windows"))]
const EXEC_NAME: &str = "flutter";

const SPINNER_VARIANT: Spinners = Spinners::Dots;
const FLUTTER_URL: &str = "https://github.com/flutter/flutter.git";

fn check_dir(dir: &Path) -> Result<(), &Path> {
    if fs::create_dir(dir).is_err() && !dir.exists() {
        Err(dir)
    } else {
        Ok(())
    }
}

macro_rules! soft_panic {
    ($message:expr) => {
        println!("{}", $message);
        exit(1);
    };
}

fn main() {
    let user_dirs = match UserDirs::new() {
        Some(v) => v,
        None => {
            soft_panic!("Ran into an error getting your user directories :(");
        }
    };

    let tools_dir = user_dirs.home_dir().join(".tools");
    let flutter_dir = tools_dir.join("flutter");
    let bin_dir = tools_dir.join("bin");

    let dirs_vec = vec![&tools_dir, &flutter_dir, &bin_dir];

    // Checks that all the dirs either exist, or can be created.
    for dir in dirs_vec {
        match check_dir(dir) {
            Ok(_) => (),
            Err(dir) => {
                println!("Something wen't wrong. I'm not 100% sure what sorry.");
                println!(
                    "Check if {} already exists, and if it doesn't, try creating it yourself.",
                    dir.display()
                );
            }
        };
    }

    let sp = Spinner::new(SPINNER_VARIANT, "Cloning Flutter Repo...".into());
    let repo = Repository::clone(FLUTTER_URL, &flutter_dir);
    sp.stop_with_symbol("\x1b[32m🗸\x1b[0m");
    println!();

    if repo.is_err() {
        soft_panic!("Failed to clone Flutter repo! Check if the directory already exists");
    }
    

    let flutter_bin = {
        let mut flutter_bin = flutter_dir.clone();
        flutter_bin.push("bin");
        flutter_bin
    };

    #[cfg(target_os = "windows")]
    println!("Please note that symlinking is not supported on Windows, so the files have simply been copied\nThis means that if you wish to pull a new commit from the git repository, you must copy the bin files again");

    for bin_file in match fs::read_dir(&flutter_bin) {
        Ok(v) => v,
        Err(_) => {
            soft_panic!("Ran into an error scanning the Flutter bin directory. Not really sure what happened. I'll look into it :)");
        }
    } {
        let path = match bin_file {
            Ok(b) => b.path(),
            Err(_) => {
                continue;
            }
        };

        if !path.is_dir() {
            let file_name = match path.file_name() {
                Some(v) => v,
                None => {
                    continue;
                }
            };

            if symlink(&path, &bin_dir.join(file_name)).is_err() {
                println!(
                    "WARN: Failed to symlink {}",
                    file_name.to_str().unwrap_or("NONE")
                );
            }
        }
    }

    let sp = Spinner::new(SPINNER_VARIANT, "Running First Time Setup".into());
    match Command::new(bin_dir.join(EXEC_NAME).as_os_str()).output() {
        Ok(_) => sp.stop_with_message("Ran first time setup!\n".into()),
        Err(_) => sp.stop_with_message("Ran into an error running first time setup\n".into()),
    };

    println!("Finished Setting up Flutter sdk :)");
    println!("Now all you have to do is add the following directory to your path: ");
    println!("{}", bin_dir.display());
}
