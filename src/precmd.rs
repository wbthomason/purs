use ansi_term::ANSIStrings;
use ansi_term::Colour::{Blue, Green, Purple, Red, Yellow};
use clap::{App, ArgMatches, SubCommand};
use git2::{self, Oid, Remote, Repository, Revwalk, StatusOptions};
use regex::Regex;
use std::env;

fn shorten_path(cwd: &str) -> String {
    let friendly_path = match env::home_dir() {
        Some(path) => Regex::new(path.to_str().unwrap())
            .unwrap()
            .replace(cwd, "~"),
        _ => return String::from(""),
    };

    String::from(friendly_path)
}

fn repo_status(r: &Repository) -> Option<String> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    let head = match r.head() {
        Ok(head) => head,
        Err(_) => return None,
    };

    let shorthand = Green.paint(head.shorthand().unwrap().to_string());
    let statuses = match r.statuses(Some(&mut opts)) {
        Ok(statuses) => statuses,
        Err(_) => return None,
    };

    let mut is_dirty = false;
    let mut has_new = false;
    let mut has_untracked = false;
    let mut has_del = false;
    let mut has_move = false;

    for entry in statuses
        .iter()
        .filter(|e| e.status() != git2::Status::CURRENT)
    {
        let s = entry.status();
        has_new = has_new || s.contains(git2::Status::INDEX_NEW);

        has_untracked = has_untracked || s.contains(git2::Status::WT_NEW);

        has_del = has_del
            || s.contains(git2::Status::INDEX_DELETED)
            || s.contains(git2::Status::WT_DELETED);

        has_move = has_move
            || s.contains(git2::Status::INDEX_RENAMED)
            || s.contains(git2::Status::WT_RENAMED);

        is_dirty = is_dirty || match entry.status() {
            s if s.contains(git2::Status::INDEX_MODIFIED)
                || s.contains(git2::Status::INDEX_TYPECHANGE)
                || s.contains(git2::Status::WT_MODIFIED)
                || s.contains(git2::Status::WT_TYPECHANGE) =>
            {
                true
            }

            _ => false,
        };
    }

    let mut out = vec![shorthand];
    if is_dirty {
        out.push(Blue.paint("＊"));
    }

    if has_new {
        out.push(Green.bold().paint("＋"));
    }

    if has_untracked {
        out.push(Yellow.bold().paint("？"));
    }

    if has_del {
        out.push(Red.paint(" ✖"));
    }

    if has_move {
        out.push(Yellow.bold().paint("➜"))
    }

    if !is_dirty && !has_new && !has_del && !has_move && !has_untracked {
        // Clean!
        out.push(Green.bold().paint(" ✔"));
    }

    // TODO: Figure out how to check for unpushed and unmerged commits.
    // let mut has_unpushed = false;
    // let mut has_unmerged = false;

    // let remotes = match r.remotes() {
    //     Ok(names) => names,
    //     Err(_) => return None,
    // };

    // for remote in remotes.iter() {
    //     if let Some(remote) = remote {
    //         let remote_info = match r.find_remote(remote) {
    //             Ok(rem_info) => rem_info,
    //             Err(_) => return None
    //         };
    //     let mut revs = match r.revwalk() {
    //         Err(_) => return None,
    //         Ok(revs) => revs,
    //     };

    //     revs.set_sorting(git2::Sort::TIME);

    //     match revs.push_head() {
    //         Err(_) => return None,
    //         _ => (),
    //     }
    //     }
    // }

    Some(ANSIStrings(&out).to_string())
}

pub fn display(_sub: &ArgMatches) {
    let my_path = env::current_dir().unwrap();
    let display_path = Blue.paint(shorten_path(my_path.to_str().unwrap()));

    let branch = match Repository::discover(my_path) {
        Ok(repo) => repo_status(&repo),
        Err(_e) => None,
    };
    let display_branch = Green.paint(branch.unwrap_or_default());

    println!("{} {}", display_path, display_branch);
}

pub fn cli_arguments<'a>() -> App<'a, 'a> {
    SubCommand::with_name("precmd")
}
