

use std::fs;
use std::path::Path;
use std::io::{self, Write};
use zip::{ZipWriter, write::FileOptions, CompressionMethod};
use chrono::Local;
use chrono::Datelike;
use chrono::Timelike;



fn main() {
    println!(r" _____ _               _             _     _       _     _     _ ");
    println!(r"|_   _| |__   ___     / \   _ __ ___| |__ (_)_   _(_)___| |_  | |");
    println!(r"  | | | '_ \ / _ \   / _ \ | '__/ __| '_ \| \ \ / / / __| __| | |");
    println!(r"  | | | | | |  __/  / ___ \| | | (__| | | | |\ V /| \__ \ |_  |_|");
    println!(r"  |_| |_| |_|\___| /_/   \_\_|  \___|_| |_|_| \_/ |_|___/\__| (_)");

    println!("Build: 1.0.1");
    println!("\nAuthor: Tobias Kuerschner - 2024\n");
    println!("This tool will archive files in a folder and optionally delete them after archiving");
    println!("Folders can be archived entirely or by file type\n");
    println!("For more information visit: https://github.com/tkuerschner/the_archivist");

    print!("\nPlease enter the folder location: ");
    io::stdout().flush().unwrap();
    let mut folder_location = String::new();
    std::io::stdin().read_line(&mut folder_location).unwrap();
    folder_location = folder_location.trim().to_string();

   
    loop {
        match fs::read_dir(&folder_location) {
            Ok(_files) => {
                break;
            }
            Err(err) => {
                eprintln!("Error reading directory: {}", err);
                print!("Please re-enter the folder location: ");
                io::stdout().flush().unwrap();
                folder_location.clear();
                std::io::stdin().read_line(&mut folder_location).unwrap();
                folder_location = folder_location.trim().to_string();
            }
        }
    }

    // count the files in the folder excluding subfolders
    let mut file_count = 0;
    for entry in fs::read_dir(&folder_location).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            file_count += 1;
        }
    }

    println!("-----------------------------------------------");
    println!("Number of files in the folder [{}]", file_count);
    println!("-----------------------------------------------\n");

    // Get all the different file endings in the folder
    let mut file_endings = Vec::new();
    for entry in fs::read_dir(&folder_location).unwrap_or_else(|err| {
        eprintln!("Error reading directory: {}", err);
        std::process::exit(1);
    }) {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(extension) = path.extension() {
            let file_ending = extension.to_str().unwrap().to_owned();
            if !file_endings.contains(&file_ending) {
                file_endings.push(file_ending);
            }
        }
    }

    // create list of files that should always be ignored
    // ignore the executable file
    let mut ignore_files = Vec::new();
    ignore_files.push("the_archivist.exe".to_string());
    //ignore everything that starts with a dot
    for entry in fs::read_dir(&folder_location).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(file_name) = path.file_name() {
            if file_name.to_str().unwrap().starts_with(".") {
                ignore_files.push(file_name.to_str().unwrap().to_string());
            }
        }
    }
  

    let archive_folder = Path::new(&folder_location).join("archive");
        if let Err(err) = fs::create_dir_all(&archive_folder) {
            eprintln!("Error creating archive directory: {}", err);
            std::process::exit(1);
        }

    //if there is an exe in the file endings, remove it
    if let Some(index) = file_endings.iter().position(|x| x == "exe") {
        file_endings.remove(index);
        println!("Binary files detected, excluding them from archiving");
        println!("-----------------------------------------------\n");
    }

    println!("Filetypes detected: {:?}", file_endings);
    println!("-----------------------------------------------\n");

    //display the amount of files for each file type

    for file_ending in &file_endings {
        let mut file_count = 0;
        for entry in fs::read_dir(&folder_location).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension.to_str().unwrap() == file_ending {
                    file_count += 1;
                }
            }
        }
        println!("Number of [*.{}] files in the folder [{}]\n", file_ending, file_count);
    }

    println!("Choose an option: ");
    println!("1 - All files in one archive");
    println!("2 - Separate archives for each file type");
    println!("3 - Select specific file types for archiving (one archive per selected file type)");

    let mut option = String::new();
    std::io::stdin().read_line(&mut option).unwrap();
    option = option.trim().to_string();

    //let the user repeat if the input was wrong
    while option != "1" && option != "2" && option != "3" {
        println!("-----------------------------------------------");
        println!("Invalid option, please try again");
        println!("-----------------------------------------------");
        println!("Choose an option: ");
        println!("1 - All files in one archive");
        println!("2 - Separate archives for each file type");
        println!("3 - Select specific file types for archiving (one archive per selected file type)");
        option.clear();
        std::io::stdin().read_line(&mut option).unwrap();
        option = option.trim().to_string();
    }

    if option == "1" {
    
        let now = Local::now();
        let zip_name = format!(
            "full_archive_{}_{}_{}_{}_{}.zip",
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute()
        );
        let zip_path = archive_folder.join(&zip_name);
        let zip_file = match fs::File::create(&zip_path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Error creating zip file {}: {}", zip_path.display(), err);
                std::process::exit(1);
            }
        };
        let mut zip = ZipWriter::new(zip_file);
        let options = FileOptions::default().compression_method(CompressionMethod::Stored);

        for entry in fs::read_dir(&folder_location).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();

            if ignore_files.contains(&file_name.to_string()) {
                continue;
            }

            if let Err(err) = zip.start_file(file_name, options) {
                eprintln!("Error starting file in zip {}: {}", file_name, err);
                continue;
            }

            let mut file = match fs::File::open(&path) {
                Ok(file) => file,
                Err(_err) => {
                    //  eprintln!("Error opening file {}: {}", path.display(), err);
                    continue;
                }
            };

            if let Err(err) = io::copy(&mut file, &mut zip) {
                eprintln!("Error copying file {} to zip: {}", path.display(), err);
            }
        }

        if let Err(err) = zip.finish() {
            eprintln!("Error finishing zip file {}: {}", zip_path.display(), err);
        }
        println!("-----------------------------------------------");
        println!("Archive created: {}", zip_path.display());
        println!("-----------------------------------------------");
        //ask the user if they want to delete the files after archiving
        println!("Do you want to delete the files after archiving? (y/n)");
        let mut delete_option = String::new();
        std::io::stdin().read_line(&mut delete_option).unwrap();
        delete_option = delete_option.trim().to_string();

        //let the user repeat if the input was wrong
        while delete_option != "y" && delete_option != "n" {
            println!("-----------------------------------------------");
            println!("Invalid option, please try again");
            println!("-----------------------------------------------");
            println!("Do you want to delete the files after archiving? (y/n)");
            delete_option.clear();
            std::io::stdin().read_line(&mut delete_option).unwrap();
            delete_option = delete_option.trim().to_string();
        }

        if delete_option == "y" {

            println!("Warning - This action is irreversible!");
            println!("-----------------------------------------------");
            println!("Are you sure you want to delete the files? (y/n)");
            let mut delete_confirmation = String::new();
            std::io::stdin().read_line(&mut delete_confirmation).unwrap();
            delete_confirmation = delete_confirmation.trim().to_string();
            
              //let the user repeat if the input was wrong
             while delete_option != "y" && delete_option != "n" {
            println!("-----------------------------------------------");
            println!("Invalid option, please try again");
            println!("-----------------------------------------------");
            println!("Do you want to delete the files after archiving? (y/n)");
            delete_option.clear();
            std::io::stdin().read_line(&mut delete_option).unwrap();
            delete_option = delete_option.trim().to_string();
        }

            if delete_confirmation == "y" {
                for entry in fs::read_dir(&folder_location).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if let Err(err) = fs::remove_file(&path) {
                        eprintln!("Error deleting file {}: {}", path.display(), err);
                    }
                }
            }
        }
    }

    if option == "2"{
        // create a zip file for each file type naming convention is filetype_archive_year_month_day_hour_minute.zip
        for file_ending in &file_endings {
            let now = Local::now();
            let zip_name = format!(
                "{}_archive_{}_{}_{}_{}_{}.zip",
                file_ending,
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute()
            );
            let zip_path = archive_folder.join(&zip_name);
            let zip_file = match fs::File::create(&zip_path) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!("Error creating zip file {}: {}", zip_path.display(), err);
                    std::process::exit(1);
                }
            };
            let mut zip = ZipWriter::new(zip_file);
            let options = FileOptions::default().compression_method(CompressionMethod::Stored);

            for entry in fs::read_dir(&folder_location).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension.to_str().unwrap() == file_ending {
                        let file_name = path.file_name().unwrap().to_str().unwrap();

                        if ignore_files.contains(&file_name.to_string()) {
                            continue;
                        }                      
                        if let Err(err) = zip.start_file(file_name, options) {
                            eprintln!("Error starting file in zip {}: {}", file_name, err);
                            continue;
                        }
                        let mut file = match fs::File::open(&path) {
                            Ok(file) => file,
                            Err(err) => {
                                eprintln!("Error opening file {}: {}", path.display(), err);
                                continue;
                            }
                        };
                        if let Err(err) = io::copy(&mut file, &mut zip) {
                            eprintln!("Error copying file {} to zip: {}", path.display(), err);
                        }
                     
                    }
                }
            }

            if let Err(err) = zip.finish() {
                eprintln!("Error finishing zip file {}: {}", zip_path.display(), err);
            }

            println!("-----------------------------------------------");
            println!("Archive created: {}", zip_path.display());
            println!("-----------------------------------------------");
        }

           

          //ask the user if they want to delete the files after archiving
          println!("Do you want to delete the files after archiving? (y/n)");
          let mut delete_option = String::new();
          std::io::stdin().read_line(&mut delete_option).unwrap();
          delete_option = delete_option.trim().to_string();

            //let the user repeat if the input was wrong
            while delete_option != "y" && delete_option != "n" {
            println!("-----------------------------------------------");
            println!("Invalid option, please try again");
            println!("-----------------------------------------------");
            println!("Do you want to delete the files after archiving? (y/n)");
            delete_option.clear();
            std::io::stdin().read_line(&mut delete_option).unwrap();
            delete_option = delete_option.trim().to_string();
            }
  
          if delete_option == "y" {
              println!("Warning - This action is irreversible!");
              println!("-----------------------------------------------");
              println!("Are you sure you want to delete the files? (y/n)");
              let mut delete_confirmation = String::new();
              std::io::stdin().read_line(&mut delete_confirmation).unwrap();
              delete_confirmation = delete_confirmation.trim().to_string();

            //let the user repeat if the input was wrong
             while delete_option != "y" && delete_option != "n" {
            println!("-----------------------------------------------");
            println!("Invalid option, please try again");
            println!("-----------------------------------------------");
            println!("Do you want to delete the files after archiving? (y/n)");
            delete_option.clear();
            std::io::stdin().read_line(&mut delete_option).unwrap();
            delete_option = delete_option.trim().to_string();
            }
  
              if delete_confirmation == "y" {
                  for entry in fs::read_dir(&folder_location).unwrap() {
                      let entry = entry.unwrap();
                      let path = entry.path();
                      if let Err(_err) = fs::remove_file(&path) {
                          //eprintln!("Error deleting file {}: {}", path.display(), err);
                      }
                  }
              }
          }

    }


    if option == "3" {
        println!("-----------------------------------------------");
        println!("Please enter the file endings separated by comma: ");
        let mut file_endings_input = String::new();
        std::io::stdin().read_line(&mut file_endings_input).unwrap();
        file_endings_input = file_endings_input.trim().to_string();
        let mut file_endings_input: Vec<String> = file_endings_input.split(',').map(|s| s.trim().to_string()).collect();
        println!("File endings input: {:?}", file_endings_input);

        // check if the file endings input are valid and if not let the user try again
        let mut valid_input = true;
        for file_ending in &file_endings_input {
            if !file_endings.contains(file_ending) {
                valid_input = false;
                break;
            }
        }

        while !valid_input {
            println!("-----------------------------------------------");
            println!("Invalid file endings, please try again");
            println!("-----------------------------------------------");
            println!("Please enter the file endings separated by comma: ");
            file_endings_input.clear();
            let mut file_endings_input2: String = String::new();
            std::io::stdin().read_line(&mut file_endings_input2).unwrap();
            file_endings_input2 = file_endings_input2.trim().to_string();
            file_endings_input = file_endings_input2.split(',').map(|s| s.trim().to_string()).collect();
            println!("File endings input: {:?}", file_endings_input);

            valid_input = true;
            for file_ending in &file_endings_input {
                if !file_endings.contains(file_ending) {
                    valid_input = false;
                    break;
                }
            }
        }
  

        // create a zip file for each file types from file_endings_input separated by comma, naming convention is filetype_archive_year_month_day_hour_minute.zip
        for file_ending in &file_endings_input {
            let now = Local::now();
            let zip_name = format!(
                "{}_archive_{}_{}_{}_{}_{}.zip",
                file_ending,
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute()
            );
            let zip_path = archive_folder.join(&zip_name);
            let zip_file = match fs::File::create(&zip_path) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!("Error creating zip file {}: {}", zip_path.display(), err);
                    std::process::exit(1);
                }
            };
            let mut zip = ZipWriter::new(zip_file);
            let options = FileOptions::default().compression_method(CompressionMethod::Stored);

            for entry in fs::read_dir(&folder_location).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension.to_str().unwrap() == file_ending {
                        let file_name = path.file_name().unwrap().to_str().unwrap();
                        if let Err(err) = zip.start_file(file_name, options) {
                            eprintln!("Error starting file in zip {}: {}", file_name, err);
                            continue;
                        }
                        let mut file = match fs::File::open(&path) {
                            Ok(file) => file,
                            Err(err) => {
                                eprintln!("Error opening file {}: {}", path.display(), err);
                                continue;
                            }
                        };
                        if let Err(err) = io::copy(&mut file, &mut zip) {
                            eprintln!("Error copying file {} to zip: {}", path.display(), err);
                        }
                    }
                }
            }

            if let Err(err) = zip.finish() {
                eprintln!("Error finishing zip file {}: {}", zip_path.display(), err);
            }
            println!("-----------------------------------------------");
            println!("Archive created: {}", zip_path.display());
            println!("-----------------------------------------------");
        }

        //ask the user if they want to delete the files after archiving buu only for the selected file types and add an additional warning and confirmation
        println!("Do you want to delete the files after archiving? (y/n)");
        let mut delete_option = String::new();
        std::io::stdin().read_line(&mut delete_option).unwrap();
        delete_option = delete_option.trim().to_string();

          //let the user repeat if the input was wrong
          while delete_option != "y" && delete_option != "n" {
            println!("-----------------------------------------------");
            println!("Invalid option, please try again");
            println!("-----------------------------------------------");
            println!("Do you want to delete the files after archiving? (y/n)");
            delete_option.clear();
            std::io::stdin().read_line(&mut delete_option).unwrap();
            delete_option = delete_option.trim().to_string();
        }

        if delete_option == "y" {
            println!("Warning - This action is irreversible!");
            println!("-----------------------------------------------");
            println!("Are you sure you want to delete the files? (y/n)");
            let mut delete_confirmation = String::new();
            std::io::stdin().read_line(&mut delete_confirmation).unwrap();
            delete_confirmation = delete_confirmation.trim().to_string();

              //let the user repeat if the input was wrong
            while delete_option != "y" && delete_option != "n" {
            println!("-----------------------------------------------");
            println!("Invalid option, please try again");
            println!("-----------------------------------------------");
            println!("Do you want to delete the files after archiving? (y/n)");
            delete_option.clear();
            std::io::stdin().read_line(&mut delete_option).unwrap();
            delete_option = delete_option.trim().to_string();
            }

            if delete_confirmation == "y" {
                for entry in fs::read_dir(&folder_location).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if let Some(extension) = path.extension() {
                        if file_endings_input.contains(&extension.to_str().unwrap().to_string()) {
                            if let Err(err) = fs::remove_file(&path) {
                                eprintln!("Error deleting file {}: {}", path.display(), err);
                            }
                        }
                    }
                }
            }
        }
    }

    // if any ignore_files were found, display them

    if !ignore_files.is_empty() {
        println!("-----------------------------------------------");
        println!("Files that were ignored: {:?}", ignore_files);
        println!("-----------------------------------------------");
    }

    //keep the program running until the user presses enter
    println!("Press enter to exit");
    let mut exit = String::new();
    std::io::stdin().read_line(&mut exit).unwrap();
    

    println!("Exiting...");


}
