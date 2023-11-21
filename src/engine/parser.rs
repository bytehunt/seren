use crate::engine::dirs::*;
use crate::engine::pkg_info::PackageInfo;
use anyhow::{anyhow, Result};
use reqwest;
use reqwest::StatusCode;
use spinoff::{spinners, Color, Spinner, Streams};
use std::fs::File;
use std::io::prelude::*;

fn build_package_toml_url(pkg_name: &str) -> String {
    let repo_url =
        std::env::var("SEREN_REPO_URL").unwrap_or_else(|_| "bytehunt/seren-pkgs".to_string());
    format!(
        "https://raw.githubusercontent.com/{}/main/data/{}.toml",
        repo_url, pkg_name
    )
}

pub async fn fetch_package_info(pkg_name: &str) -> Result<PackageInfo> {
    let pkg_toml_file_url = build_package_toml_url(pkg_name);

    let spinner = Spinner::new_with_stream(
        spinners::Line,
        "Fetching package info ... ",
        Color::Yellow,
        Streams::Stderr,
    );

    let response = reqwest::get(&pkg_toml_file_url)
        .await
        .map_err(|e| anyhow!("Failed to fetch package: {}", e))?;

    match response.status() {
        StatusCode::OK => {
            let toml_text = response.text().await?;
            let parsed_toml: PackageInfo = toml::from_str(&toml_text)?;
            // dbg!("{}", &parsed_toml);
            let file_name = format!("{}.toml", pkg_name);
            let data_dir = &*SEREN_DATA_DIR;
            let file_path = data_dir.join(&file_name);

            let mut file = File::create(&file_path)?;
            file.write_all(toml_text.as_bytes())?;
            spinner.stop_and_persist("  ", "Done");
            Ok(parsed_toml)
        }
        _ => Err(anyhow!("No such package found")),
    }
}