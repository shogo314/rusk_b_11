use std::{
    fs,
    path::{Path, PathBuf},
};

fn main() {
    compiler_identity();
    println!("cargo:rerun-if-changed=rusk.par");
    println!("cargo:rerun-if-changed=lib");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo output directory"));
    // Identity changes and library edits must not regenerate an unchanged parser.
    let parser_inputs = ["rusk.par", "build.rs", "Cargo.lock"]
        .map(|path| fs::read_to_string(path).expect("parser input"));
    let parser_inputs = format!("{parser_inputs:?}");
    let stamp = out.join("parser-inputs");
    if fs::read_to_string(&stamp).ok().as_deref() != Some(parser_inputs.as_str())
        || !out.join("rusk_parser.rs").exists()
        || !out.join("rusk_grammar_trait.rs").exists()
    {
        parol::build::Builder::with_explicit_output_dir(&out)
            .max_lookahead(3)
            .expect("valid lookahead")
            // Bound the recursion needed to build, print and drop the generated CST.
            .max_parsing_depth(4096)
            .grammar_file("rusk.par")
            .parser_output_file("rusk_parser.rs")
            .actions_output_file("rusk_grammar_trait.rs")
            .user_type_name("RuskGrammar")
            .user_trait_module_name("frontend::rusk_grammar")
            .trim_parse_tree()
            .generate_parser()
            .expect("Rusk grammar generation failed");
        // Generated module-level lint attributes cannot appear inside include!.
        for name in ["rusk_parser.rs", "rusk_grammar_trait.rs"] {
            let path = out.join(name);
            let text = fs::read_to_string(&path).unwrap();
            let text = text
                .lines()
                .filter(|line| !line.starts_with("#!["))
                .collect::<Vec<_>>()
                .join("\n");
            fs::write(path, text).unwrap();
        }
        fs::write(stamp, parser_inputs).expect("parser input stamp");
    }
    embed_standard_library(&out);
}

fn embed_standard_library(out: &Path) {
    use std::fmt::Write;

    fn visit(root: &Path, dir: &Path, resources: &mut Vec<(String, PathBuf)>) {
        for entry in fs::read_dir(dir).unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
        {
            let entry = entry.expect("read library directory entry");
            let path = entry.path();
            let kind = entry.file_type().expect("read library resource type");
            if kind.is_dir() {
                visit(root, &path, resources);
            } else if kind.is_file()
                && path
                    .extension()
                    .is_some_and(|ext| ext == "rusk" || ext == "hpp")
            {
                let name = path
                    .strip_prefix(root)
                    .expect("library resource relative to root")
                    .components()
                    .map(|component| {
                        component
                            .as_os_str()
                            .to_str()
                            .expect("UTF-8 library resource name")
                    })
                    .collect::<Vec<_>>()
                    .join("/");
                resources.push((name, path));
            }
        }
    }

    let root =
        PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory"))
            .join("lib");
    let mut resources = Vec::new();
    visit(&root, &root, &mut resources);
    resources.sort();
    let mut generated = String::from("const FILES: &[(&str, &str)] = &[\n");
    for (name, path) in resources {
        let path = path.to_str().expect("UTF-8 library source path");
        writeln!(generated, "    ({name:?}, include_str!({path:?})),").unwrap();
    }
    generated.push_str("];\n");
    fs::write(out.join("stdlib_resources.rs"), generated)
        .expect("write standard library resource catalog");
}

fn compiler_identity() {
    use std::process::Command;
    for path in [
        "src",
        "lib",
        "tests",
        "docs",
        "design",
        "examples",
        "tools",
        ".github",
        "Cargo.toml",
        "Cargo.lock",
        "README.md",
        "AGENTS.md",
        "mkdocs.yml",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }
    let git = |args: &[&str]| {
        Command::new("git")
            .arg("--no-optional-locks")
            .args(args)
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
    };
    let revision = git(&["rev-parse", "--verify", "HEAD"]).unwrap_or_default();
    let modified = git(&["status", "--porcelain", "--untracked-files=normal"])
        .map(|status| !status.is_empty())
        .unwrap_or(false);
    println!("cargo:rustc-env=RUSK_BUILD_REVISION={}", revision.trim());
    println!("cargo:rustc-env=RUSK_BUILD_MODIFIED={modified}");
    for path in [
        git(&["rev-parse", "--git-path", "HEAD"]),
        git(&["rev-parse", "--git-path", "index"]),
    ]
    .into_iter()
    .flatten()
    {
        println!("cargo:rerun-if-changed={}", path.trim());
    }
    if let Some(reference) = git(&["symbolic-ref", "-q", "HEAD"])
        && let Some(path) = git(&["rev-parse", "--git-path", reference.trim()])
    {
        println!("cargo:rerun-if-changed={}", path.trim());
    }
}
