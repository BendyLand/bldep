use std::env;
use std::process::Command;
use std::str;
use which::which;

fn main() {
    let header_str = include_str!("headers.txt");
    let args: Vec<String> = env::args().collect();
    let dirname: String;
    if args.len() > 1 { dirname = args[1].clone(); }
    else { dirname = ".".to_string(); }
    let headers: Vec<String> = {
        header_str
            .to_string()
            .lines()
            .map(|x| x.to_string())
            .collect()
    };
    let files = walk_dir(&dirname);
    let mut includes = get_includes(&files, &headers);
    let local_files = check_for_local_files(&dirname);
    includes = remove_local_files_from_includes(&includes, &local_files);
    let packages = extract_names_from_headers(&includes);
    let installed_managers = get_installed_pkg_managers();
    let mut found_packages = Vec::<(String, String)>::new();
    if installed_managers.len() > 0 {
        for manager in installed_managers.clone() {
            for package in packages.clone() {
                let res = check_manager_for_pkg(&manager, &package);
                if res { found_packages.push((manager.clone(), package)); }
            }
            println!("");
        }
    }
    let managers = get_missing_pkg_managers();
    if installed_managers.len() < 3 { download_missing_pkg_managers(); }
    for manager in managers {
        for package in packages.clone() {
            let res = check_manager_for_pkg(&manager, &package);
            if res { found_packages.push((manager.clone(), package)); }
        }
        println!("");
    }
    for entry in found_packages.iter() {
        println!("'{}' found with {}!", entry.1, entry.0);
    }
    println!("");
    for package in packages {
        if !found_packages.iter().map(|x| x.1.clone()).collect::<Vec<String>>().contains(&package) {
            println!("'{}' not found.", &package);
        }
    }
    //todo: check package manager outputs for messages like "Did you mean..."
}

fn get_includes(files: &Vec<String>, headers: &Vec<String>) -> Vec<String> {
    let mut result = Vec::<String>::new();
    for file in files {
        let lines = file.lines();
        for line in lines {
            if line.contains("#include") {
                result.push(extract_header_name(&line));
            }
        }
    }
    result = remove_stdlib_headers(&result, &headers);
    return result;
}

fn extract_header_name(line: &str) -> String {
    let result: String;
    if let Some(start) = line.find("<") {
        let end = line.find(">").unwrap();
        result = line[start+1..end].to_string();
    }
    else {
        let start = line.find("\"").unwrap();
        let end = line.rfind("\"").unwrap();
        result = line[start+1..end].to_string();
    }
    return result;
}

fn remove_stdlib_headers(includes: &Vec<String>, headers: &Vec<String>) -> Vec<String> {
    let mut result: Vec<String> = {
        includes
            .into_iter()
            .filter(|x| !headers.contains(x))
            .map(|x| x.to_string())
            .collect()
    };
    result.sort();
    result.dedup();
    return result;
}

fn walk_dir(dirname: &String) -> Vec<String> {
    let mut result = Vec::<String>::new();
    if let Ok(entries) = std::fs::read_dir(dirname) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                let subdir_contents = walk_dir(&path.to_string_lossy().to_string());
                result.extend(subdir_contents);
            } 
            else if let Some(extension) = path.extension() {
                let ext = format!(".{}", extension.to_str().unwrap_or(""));
                if ext.contains(".c") || ext.contains(".h") {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        result.push(content);
                    }
                }
            }
        }
    }
    return result;
}

fn check_for_local_files(dirname: &String) -> Vec<String> {
    let mut result = Vec::<String>::new();
    if let Ok(entries) = std::fs::read_dir(dirname) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                let subdirs = walk_dir(&path.to_string_lossy().to_string());
                result.extend(subdirs);
            } 
            else if let Some(extension) = path.extension() {
                let ext = format!(".{}", extension.to_str().unwrap_or(""));
                if ext.contains(".c") || ext.contains(".h") {
                    result.push(path.to_str().unwrap().to_string());
                }
            }
        }
    }
    for i in 0..result.len() {
        if result[i].contains("/") {
            let idx = result[i].rfind("/").unwrap();
            result[i] = result[i][idx+1..].to_string();
        }
    }
    return result;
}

fn get_missing_pkg_managers() -> Vec<String> {
    let names: Vec<String> = vec!["vcpkg", "Conan", "pkg-config"].into_iter().map(|x| x.to_string()).collect();
    let mut not_installed = Vec::<String>::new();
    for name in names {
        match which(name.clone()) {
            Ok(_) => (),
            Err(_) => not_installed.push(name),
        }
    }
    if not_installed.contains(&"vcpkg".to_string()) {
        if std::path::Path::new("vcpkg").exists() {
            let idx = not_installed.iter().position(|x| x == "vcpkg").unwrap();
            not_installed.remove(idx);
        }
    }
    return not_installed;
}

fn get_installed_pkg_managers() -> Vec<String> {
    let names: Vec<String> = vec!["vcpkg", "Conan", "pkg-config"].into_iter().map(|x| x.to_string()).collect();
    let mut installed = Vec::<String>::new();
    for name in names {
        match which(name.clone()) {
            Ok(_) => installed.push(name),
            Err(_) => (),
        }
    }
    if !installed.contains(&"vcpkg".to_string()) {
        if std::path::Path::new("vcpkg").exists() {
            installed.push("vcpkg".to_string());
        }
    }
    return installed;
}

fn download_missing_pkg_managers() {
    println!("Downloading missing package managers...");
    let missing_managers = get_missing_pkg_managers();
    for manager in missing_managers {
        match manager.as_str() {
            "vcpkg" => install_vcpkg(),
            "Conan" => install_conan(),
            "pkg-config" => install_pkg_config(),
            _ => continue,
        }
    }
}

fn install_vcpkg() {
    let repo_url = "https://github.com/microsoft/vcpkg.git";
    let target_dir = "vcpkg";
    if !std::path::Path::new(target_dir).exists() {
        println!("Cloning vcpkg...");
        Command::new("git")
            .args(&["clone", repo_url, target_dir])
            .status()
            .expect("Failed to clone vcpkg");
        println!("Bootstrapping vcpkg...");
        #[cfg(target_os = "windows")]
        let bootstrap = {
            Command::new("cmd")
                .args(&["/C", format!("{}/bootstrap-vcpkg.bat", target_dir)])
                .status()
        };
        #[cfg(not(target_os = "windows"))]
        let bootstrap = {
            Command::new("sh")
                .arg(format!("{}/bootstrap-vcpkg.sh", target_dir))
                .status()
        };
        bootstrap.expect("Failed to bootstrap vcpkg");
        println!("vcpkg installed successfully.\n");
    } 
    else {
        println!("vcpkg is already installed.\n");
    }
}

fn install_conan() {
    let output = {
        Command::new("pip")
            .arg("--version")
            .output()
            .expect("Failed to check for pip")
    };
    if !output.status.success() {
        eprintln!("Python and pip are required to install Conan. Please install them first.");
        return;
    }
    println!("Installing Conan...");
    let status = {
        Command::new("pip")
            .args(&["install", "--user", "conan"])
            .status()
            .expect("Failed to install Conan")
    };
    if status.success() {
        println!("Conan installed successfully.\n");
    } 
    else {
        eprintln!("Failed to install Conan.\n");
    }
}

fn install_pkg_config() {
    #[cfg(target_os = "linux")] {
        println!("Installing pkg-config via apt...");
        let status = {
            Command::new("sudo")
                .args(&["apt-get", "install", "-y", "pkg-config"])
                .status()
                .expect("Failed to install pkg-config")
        };
        if status.success() {
            println!("pkg-config installed successfully.\n");
        } 
        else {
            eprintln!("Failed to install pkg-config.\n");
        }
    }
    #[cfg(target_os = "macos")] {
        println!("Installing pkg-config via Homebrew...");
        let status = {
            Command::new("brew")
                .args(&["install", "pkg-config"])
                .status()
                .expect("Failed to install pkg-config")
        };
        if status.success() {
            println!("pkg-config installed successfully.\n");
        } 
        else {
            eprintln!("Failed to install pkg-config.\n");
        }
    }
    #[cfg(target_os = "windows")] {
        eprintln!("pkg-config installation on Windows is not yet automated in this script.\n");
    }
}

fn check_manager_for_pkg(manager: &String, pkg_name: &String) -> bool {
    // #[cfg(target_os = "windows")]
    // let cmd = {
    //     Command::new("cmd")
    //     .args(&["/C", ""])
    //     .status()
    // };
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
                    // else { eprintln!("Package '{}' not found in vcpkg.", pkg_name); }
                } 
                // else { eprintln!("Failed to parse vcpkg output."); }
            } 
            // else { eprintln!("vcpkg command failed with error.") }
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
        Ok(cmd) if cmd.status.success() => return true,
        Ok(_) => (), // eprintln!("Conan failed to find the package: '{}'", pkg_name),
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
        Ok(_) => (), // eprintln!("pkg-config failed to find the package: '{}'", pkg_name),
        Err(_) => eprintln!("Failed to run pkg-config command."),
    }
    return false;
}

fn extract_pkg_name(include_name: &String) -> String {
    let mut result: String;
    if include_name.contains("/") {
        let idx = include_name.find("/").unwrap();
        let new_str = include_name[..idx].to_string();
        if ends_with_any(&new_str, &"0123456789".to_string()) {
            result = new_str[..find_first_of_any(&new_str, &"0123456789".to_string())].to_string();
        }
        else { result = new_str; }
    }
    else if include_name.contains(".") {
        let idx = include_name.find(".").unwrap();
        let new_str = include_name[..idx].to_string();
        if ends_with_any(&new_str, &"0123456789".to_string()) {
            result = new_str[..find_first_of_any(&new_str, &"0123456789".to_string())].to_string();
        }
        else { result = new_str; }
    }
    else {
        if ends_with_any(&include_name, &"0123456789".to_string()) {
            result = include_name[..find_first_of_any(&include_name, &"0123456789".to_string())].to_string();
        }
        else { result = include_name.to_string(); }
    }
    result = result.to_lowercase().to_string();
    return result;
}

fn ends_with_any(s: &String, cs: &String) -> bool {
    let checks: Vec<char> = cs.chars().collect();
    for check in checks { if s.ends_with(check) { return true; } }
    return false;
}

fn find_first_of_any(s: &String, cs: &String) -> usize {
    let checks: Vec<char> = cs.chars().collect();
    let mut result = usize::MAX;
    for check in checks { 
        if let Some(idx) = s.find(check) { 
            if idx < result { result = idx; }
        } 
    }
    return result;
}

fn extract_names_from_headers(headers: &Vec<String>) -> Vec<String> {
    let mut result = Vec::<String>::new();
    for header in headers {
        let temp = extract_pkg_name(header);
        result.push(temp);
    }
    result.sort();
    result.dedup();
    return result;
}

fn remove_local_files_from_includes(includes: &Vec<String>, local_files: &Vec<String>) -> Vec<String> {
    let result = includes.into_iter().filter(|x| {
        if x.contains("/") {
            let idx = x.rfind("/").unwrap();
            let temp = x[idx+1..].to_string();
            return !local_files.contains(&temp);
        }
        else {
            return !local_files.contains(x);
        }
    }).map(|x| x.to_owned()).collect::<Vec<String>>();
    return result;
}

