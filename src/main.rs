use std::env;
use std::str;

mod installer;
mod manager;
mod utils;

fn main() {
    let header_str = include_str!("headers.txt");
    let args: Vec<String> = env::args().collect();
    let dirname = if args.len() > 1 { args[1].clone() } else { ".".to_string() };
    let headers: Vec<String> = header_str.to_string().lines().map(|x| x.to_string()).collect();
    let files = walk_dir(&dirname);
    let includes = remove_local_files_from_includes(
        &get_includes(&files, &headers), 
        &check_for_local_files(&dirname),
    );
    let packages = extract_names_from_headers(&includes);
    let missing_managers = installer::get_missing_pkg_managers();
    let installed_managers = installer::get_installed_pkg_managers();
    if installed_managers.len() < 3 { installer::install_missing_pkg_managers(); }
    let found_packages = find_packages(&installed_managers, &packages);
    let additional_found_packages = find_packages(&missing_managers, &packages);
    let all_found_packages = [found_packages, additional_found_packages].concat();
    report_packages(&all_found_packages, &packages);
    if missing_managers.contains(&"vcpkg".to_string()) { 
        let res = installer::remove_vcpkg(); 
        if res { println!("vcpkg removed successfully!"); }
    }
    //todo: check package manager outputs for messages like "Did you mean..."
}

fn get_includes(files: &Vec<String>, headers: &Vec<String>) -> Vec<String> {
    let mut result = Vec::<String>::new();
    for file in files {
        let lines = file.lines();
        for line in lines {
            if line.starts_with("#include ") {
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
                let subdirs = check_for_local_files(&path.to_string_lossy().to_string());
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

fn extract_pkg_name(include_name: &String) -> String {
    let mut result: String;
    if include_name.contains("/") {
        let idx = include_name.find("/").unwrap();
        let new_str = include_name[..idx].to_string();
        if utils::ends_with_any(&new_str, &"0123456789".to_string()) {
            result = new_str[..utils::find_first_of_any(&new_str, &"0123456789".to_string())].to_string();
        }
        else { result = new_str; }
    }
    else if include_name.contains(".") {
        let idx = include_name.find(".").unwrap();
        let new_str = include_name[..idx].to_string();
        if utils::ends_with_any(&new_str, &"0123456789".to_string()) {
            result = new_str[..utils::find_first_of_any(&new_str, &"0123456789".to_string())].to_string();
        }
        else { result = new_str; }
    }
    else {
        if utils::ends_with_any(&include_name, &"0123456789".to_string()) {
            result = include_name[..utils::find_first_of_any(&include_name, &"0123456789".to_string())].to_string();
        }
        else { result = include_name.to_string(); }
    }
    result = result.to_lowercase().to_string();
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

fn find_packages(installed_managers: &Vec::<String>, packages: &Vec::<String>) -> Vec<(String, String)> {
    let mut found_packages = Vec::<(String, String)>::new();
    if installed_managers.len() > 0 {
        for manager in installed_managers {
            for package in packages {
                let res = manager::check_manager_for_pkg(&manager, &package);
                if res { found_packages.push((manager.clone(), package.to_owned())); }
            }
            println!("");
        }
    }
    return found_packages;
}

fn report_packages(found_packages: &Vec::<(String, String)>, packages: &Vec::<String>) {
    for (manager, package) in found_packages {
        println!("'{}' found with {}!", package, manager);
    }
    println!("");
    let found_package_names: Vec<&String> = found_packages.iter().map(|(_, pkg)| pkg).collect();
    for package in packages {
        if !found_package_names.contains(&package) {
            println!("'{}' not found.", package);
        }
    }
    println!("");
}