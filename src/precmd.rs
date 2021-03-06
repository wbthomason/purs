use ansi_term::ANSIStrings;
use ansi_term::Colour::{Blue, Cyan, Green, Red, Yellow};
use clap::{App, ArgMatches, SubCommand};
use git2::{self, Repository, StatusOptions};
use std::env;

fn shorten_path(cwd: &str) -> String {
  match dirs::home_dir() {
    Some(path) => cwd.replacen(path.to_str().unwrap(), "~", 1),
    _ => String::new(),
  }
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

    has_del =
      has_del || s.contains(git2::Status::INDEX_DELETED) || s.contains(git2::Status::WT_DELETED);

    has_move =
      has_move || s.contains(git2::Status::INDEX_RENAMED) || s.contains(git2::Status::WT_RENAMED);

    is_dirty = is_dirty
      || match entry.status() {
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

  // Adapted from @Kurt-Bonatz in https://github.com/alexcrichton/git2-rs/issues/332#issuecomment-405623972
  let head_ref = r.revparse_single("HEAD").unwrap().id();
  let (is_ahead, is_behind) = r
    .revparse_ext("@{u}")
    .ok()
    .and_then(|(upstream, _)| r.graph_ahead_behind(head_ref, upstream.id()).ok())
    .map(|(ahead, behind)| (ahead > 0, behind > 0))
    .unwrap_or((false, false));

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
    out.push(Red.bold().paint("Ｘ"));
  }

  if has_move {
    out.push(Yellow.bold().paint("➜"))
  }

  let is_clean = !is_dirty && !has_new && !has_untracked && !has_del && !has_move;
  if is_clean {
    // Clean!
    out.push(Green.bold().paint(" ✔"));
  }

  if is_ahead {
    let spacer = if is_clean || has_del || has_move {
      " "
    } else {
      ""
    };
    out.push(Cyan.paint(spacer.to_owned() + "↑"));
  }

  if is_behind {
    let spacer = if !is_ahead && (is_clean || has_move) {
      " "
    } else {
      ""
    };
    out.push(Cyan.paint(spacer.to_owned() + "↓"));
  }

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
