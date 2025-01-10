use std::process::Command;
use std::str;

pub fn check_manager_for_pkg(manager: &String, pkg_name: &String) -> bool {
    #[cfg(not(target_os = "windows"))]
    return match manager.as_str() {
        "pkg-config" => check_pkg_config_for_pkg_unix(pkg_name),
        "Conan" => check_conan_for_pkg_unix(pkg_name),
        "vcpkg" => check_vcpkg_for_pkg_unix(pkg_name),
        _ => false,
    };
}

fn check_vcpkg_for_pkg_unix(pkg_name: &String) -> bool {
    println!("Checking vcpkg for '{}'...", &pkg_name);
    let vcpkg_cmd = {
        Command::new("./vcpkg/vcpkg")
            .args(&["search", pkg_name])
            .output() 
    };
    match vcpkg_cmd {
        Ok(output) => {
            if output.status.success() {
                if let Ok(stdout) = str::from_utf8(&output.stdout[..]) {
                    if stdout.contains(pkg_name) { return true; } 
                } 
            } 
        }
        Err(_) => eprintln!("Failed to run vcpkg command."),
    }
    return false;
}

fn check_conan_for_pkg_unix(pkg_name: &String) -> bool {
    println!("Checking Conan for '{}'...", &pkg_name);
    let conan_cmd = {
        Command::new("conan")
            .args(["search", pkg_name])
            .output()
    };
    match conan_cmd {
        Ok(cmd) if cmd.status.success() => {
            let res = cmd.stdout;
            let mut message = String::new();
            for item in res { message.push(item as char); }
            if message.contains("ERROR") { return false; }
            return true;
        },
        Ok(_) => (), 
        Err(_) => eprintln!("Failed to run Conan command."),
    }
    return false;
}

fn check_pkg_config_for_pkg_unix(pkg_name: &String) -> bool {
    println!("Checking pkg-config for '{}'...", &pkg_name);
    let pkg_config_cmd = {
        Command::new("pkg-config")
            .args(&["--cflags", pkg_name])
            .output()
    };
    match pkg_config_cmd {
        Ok(cmd) if cmd.status.success() => return true,
        Ok(_) => (), 
        Err(_) => eprintln!("Failed to run pkg-config command."),
    }
    return false;
}
