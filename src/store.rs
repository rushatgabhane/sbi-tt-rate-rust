use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::NaiveDateTime;

use crate::{RateSheet, CSV_DATE_FORMAT, RATE_COLUMNS};

/// Save the day's PDF under `pdf_files/<year>/<month>/<YYYY-MM-DD>.pdf`
/// (unpadded month, matching sahilgupta/sbi-fx-ratekeeper's layout).
pub fn save_pdf(bytes: &[u8], published_at: NaiveDateTime, root: &Path) -> Result<PathBuf> {
    let dir = root
        .join("pdf_files")
        .join(published_at.format("%Y").to_string())
        .join(published_at.format("%-m").to_string());
    fs::create_dir_all(&dir)?;

    let path = dir.join(format!("{}.pdf", published_at.format("%Y-%m-%d")));
    fs::write(&path, bytes).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(path)
}

/// Upsert the sheet's rates into `csv_files/SBI_REFERENCE_RATES_<CCY>.csv`,
/// one file per currency, deduplicated by DATE and sorted chronologically.
/// Layout matches sahilgupta/sbi-fx-ratekeeper so the files are drop-in
/// compatible.
pub fn update_csvs(sheet: &RateSheet, root: &Path) -> Result<()> {
    let csv_dir = root.join("csv_files");
    fs::create_dir_all(&csv_dir)?;

    let date_value = sheet.published_at.format(CSV_DATE_FORMAT).to_string();
    let pdf_link = pdf_link(sheet.published_at);
    let headers: Vec<String> = ["DATE", "PDF FILE"]
        .iter()
        .map(|s| s.to_string())
        .chain(RATE_COLUMNS.iter().map(|s| s.to_string()))
        .collect();

    for currency in &sheet.rates {
        let path = csv_dir.join(format!("SBI_REFERENCE_RATES_{}.csv", currency.currency));

        // Keyed by parsed DATE so rewriting is dedup + sort in one pass.
        let mut rows: BTreeMap<NaiveDateTime, Vec<String>> = BTreeMap::new();
        if path.exists() {
            let mut reader = csv::Reader::from_path(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            for record in reader.records() {
                let record = record?;
                let row: Vec<String> = record.iter().map(str::to_string).collect();
                let parsed = NaiveDateTime::parse_from_str(&row[0], CSV_DATE_FORMAT)
                    .with_context(|| format!("bad DATE '{}' in {}", row[0], path.display()))?;
                rows.insert(parsed, row);
            }
        }

        let mut new_row = vec![date_value.clone(), pdf_link.clone()];
        new_row.extend(currency.rates.iter().cloned());
        rows.insert(sheet.published_at, new_row);

        let mut writer = csv::Writer::from_path(&path)
            .with_context(|| format!("failed to write {}", path.display()))?;
        writer.write_record(&headers)?;
        for row in rows.values() {
            writer.write_record(row)?;
        }
        writer.flush()?;
    }

    Ok(())
}

/// Link recorded in the PDF FILE column. Points at the GitHub blob when
/// running in Actions, otherwise the repo-relative path.
fn pdf_link(published_at: NaiveDateTime) -> String {
    let relative = format!(
        "pdf_files/{}/{}/{}.pdf",
        published_at.format("%Y"),
        published_at.format("%-m"),
        published_at.format("%Y-%m-%d")
    );
    match std::env::var("GITHUB_REPOSITORY") {
        Ok(repo) if !repo.is_empty() => {
            format!("https://github.com/{repo}/blob/main/{relative}")
        }
        _ => relative,
    }
}
