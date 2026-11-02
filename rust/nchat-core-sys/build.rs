// Builds the C++/Go core with CMake and tells cargo how to link it.
//
// The full link line (all static archives, frameworks, system libs) is taken from the
// link.txt CMake writes for the nchat-bridge-probe executable, so protocol libraries and
// their dependencies never have to be listed here.
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_CMAKE_ARGS: &str = "-DHAS_BRIDGE=ON -DHAS_TESTS=OFF -DHAS_TELEGRAM=OFF \
     -DHAS_WHATSAPP=OFF -DHAS_SIGNAL=OFF -DHAS_SHARED_LIBS=OFF -DCMAKE_BUILD_TYPE=Release";

fn run(cmd: &mut Command) {
    let status = cmd
        .status()
        .unwrap_or_else(|e| panic!("failed to run {cmd:?}: {e}"));
    assert!(status.success(), "{cmd:?} failed with {status}");
}

fn abs(cwd: &Path, p: &str) -> PathBuf {
    let path = Path::new(p);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        let joined = cwd.join(path);
        joined.canonicalize().unwrap_or(joined)
    }
}

fn emit_file(path: &Path, kind: &str) {
    let dir = path.parent().expect("library path has a parent");
    let file = path.file_name().unwrap().to_string_lossy().into_owned();
    let stem = file.split('.').next().unwrap();
    let name = stem.strip_prefix("lib").unwrap_or(stem);
    println!("cargo:rustc-link-search=native={}", dir.display());
    println!("cargo:rustc-link-lib={kind}={name}");
}

// Paths containing spaces are not supported (the repository path has none).
fn emit_link(link_txt: &str, cwd: &Path) {
    let toks: Vec<&str> = link_txt.split_whitespace().collect();
    let mut i = 1; // toks[0] is the compiler driver
    while i < toks.len() {
        let t = toks[i];
        i += 1;
        if t == "-o" || t == "-arch" || t == "-isysroot" {
            i += 1;
        } else if t == "-framework" {
            println!("cargo:rustc-link-lib=framework={}", toks[i]);
            i += 1;
        } else if let Some(dir) = t.strip_prefix("-L") {
            println!("cargo:rustc-link-search=native={}", abs(cwd, dir).display());
        } else if let Some(lib) = t.strip_prefix("-l") {
            println!("cargo:rustc-link-lib={lib}");
        } else if t.ends_with(".a") {
            emit_file(&abs(cwd, t), "static:-bundle");
        } else if t.ends_with(".dylib") || t.ends_with(".tbd") || t.contains(".so") {
            emit_file(&abs(cwd, t), "dylib");
        }
    }

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=c++");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
    }
}

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest.join("../..").canonicalize().unwrap();
    let build = root.join("build-rs");

    println!("cargo:rerun-if-env-changed=NCHAT_CMAKE_ARGS");
    println!("cargo:rerun-if-changed={}", root.join("CMakeLists.txt").display());
    for dir in [
        "lib/bridge", "lib/common/src", "lib/ncutil/src", "lib/duchat/src",
        "lib/tgchat/src", "lib/wmchat/src", "lib/sgchat/src",
        "lib/wmchat/go", "lib/sgchat/go", "lib/gostat",
        "lib/duchat/CMakeLists.txt", "lib/gostat/CMakeLists.txt", "lib/ncutil/CMakeLists.txt",
        "lib/sgchat/CMakeLists.txt", "lib/tgchat/CMakeLists.txt", "lib/wmchat/CMakeLists.txt",
    ] {
        println!("cargo:rerun-if-changed={}", root.join(dir).display());
    }

    // Defaults only seed a fresh build dir; an existing cache is reused as-is.
    let args = std::env::var("NCHAT_CMAKE_ARGS").unwrap_or_else(|_| {
        if build.join("CMakeCache.txt").exists() { String::new() } else { DEFAULT_CMAKE_ARGS.to_string() }
    });
    let jobs = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);

    run(Command::new("cmake")
        .arg("-S").arg(&root)
        .arg("-B").arg(&build)
        .args(["-G", "Unix Makefiles"])
        .args(args.split_whitespace()));
    run(Command::new("cmake")
        .arg("--build").arg(&build)
        .args(["--target", "nchat-bridge-probe", "-j"])
        .arg(jobs.to_string()));

    let bridge_dir = build.join("lib/bridge");
    let link_txt = bridge_dir.join("CMakeFiles/nchat-bridge-probe.dir/link.txt");
    let text = std::fs::read_to_string(&link_txt)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", link_txt.display()));
    emit_link(&text, &bridge_dir);
}
