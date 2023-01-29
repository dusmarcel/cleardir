use std::fs;
use::std::io;
use std::path::PathBuf;
use clap;
use ansi_term;
use sha2::{Sha256, Digest};
use std::collections::HashMap;

fn main() {
	let matches = clap::Command::new("cleardir")
		.version("0.1.0")
		.author("Marcel Keienborg <marcel@keienb.org>")
		.about("cleardir compares the SHA256 checksums of the files in the specified directories. If multiple files with the same checksum are found in a directory, all files except the one with the shortest filename are deleted. CAUTION: Files will be deleted without further inquiry. Use at your own risk.")
		.arg(clap::Arg::new("verbose")
			.short('v')
			.long("verbose")
			.action(clap::ArgAction::SetTrue)
			.help("more detailed output"))
		.arg(clap::Arg::new("dry-run")
			.short('d')
			.long("dry-run")
			.action(clap::ArgAction::SetTrue)
			.help("Test run: no files will be deleted"))
		.arg(clap::Arg::new("paths")
			.action(clap::ArgAction::Append)
			.help("The paths to be searched"))
		.try_get_matches();
	
	match matches {
		Err(e) => match e.kind() {
			clap::error::ErrorKind::DisplayHelp |
			clap::error::ErrorKind::DisplayVersion => println!("{}", e),
			_ => eprintln!("{} {}",
					ansi_term::Colour::Red.paint("An error occurred while parsing arguments:"),
					ansi_term::Colour::Red.paint(e.to_string())),
		}
		
		Ok(matches) => {
			let v = *matches.get_one::<bool>("verbose").unwrap_or(&false);
			let d = *matches.get_one::<bool>("dry-run").unwrap_or(&false);
			let p = matches.get_many::<String>("paths")
				.unwrap_or_default()
				.map(|v| v.as_str())
				.collect::<Vec<_>>();
			
			if v { print_arguments (&v, &d, &p) };
			
			if p.len() == 0 { println!("No paths given. Exiting."); }
			else {
				for i in p {
					match list_dir(&v, &d, i) {
						Err(e) => eprintln!("{}: {}",
							ansi_term::Colour::Red.paint("Error:"),
							ansi_term::Colour::Yellow.paint(e.to_string())),
						_ => (),
					}
				}
			}
		},
	}	
}

fn list_dir (v: &bool, d: &bool, path: &str) -> Result<(), io::Error> {
    match fs::read_dir(path) {
		Err(e) => eprintln!("Error: {:?}", e.kind()),
		Ok(entries) => {
			println!("{} {}",
				ansi_term::Colour::Green.paint("Searching for duplicates in directory:"),
				ansi_term::Colour::Green.bold().paint(path)
			);
			
			let mut files = HashMap::<String, Vec<PathBuf>>::new();
			let mut p: PathBuf;
			let mut hash: String;
						
			for entry in entries {
				match entry {
					Ok(entry) => {
						p = entry.path();
						
						if !p.is_dir() {
							let mut file = fs::File::open(&p)?;
							let mut hasher = Sha256::new();
							io::copy(&mut file, &mut hasher)?;
							hash = format!("{:x}", hasher.finalize());
							print!("{} => {}",
								p.display(),
								hash
							);
							if files.contains_key(&hash) { 
								print!("({}", ansi_term::Colour::Red.paint(" (dup!) "));
								if let Some(v) = files.get_mut(&hash) { v.push(p); }
							} else {
							files.insert(hash, vec![p]);
							}
							println!();
						} else if *v {
							println!("(ignoring directory: {})", p.display());
						}
					},
					Err(e) => eprintln!("Error: {:?}", e.kind()),
				}
			}
			if *v { println!("{:#?}", files); }
			match remove_files(&v, &d, files) {
				Err(e) => eprintln!("Error: {:?}", e.kind()),
				_ => (),
			}
		}
	}
	Ok(())
}

fn remove_files(v: &bool, d: &bool, files: HashMap<String, Vec<PathBuf>>) -> Result<(), io::Error> {
	for vc in files.values() {
		if vc.len() > 1 {
			let mut l: usize = 0;
			let mut dup_del = false;
			
			for i in 0..vc.len() {
				match vc[i].to_str() {
					None => continue,
					Some(s) => {
						let il = s.len();
						
						if i == 0 { l = il; }
						else if il < l { l = il; }
					}
				}
			}
			
			for i in 0..vc.len() {
				match vc[i].to_str() {
					None => continue,
					Some(s) => {
						let il = s.len();
						
						if il > l || (il == l && dup_del) {
							if *v { println!("I want to delete {}", s) };
							//if !*d { println!("fs::remove_file({});", vc[i].display()); }
							if !*d { fs::remove_file(&vc[i])?; }
						} else if il == l { dup_del = true; }
					}
				}
			}
		}
	}
	Ok(())
}

fn print_arguments (v: &bool, d: &bool, p: &Vec<&str>) {
	println!("{} {}",
		ansi_term::Colour::Green.paint("Command line option \"verbose\" set?"),
		ansi_term::Colour::Yellow.paint(v.to_string()));
	println!("{} {}",
		ansi_term::Colour::Green.paint("Command line option \"dry-run\" set?"),
		ansi_term::Colour::Yellow.paint(d.to_string()));
	println!("{} {:?}",
		ansi_term::Colour::Green.paint("Which paths to search?"), p);
}		
