# bldep

`bldep` is a command-line tool designed to streamline the process of locating external dependencies for C/C++ projects by leveraging multiple package managers. It automatically identifies project dependencies and attempts to locate them using `vcpkg`, `Conan`, or `pkg-config`.

## Features

 - **Detect Missing Package Managers**: Automatically checks for the presence of `vcpkg`, `Conan`, and `pkg-config`.
 - **Install Missing Package Managers**: Downloads and installs missing tools to ensure seamless dependency management.
 - **Scan Project Files**: Recursively scans your project directory for `#include` directives in C/C++ files and their headers.
 - **Extract Dependency Names**: Attempts to infer package names based on `#include` statements.
 - **Check Local Headers**: Checks standard library headers before moving onto package managers.
 - **Search Package Managers**: Searches installed package managers for the required dependencies.
 - **Report Results**: Provides a report of found and missing dependencies.

## Installation

### Prerequisites
 - Rust toolchain (`cargo`) for building the project.
 - `git` for cloning repositories (required for installing `vcpkg`).
 - `pip` for Python package management (required for installing `Conan`).
 - A system package manager (e.g., `brew` or `apt`).
     - Windows not yet supported.

### Build and Install
 - Clone the repository:
    ```bash
    git clone https://github.com/BendyLand/bldep.git
    cd bldep
    ```
 - Build the project:
    ```bash
    cargo build --release
    ```
 - Install the executable:
    ```bash
    cp target/release/bldep /usr/local/bin/
    ```

## Usage

 - Navigate to the root directory of your C/C++ project:
    ```bash
    cd /path/to/your/project
    ```
 - Run `bldep` to analyze dependencies:
    ```bash
    bldep
    ```
 - Optionally specify a directory to analyze:
    ```bash
    bldep /path/to/another/project
    ```

### Example Output
```bash
bldep pytorch-cpp
```
```txt
Downloading missing package managers...
Cloning vcpkg...
Cloning into 'vcpkg'...
remote: Enumerating objects: 259645, done.
remote: Counting objects: 100% (11122/11122), done.
remote: Compressing objects: 100% (330/330), done.
remote: Total 259645 (delta 10947), reused 10800 (delta 10792), pack-reused 248523 (from 5)
Receiving objects: 100% (259645/259645), 80.26 MiB | 6.53 MiB/s, done.
Resolving deltas: 100% (171781/171781), done.
Updating files: 100% (12012/12012), done.
Bootstrapping vcpkg...
Downloading vcpkg-macos...
vcpkg package management program version 2024-12-09-1005b78fa1bf1dde1a20c2734cba4ea61ca94d9a

See LICENSE.txt for license information.
Telemetry
---------
vcpkg collects usage data in order to help us improve your experience.
The data collected by Microsoft is anonymous.
You can opt-out of telemetry by re-running the bootstrap-vcpkg script with -disableMetrics,
passing --disable-metrics to vcpkg on the command line,
or by setting the VCPKG_DISABLE_METRICS environment variable.

Read more about vcpkg telemetry at docs/about/privacy.md
vcpkg installed successfully.

Checking Conan for 'aten'...
Checking Conan for 'cuda_runtime'...
Checking Conan for 'h5cpp'...
Checking Conan for 'opencv'...

Checking pkg-config for 'aten'...
Checking pkg-config for 'cuda_runtime'...
Checking pkg-config for 'h5cpp'...
Checking pkg-config for 'opencv'...

Checking vcpkg for 'aten'...
Checking vcpkg for 'cuda_runtime'...
Checking vcpkg for 'h5cpp'...
Checking vcpkg for 'opencv'...

'opencv' found with Conan!
'aten' found with vcpkg!
'opencv' found with vcpkg!

'cuda_runtime' not found.
'h5cpp' not found.

vcpkg removed successfully!
```

## Supported Package Managers

 - **vcpkg**: Installs via Git and bootstraps automatically into the working directory.
     - Automatically removed after usage.
 - **Conan**: Installs via `pip` if not already installed.
 - **pkg-config**: Installs via system package manager (`apt` on Linux, `brew` on MacOS).

## Current Limitations

 - The tool currently only supports MacOS and Linux.
 - Package manager outputs are not deeply parsed for suggestions like "Did you mean...".

## Design Limitations 

 - Package names that do not directly correspond to their header file names will need to rely on storing hardcoded names as records, since they are nearly impossible to infer accurately across all header styles.
