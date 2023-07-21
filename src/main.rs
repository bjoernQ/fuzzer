use std::{env, process::exit};

use regex::{Match, Regex};
use xshell::cmd;

fn main() {
    let env: Vec<(String, String)> = env::vars().collect();
    let mut found_wokwi_key = false;

    for (key, _value) in env {
        if key == "WOKWI_CLI_TOKEN" {
            found_wokwi_key = true;
        }
    }

    if !found_wokwi_key {
        println!("Set WOKWI_CLI_TOKEN");
        exit(1);
    }

    loop {
        let sh = xshell::Shell::new().unwrap();
        let cargo = format!(
            "{}/.cargo/bin/cargo",
            home::home_dir().unwrap().to_str().unwrap()
        );

        std::fs::remove_dir_all("outRust").ok();
        cmd!(sh, "java -jar RustSmith-1.0-SNAPSHOT.jar -n 1")
            .run()
            .unwrap();

        let mut content = std::fs::read_to_string("outRust/file0/file0.rs").unwrap();
        let args_content = std::fs::read_to_string("outRust/file0/file0.txt").unwrap();

        // RustSmith assumes std - change that to only assume alloc
        content = content.replace("#![allow(warnings, unused, unconditional_panic)]", "");
        content = content.replace("use std::env;", "");
        content = content.replace("use std::collections::hash_map::DefaultHasher;", "");
        content = content.replace("use std::hash::{Hash, Hasher};", "");

        content.insert_str(
            0,
            "
            #![allow(warnings, unused, unconditional_panic)]
            use core::hash::{Hash, Hasher};
            use alloc::{string::String, format, vec::Vec, boxed::Box};
            use alloc::vec;
            use esp_println::println;
            use core::hash::SipHasher as DefaultHasher;
            use crate::alloc::string::ToString;            
            ",
        );

        // RustSmith assumes 64bit usize ... change that
        let re = Regex::new(r"([0-9]+)usize").unwrap();
        let copy = content.to_owned();
        let all: Vec<Match> = re.find_iter(&copy).collect();

        let mut replacements: Vec<(String, String)> = Vec::new();

        for found in all.into_iter().rev() {
            let previous = &content[found.start()..found.end()];
            let new: u64 =
                previous[..(previous.len() - 5)].parse::<u64>().unwrap() / u32::MAX as u64;
            replacements.push((previous.to_owned(), format!("{}usize", new)));
        }

        for r in replacements {
            content = content.replace(&r.0, &r.1);
        }

        // RustSmith assumes running the code as a CLI command - change that
        let index = content.find("fn main( ) -> () {").unwrap();
        content.insert_str(index, "pub ");

        // TODO
        let mut args_as_vec = String::from("vec![\"\".to_string(),");
        for arg in args_content.split(" ") {
            if let Ok(arg) = arg.parse::<u64>() {
                args_as_vec.push_str(&format!("\"{}\".to_string(),", arg / u32::MAX as u64));
            } else {
                args_as_vec.push_str(&format!("\"{}\".to_string(),", arg));
            }
        }
        args_as_vec.push_str("];");
        content = content.replace("env::args().collect();", &args_as_vec);

        std::fs::write("esp32c3/src/tst.rs", &content).unwrap();
        std::fs::write("esp32/src/tst.rs", &content).unwrap();

        // ESP32-C3

        sh.change_dir("esp32c3");

        cmd!(sh, "{cargo} build --release")
            .ignore_stdout()
            .ignore_stderr()
            .run()
            .unwrap();

        let out = cmd!(
            sh,
            "esp32c3/wokwi-cli-win-x64.exe --timeout 3000 --timeout-exit-code 0"
        )
        .read()
        .unwrap();

        if !out.contains("<<<") {
            continue;
        }

        let wanted = out.split_terminator(">>>").nth(1).unwrap();
        let wanted = wanted.split_terminator("<<<").nth(0).unwrap().trim();

        // ESP32
        sh.change_dir("../esp32");

        cmd!(sh, "{cargo} +esp build --release")
            .ignore_stdout()
            .ignore_stderr()
            .run()
            .unwrap();

        let out = cmd!(
            sh,
            "esp32/wokwi-cli-win-x64.exe --timeout 4000 --timeout-exit-code 0"
        )
        .read()
        .unwrap();

        if !out.contains(">>>") {
            println!("Didn't got any result: {}", out);
            continue;
        }

        let out = out.split_terminator(">>>").nth(1).unwrap();
        let out = out.split_terminator("<<<").nth(0).unwrap().trim();

        if wanted != out {
            println!("Oh no! wanted='{wanted}', got '{out}'");
            std::fs::copy(
                "./esp32c3/src/tst.rs",
                format!(
                    "./findings/{}.rs",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                ),
            )
            .unwrap();
        } else {
            println!("All fine this time {wanted}=={out}");
        }
    }
}
