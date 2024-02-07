use std::{env, io};
use std::io::{stdin, Write};
use std::path::PathBuf;
use rpkg_rs::misc::ini_file_system::IniFileSystem;
use rpkg_rs::misc::resource_id::ResourceID;
use rpkg_rs::runtime::resource::package_defs::{PackageDefinitionSource};
use rpkg_rs::runtime::resource::partition_manager::{PartitionManager, PartitionState};
use rpkg_rs::runtime::resource::runtime_resource_id::RuntimeResourceID;
use rpkg_rs::utils;

fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() < 3{
        eprintln!("Usage: cargo run --example <example_name> -- <path to a retail directory> <game version (H2016 | HM2 | HM3)>");
        return;
    }

    let retail_path = PathBuf::from(&args[1]);
    let thumbs_path = retail_path.join("thumbs.dat");

    let thumbs = IniFileSystem::from(&thumbs_path.as_path()).unwrap_or_else(|err| {
        eprintln!("Error reading thumbs file: {:?}", err);
        std::process::exit(1);
    });

    let app_options = &thumbs.get_root().unwrap_or_else(|| {
        eprintln!("Missing root in thumbs file");
        std::process::exit(1);
    })["application"];

    if let (Some(proj_path), Some(relative_runtime_path)) = (app_options.get_option("PROJECT_PATH"), app_options.get_option("RUNTIME_PATH")) {

        let runtime_path = PathBuf::from(format!("{}\\{proj_path}\\{relative_runtime_path}", retail_path.display()));
        std::println!("start reading package definitions {:?}", runtime_path);

        let mut package_manager = PartitionManager::new(runtime_path.clone());

        //read the packagedefs here
        let mut last_index = 0;
        let mut progress = 0.0;
        let progress_callback = |current, state: &PartitionState| {
            if current != last_index{
                last_index = current;
                print!("Mounting partition {} ", current);
            }
            let install_progress= (state.install_progress*10.0).ceil()/10.0;

            let chars_to_add = (install_progress*10.0 - progress * 10.0) as usize * 2;
            let chars_to_add = std::cmp::min(chars_to_add, 20);
            print!("{}", "█".repeat(chars_to_add));
            io::stdout().flush().unwrap();

            progress = install_progress;

            if progress == 1.0{
                progress = 0.0;
                println!(" done :)");
            }
        };

        let package_defs_bytes = utils::get_file_as_byte_vec(runtime_path.join("packagedefinition.txt").as_path()).unwrap();

        let package_defs = match args[2].as_str(){
            "H2016" => {PackageDefinitionSource::H2016(package_defs_bytes).read()},
            "HM2" => {PackageDefinitionSource::HM2(package_defs_bytes).read()},
            "HM3" => {PackageDefinitionSource::HM3(package_defs_bytes).read()},
            e => {
                eprintln!("invalid game version: {}", e);
                std::process::exit(0);
            }
        };

        package_manager.mount_partitions(PackageDefinitionSource::Custom(package_defs.unwrap()), progress_callback).unwrap_or_else(|e|{
            eprintln!("failed to init package manager: {}", e);
            std::process::exit(0);
        });

        loop {
            print!("enter a ResourceID > ");
            io::stdout().flush().unwrap();

            let mut input_string = String::new();
            stdin().read_line(&mut input_string)
                .ok()
                .expect("Failed to read line");

            let rid = ResourceID::from_string(input_string.as_str());
            if !rid.is_valid() {
                println!("Invalid ResourceID");
                continue;
            }

            let rrid = RuntimeResourceID::from_resource_id(&rid);
            println!("Try to find {}", rrid);
            match package_manager.get_resource_info_from(0, rrid) {
                Ok(info) => {println!("Managed to find {}.{}", rrid, info.header.get_type())}
                Err(e) => {eprintln!("error: {}", e)}
            }
        }
    } else {
        eprintln!("Missing required properties inside thumbs.dat:\n\
             PROJECT_PATH: {}\n\
             RUNTIME_PATH: {}",
                  app_options.has_option("PROJECT_PATH"),
                  app_options.has_option("RUNTIME_PATH"));
        return;
    }
}