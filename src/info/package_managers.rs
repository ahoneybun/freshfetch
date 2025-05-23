use crate::mlua;

use super::kernel;
use crate::errors;

use std::path::Path;
use std::process::Command;

use mlua::prelude::*;

use crate::Inject;
use kernel::Kernel;

pub(crate) struct PackageManager {
	pub name: String,
	pub packages: i32,
}

impl PackageManager {
	pub fn new(name: &str, packages: i32) -> Self {
		PackageManager {
			name: String::from(name),
			packages: packages,
		}
	}
}

pub(crate) struct PackageManagers(Vec<PackageManager>);

impl PackageManagers {
	pub fn new(k: &Kernel) -> Self {
		let mut to_return = Vec::new();

        let usr_bin = "/run/current-system/sw/bin";
        let path = Path::new(usr_bin);

		let has_bin = |package_manager: &str| -> bool {
            if path.exists() {
            	Path::new("/run/current-system/sw/bin").join(package_manager).exists()
            } else {
            	Path::new("/usr/bin").join(package_manager).exists()
            }
		};
		let mut add = |package_manager: &str, command: &str| {
			to_return.push(PackageManager::new(package_manager, {
				let try_output = Command::new("sh")
					.arg("-c")
					.arg(&format!(r#"{}"#, command))
					.output();
				match try_output {
					Ok(output) => {
						let stdout_string = String::from_utf8(output.stdout).expect(&format!(
							"The output of \"{}\" contained invalid UTF8.",
							command
						));
						let stdout_lines: Vec<&str> = stdout_string.split("\n").collect();
						// 1 is subtracted because of the trailing
						// newline that commands have.
						stdout_lines.len() as i32 - 1
					}
					#[allow(non_fmt_panics)]
					Err(e) => panic!(format!(
						"Failed to run \"{cmd}\" Details:\n{err}",
						cmd = command,
						err = e
					)),
				}
			}))
		};

		match k.name.as_str() {
			"Linux" | "BSD" | "iPhone OS" | "Solaris" => {
				if has_bin("kiss") {
					add("kiss", "kiss l");
				}
				if has_bin("pacman") {
					add("pacman", "pacman -Qq --color never");
				}
				if has_bin("dpkg") {
					add("dpkg", "dpkg-query -f '.\n' -W");
				}
				if has_bin("rpm") {
					add("rpm", "rpm -qa");
				}
				if has_bin("xbps-query") {
					add("xbps-query", "xbps-query -l");
				}
				if has_bin("apk") {
					add("apk", "apk info");
				}
				if has_bin("opkg") {
					add("opkg", "opkg list-installed");
				}
				if has_bin("pacman-g2") {
					add("pacman-g2", "pacman-g2 -Q");
				}
				if has_bin("lvu") {
					add("lvu", "lvu installed");
				}
				if has_bin("tce-status") {
					add("tce-status", "tce-status -i");
				}
				if has_bin("pkg-info") {
					add("pkg-info", "pkg_info");
				}
				if has_bin("tazpkg") {
					add("tazpkg", "tazpkg list");
				}
				if has_bin("sorcery") {
					add("sorcery", "gaze installed");
				}
				if has_bin("alps") {
					add("alps", "alps showinstalled");
				}
				if has_bin("butch") {
					add("butch", "butch list");
				}
				if has_bin("mine") {
					add("mine", "mine -q");
				}
				if has_bin("nix-store") {
					add("nix-store", "nix-store -q --references /var/run/current-system/sw | cut -d'-' -f2-");
				}
				if has_bin("nix-env") {
					add("nix-env", "nix-env --query");
				}

				if has_bin("flatpak") {
					add("flatpak", "flatpak list");
				}
				if has_bin("snap") {
					let daemon_running = {
						let try_output = Command::new("sh")
							.arg("-c")
							.arg(r#"ps aux | grep -qFm 1 snapd"#)
							.output();
						match try_output {
							Ok(output) => output.status.success(),
							Err(_) => false,
						}
					};
					if daemon_running {
						add("snap", "snap list");
					}
				}
			}
			_ => {}
		}

		PackageManagers(to_return)
	}
}

impl Inject for PackageManagers {
	fn inject(&self, lua: &mut Lua) {
		let globals = lua.globals();

		match lua.create_table() {
			Ok(t) => {
				for (i, package_manager) in self.0.iter().enumerate() {
					match lua.create_table() {
						Ok(t2) => {
							match t2.set("name", package_manager.name.as_str()) {
								Ok(_) => (),
								Err(e) => errors::handle(&format!("{}{}", errors::LUA, e)),
							}
							match t2.set("packages", package_manager.packages) {
								Ok(_) => (),
								Err(e) => errors::handle(&format!("{}{}", errors::LUA, e)),
							}
							match t.raw_insert(i as i64 + 1, t2) {
								Ok(_) => (),
								Err(e) => errors::handle(&format!("{}{}", errors::LUA, e)),
							}
						}
						Err(e) => errors::handle(&format!("{}{}", errors::LUA, e)),
					}
				}
				match globals.set("packageManagers", t) {
					Ok(_) => (),
					Err(e) => errors::handle(&format!("{}{}", errors::LUA, e)),
				}
			}
			Err(e) => errors::handle(&format!("{}{}", errors::LUA, e)),
		}
	}
}
