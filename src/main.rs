use std::env;
use std::process::Command;
use walkdir::WalkDir;
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
    let includes = get_includes(&files, &headers);
    //todo: need to check for local files matching headers before extracting package names
    let packages = extract_names_from_headers(&includes);
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
    for entry in WalkDir::new(dirname).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if let Some(extension) = path.extension() {
            let ext = format!(".{}", extension.to_str().unwrap_or(""));
            if ext.contains(".c") || ext.contains(".h") {
                if let Ok(content) = std::fs::read_to_string(path) {
                    result.push(content);
                }
            }
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

fn download_missing_pkg_managers() {
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

fn check_existing_managers_for_pkg(pkg_name: &String) {
    let missing_managers = get_missing_pkg_managers();
    // #[cfg(target_os = "windows")]
    // let cmd = {
    //     Command::new("cmd")
    //     .args(&["/C", ""])
    //     .status()
    // };
    #[cfg(not(target_os = "windows"))]
    if !missing_managers.contains(&"pkg-config".to_string()) {
        let pkg_config_cmd = {
            Command::new("sh")
                .args(["pkg-congif", "--cflags", pkg_name])
                .status()
        };
        pkg_config_cmd.as_ref().expect("Failed to run vcpkg command.");
        match pkg_config_cmd {
            Ok(_) => return,
            Err(_) => (),
        }
    }
    if !missing_managers.contains(&"Conan".to_string()) {
        let conan_cmd = {
            Command::new("sh")
                .args(["conan", "search", pkg_name])
                .status()
        };
        conan_cmd.as_ref().expect("Failed to run vcpkg command.");
        match conan_cmd {
            Ok(_) => return,
            Err(_) => (),
        }
    }
    if !missing_managers.contains(&"vcpkg".to_string()) {
        let vcpkg_cmd = {
            Command::new("sh")
                .args(["vcpkg", "search", pkg_name])
                .status()
        };
        vcpkg_cmd.as_ref().expect("Failed to run vcpkg command.");
        match vcpkg_cmd {
            Ok(_) => return,
            Err(_) => (),
        }
    }
}

fn extract_pkg_name(include_name: &String) -> String {
    let result: String;
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
    return result;
}

