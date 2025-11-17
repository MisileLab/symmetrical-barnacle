mod ast;
mod lexer;
mod parser;
mod types;
mod effect;
mod typecheck;
mod codegen;
mod optimize;
mod runtime;
mod backend_x86;
mod backend_wasm;
mod backend_llvm;

use clap::{Parser as ClapParser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(ClapParser)]
#[command(name = "fluxc")]
#[command(about = "Flux language compiler with zero-allocation safety and effect system", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Type-check a Flux source file
    Check {
        /// Path to the Flux source file
        file: PathBuf,
    },
    /// Build a Flux source file to native executable or WebAssembly
    Build {
        /// Path to the Flux source file
        file: PathBuf,

        /// Target architecture (x86_64 or wasm32)
        #[arg(long, default_value = "x86_64")]
        target: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Build and run a Flux source file (x86_64 only)
    Run {
        /// Path to the Flux source file
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Check { file } => cmd_check(&file),
        Commands::Build { file, target, output } => cmd_build(&file, &target, output),
        Commands::Run { file } => cmd_run(&file),
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn cmd_check(file: &PathBuf) -> Result<(), String> {
    let source = fs::read_to_string(file)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let mut parser = parser::Parser::new(&source);
    let module = parser.parse_module()
        .map_err(|e| format!("Parse error: {}", e.message))?;

    let mut checker = typecheck::TypeChecker::new();
    checker.check_module(&module)
        .map_err(|e| format!("Type error: {}", e.message))?;

    println!("✓ Type checking passed");
    println!("✓ Effect checking passed");
    Ok(())
}

fn cmd_build(file: &PathBuf, target: &str, output: Option<PathBuf>) -> Result<(), String> {
    let source = fs::read_to_string(file)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Parse
    let mut parser = parser::Parser::new(&source);
    let module = parser.parse_module()
        .map_err(|e| format!("Parse error: {}", e.message))?;

    // Type check
    let mut checker = typecheck::TypeChecker::new();
    let env = checker.check_module(&module)
        .map_err(|e| format!("Type error: {}", e.message))?;

    // Code generation
    let mut codegen = codegen::CodeGenerator::new();
    let ir_module = codegen.generate(&module, &env);

    // Apply aggressive optimizations
    let optimizer = optimize::Optimizer::aggressive();
    let ir_module = optimizer.optimize(ir_module);

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        let mut path = file.clone();
        path.set_extension(if target == "wasm32" { "wasm" } else { "" });
        path
    });

    // Backend compilation - LLVM is default for best performance
    match target {
        "x86_64" | "x86-64" | "native" => {
            println!("Compiling with LLVM backend (optimal performance)...");
            let mut backend = backend_llvm::LLVMBackend::new();
            match backend.compile_to_executable(&ir_module, &output_path) {
                Ok(_) => {
                    println!("✓ Compiled to: {}", output_path.display());
                }
                Err(e) if e.contains("llc not found") => {
                    println!("⚠ LLVM not available, falling back to Cranelift");
                    let mut backend = backend_x86::X86Backend::new();
                    backend.compile_to_executable(&ir_module, &output_path)?;
                    println!("✓ Compiled to: {} (Cranelift)", output_path.display());
                }
                Err(e) => return Err(e),
            }
        }
        "cranelift" => {
            println!("Compiling with Cranelift backend...");
            let mut backend = backend_x86::X86Backend::new();
            backend.compile_to_executable(&ir_module, &output_path)?;
            println!("✓ Compiled to: {}", output_path.display());
        }
        "wasm32" | "wasm" => {
            println!("Compiling to WebAssembly...");
            let mut backend = backend_wasm::WasmBackend::new();
            backend.compile_to_wasm(&ir_module, &output_path)?;
            println!("✓ Compiled to: {}", output_path.display());
        }
        _ => {
            return Err(format!("Unknown target: {}", target));
        }
    }

    Ok(())
}

fn cmd_run(file: &PathBuf) -> Result<(), String> {
    let source = fs::read_to_string(file)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Parse
    let mut parser = parser::Parser::new(&source);
    let module = parser.parse_module()
        .map_err(|e| format!("Parse error: {}", e.message))?;

    // Type check
    let mut checker = typecheck::TypeChecker::new();
    let env = checker.check_module(&module)
        .map_err(|e| format!("Type error: {}", e.message))?;

    // Code generation
    let mut codegen = codegen::CodeGenerator::new();
    let ir_module = codegen.generate(&module, &env);

    // Apply aggressive optimizations
    let optimizer = optimize::Optimizer::aggressive();
    let ir_module = optimizer.optimize(ir_module);

    // Check if main function exists
    if !ir_module.functions.iter().any(|f| f.name == "main") {
        return Err("No main function found".to_string());
    }

    // Compile to temporary executable
    let temp_dir = std::env::temp_dir();
    let exe_path = temp_dir.join("flux_temp_exe");

    println!("Compiling with LLVM...");
    let mut backend = backend_llvm::LLVMBackend::new();
    match backend.compile_to_executable(&ir_module, &exe_path) {
        Ok(_) => {}
        Err(e) if e.contains("llc not found") => {
            println!("⚠ LLVM not available, using Cranelift");
            let mut backend = backend_x86::X86Backend::new();
            backend.compile_to_executable(&ir_module, &exe_path)?;
        }
        Err(e) => return Err(e),
    }

    // Run the executable
    println!("Running...");
    let output = process::Command::new(&exe_path)
        .output()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    // Print output
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }

    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    // Get exit code
    let exit_code = output.status.code().unwrap_or(1);
    println!("\nProgram exited with code: {}", exit_code);

    // Clean up
    std::fs::remove_file(&exe_path).ok();

    if !output.status.success() {
        return Err(format!("Program exited with code: {}", exit_code));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_check_simple_program() {
        let source = "inc: i32 -> i32\ninc x = x + 1";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(source.as_bytes()).unwrap();

        let result = cmd_check(&temp_file.path().to_path_buf());
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_effect_violation() {
        let source = "foo: i32 -> i32 !{pure, cpu, alloc heap}\nfoo x = x + 1\n\nbar: i32 -> i32 !{pure, cpu, alloc none}\nbar x = foo x";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(source.as_bytes()).unwrap();

        let result = cmd_check(&temp_file.path().to_path_buf());
        assert!(result.is_err());
    }
}
