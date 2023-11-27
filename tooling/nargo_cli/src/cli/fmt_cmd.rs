use std::{fs::DirEntry, path::Path};

use clap::Args;
use fm::FileManager;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::NOIR_ARTIFACT_VERSION_STRING;
use noirc_errors::CustomDiagnostic;
use noirc_frontend::{hir::def_map::parse_file, parser::ParserError};

use crate::errors::CliError;

use super::NargoConfig;

/// Format the Noir files in a workspace
#[derive(Debug, Clone, Args)]
pub(crate) struct FormatCommand {
    /// Run noirfmt in check mode
    #[arg(long)]
    check: bool,
}

pub(crate) fn run(args: FormatCommand, config: NargoConfig) -> Result<(), CliError> {
    let check_mode = args.check;

    let toml_path = get_package_manifest(&config.program_dir)?;
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        PackageSelection::All,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;

    let config = nargo_fmt::Config::read(&config.program_dir)
        .map_err(|err| CliError::Generic(err.to_string()))?;

    let mut check_exit_code_one = false;

    for package in &workspace {
        let mut file_manager =
            FileManager::new(&package.root_dir, Box::new(|path| std::fs::read_to_string(path)));

        visit_noir_files(&package.root_dir.join("src"), &mut |entry| {
            let file_id = file_manager.add_file(&entry.path()).expect("file exists");
            let (parsed_module, errors) = parse_file(&file_manager, file_id);

            let is_all_warnings = errors.iter().all(ParserError::is_warning);
            if !is_all_warnings {
                let errors = errors
                    .into_iter()
                    .map(|error| {
                        let error: CustomDiagnostic = error.into();
                        error.in_file(file_id)
                    })
                    .collect();

                let _ = super::compile_cmd::report_errors::<()>(
                    Err(errors),
                    &file_manager,
                    false,
                    false,
                );
                return Ok(());
            }

            let original = file_manager.fetch_file(file_id).source();
            let formatted = nargo_fmt::format(original, parsed_module, &config);

            if check_mode {
                let diff = similar_asserts::SimpleDiff::from_str(
                    original,
                    &formatted,
                    "original",
                    "formatted",
                )
                .to_string();

                if !diff.lines().next().is_some_and(|line| line.contains("Invisible differences")) {
                    if !check_exit_code_one {
                        check_exit_code_one = true;
                    }

                    println!("{diff}");
                }

                Ok(())
            } else {
                std::fs::write(entry.path(), formatted)
            }
        })
        .map_err(|error| CliError::Generic(error.to_string()))?;
    }

    if check_exit_code_one {
        std::process::exit(1);
    } else if check_mode {
        println!("No formatting changes were detected");
    }

    Ok(())
}

fn visit_noir_files(
    dir: &Path,
    cb: &mut dyn FnMut(&DirEntry) -> std::io::Result<()>,
) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_noir_files(&path, cb)?;
            } else if entry.path().extension().map_or(false, |extension| extension == "nr") {
                cb(&entry)?;
            }
        }
    }
    Ok(())
}
