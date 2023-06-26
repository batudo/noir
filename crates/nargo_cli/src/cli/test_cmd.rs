use std::{io::Write, path::Path};

use acvm::{acir::native_types::WitnessMap, Backend};
use clap::Args;
use nargo::ops::execute_circuit;
use noirc_driver::{CompileOptions, Driver};
use noirc_frontend::node_interner::FuncId;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::{
    cli::{check_cmd::check_crate_and_report_errors, compile_cmd::setup_driver},
    errors::CliError,
};

use super::NargoConfig;

/// Run the tests for this program
#[derive(Debug, Clone, Args)]
pub(crate) struct TestCommand {
    /// If given, only tests with names containing this string will be run
    test_name: Option<String>,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: TestCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let test_name: String = args.test_name.unwrap_or_else(|| "".to_owned());

    run_tests(backend, &config.nargo_package_root, &test_name, &args.compile_options)
}

fn run_tests<B: Backend>(
    backend: &B,
    program_dir: &Path,
    test_name: &str,
    compile_options: &CompileOptions,
) -> Result<(), CliError<B>> {
    let mut driver = setup_driver(backend, program_dir)?;
    check_crate_and_report_errors(&mut driver, compile_options.deny_warnings)?;

    let test_functions = driver.get_all_test_functions_in_crate_matching(test_name);
    println!("Running {} test functions...", test_functions.len());
    let mut failing = 0;

    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut writer = writer.lock();

    for test_function in test_functions {
        let test_name = driver.function_name(test_function);
        writeln!(writer, "Testing {test_name}...").expect("Failed to write to stdout");
        writer.flush().ok();

        match run_test(backend, test_name, test_function, &driver, compile_options) {
            Ok(_) => {
                writer.set_color(ColorSpec::new().set_fg(Some(Color::Green))).ok();
                writeln!(writer, "ok").ok();
            }
            // Assume an error was already printed to stdout
            Err(_) => failing += 1,
        }
        writer.reset().ok();
    }

    if failing == 0 {
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap();
        writeln!(writer, "All tests passed").ok();
    } else {
        let plural = if failing == 1 { "" } else { "s" };
        return Err(CliError::Generic(format!("{failing} test{plural} failed")));
    }

    writer.reset().ok();
    Ok(())
}

fn run_test<B: Backend>(
    backend: &B,
    test_name: &str,
    main: FuncId,
    driver: &Driver,
    config: &CompileOptions,
) -> Result<(), CliError<B>> {
    let program = driver
        .compile_no_check(config, main)
        .map_err(|_| CliError::Generic(format!("Test '{test_name}' failed to compile")))?;

    // Run the backend to ensure the PWG evaluates functions like std::hash::pedersen,
    // otherwise constraints involving these expressions will not error.
    match execute_circuit(backend, program.circuit, WitnessMap::new()) {
        Ok(_) => Ok(()),
        Err(error) => {
            let writer = StandardStream::stderr(ColorChoice::Always);
            let mut writer = writer.lock();
            writer.set_color(ColorSpec::new().set_fg(Some(Color::Red))).ok();
            writeln!(writer, "failed").ok();
            writer.reset().ok();
            Err(error.into())
        }
    }
}
