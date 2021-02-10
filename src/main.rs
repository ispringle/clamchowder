#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use std::ops::{Deref, DerefMut};
use std::path::Path;

use clamav_rs::{db, engine, engine::ScanResult, scan_settings::ScanSettings};
use rocket::{Data, State};

struct ClamScanner {
    scanner: engine::Engine,
}

fn init_clam() -> engine::Engine {
    clamav_rs::initialize().expect("failt to init");

    let scanner = engine::Engine::new();
    scanner.load_databases(&db::default_directory()).expect("failed to load default");
    scanner.compile().expect("failt to compile");
    scanner
}

fn scan(scanner: &engine::Engine, settings: &mut ScanSettings, file_path: &str) -> bool {
    let hit = scanner.scan_file(file_path, settings)
        .expect("expected scan to succeed");

    match hit {
        ScanResult::Virus(name) => false,
        ScanResult::Clean => true,
        ScanResult::Whitelisted => true,
    }
}

#[post("/upload", data = "<file>")]
fn upload(file: Data, clam: State<ClamScanner>) -> Result<String, std::io::Error> {
    file.stream_to_file(Path::new("test"))?;
    let mut settings: ScanSettings = Default::default();
    if scan(&clam.scanner, &mut settings, "test") {
        Ok("safe".to_string())
    } else {
        Ok("not safe".to_string())
    }
}

fn main() {
    let scanner = init_clam();
    // let file_one = "/home/ian/Downloads/sample-pdf-download-10-mb.pdf";
    // let file_two = "/home/ian/git/hub/pard68/clamchowder/test";
    // scan(&scanner, &mut settings, file_one);
    // scan(&scanner, &mut settings, file_two);
    rocket::ignite()
        .mount("/", routes![upload])
        .manage(ClamScanner{ scanner: scanner })
        .launch();
}
