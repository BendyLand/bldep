use std::process::Command;
use which::which;

pub fn get_missing_pkg_managers() -> Vec<String> {
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

pub fn get_installed_pkg_managers() -> Vec<String> {
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

pub fn install_missing_pkg_managers() {
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
    if !is_conan_installed() {
        let status = {
            Command::new("pip")
                .args(&["install", "--upgrade", "--force-reinstall", "conan"])
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

fn is_conan_installed() -> bool {
    let output = Command::new("conan").arg("--version").output();
    return match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    };
}
