//! A program which prints the full path to the Git binary if it can be found.

extern crate find_git;

fn main() {
    match find_git::git_path() {
        Some(git) => println!("Git is located at {:?}.", git),
        None => println!("Failed to locate Git."),
    }
}
