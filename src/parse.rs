use std::io::Write;
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use regex::Regex;

use crate::{CurrencyRates, RateSheet};

const REFERENCE_MARKER: &str = "to be used as reference rates";

/// Parse the rate sheet out of the PDF bytes. Tries the pure-Rust extractor
/// first and falls back to poppler's `pdftotext` (if installed) when that
/// fails, since the two handle malformed PDFs differently.
pub fn parse_pdf(bytes: &[u8]) -> Result<RateSheet> {
    let rust_result = extract_text_rust(bytes).and_then(|text| parse_text(&text));
    match rust_result {
        Ok(sheet) => Ok(sheet),
        Err(rust_err) => {
            eprintln!("pdf-extract path failed ({rust_err:#}), trying pdftotext");
            extract_text_poppler(bytes)
                .and_then(|text| parse_text(&text))
                .map_err(|poppler_err| {
                    anyhow!("pdf-extract: {rust_err:#}; pdftotext: {poppler_err:#}")
                })
        }
    }
}

fn extract_text_rust(bytes: &[u8]) -> Result<String> {
    // pdf-extract is known to panic on some malformed PDFs, not just error.
    std::panic::catch_unwind(|| pdf_extract::extract_text_from_mem(bytes))
        .map_err(|_| anyhow!("pdf-extract panicked"))?
        .context("pdf-extract failed")
}

fn extract_text_poppler(bytes: &[u8]) -> Result<String> {
    let path = std::env::temp_dir().join(format!("sbi-tt-rate-{}.pdf", std::process::id()));
    std::fs::File::create(&path)?.write_all(bytes)?;

    let output = Command::new("pdftotext")
        .args(["-layout", path.to_str().unwrap(), "-"])
        .output()
        .context("failed to run pdftotext (is poppler installed?)")?;
    let _ = std::fs::remove_file(&path);

    if !output.status.success() {
        bail!("pdftotext exited with {}", output.status);
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Parse the extracted PDF text into a rate sheet.
pub fn parse_text(text: &str) -> Result<RateSheet> {
    if !text.to_lowercase().contains(REFERENCE_MARKER) {
        bail!("reference-rates marker text not found; not a rate sheet we understand");
    }

    let published_at = extract_date_time(text)?;
    let rates = extract_currency_rates(text);
    if rates.is_empty() {
        bail!("no currency rate lines found");
    }

    Ok(RateSheet {
        published_at,
        rates,
    })
}

/// SBI publishes dates day-first (e.g. `31-12-2025`), with `Date` and `Time`
/// labels somewhere on the first page.
fn extract_date_time(text: &str) -> Result<NaiveDateTime> {
    let date_re = Regex::new(r"(?i)date\s*:?\s*(\d{1,2})[-/. ](\d{1,2})[-/. ](\d{2,4})").unwrap();
    let time_re = Regex::new(r"(?i)time\s*:?\s*(\d{1,2}):(\d{2})(?::\d{2})?\s*(am|pm)?").unwrap();

    let date_caps = date_re
        .captures(text)
        .context("date not found in PDF text")?;
    let (day, month, mut year): (u32, u32, i32) = (
        date_caps[1].parse()?,
        date_caps[2].parse()?,
        date_caps[3].parse()?,
    );
    if year < 100 {
        year += 2000;
    }
    let date = NaiveDate::from_ymd_opt(year, month, day)
        .with_context(|| format!("invalid date {day}-{month}-{year}"))?;

    let time_caps = time_re
        .captures(text)
        .context("time not found in PDF text")?;
    let mut hour: u32 = time_caps[1].parse()?;
    let minute: u32 = time_caps[2].parse()?;
    match time_caps.get(3).map(|m| m.as_str().to_lowercase()) {
        Some(ref meridiem) if meridiem == "pm" && hour != 12 => hour += 12,
        Some(ref meridiem) if meridiem == "am" && hour == 12 => hour = 0,
        _ => {}
    }
    let time = NaiveTime::from_hms_opt(hour, minute, 0)
        .with_context(|| format!("invalid time {hour}:{minute}"))?;

    Ok(NaiveDateTime::new(date, time))
}

/// Pull out lines like `UNITED STATES DOLLAR  USD/INR  89.47 90.32 ...`,
/// keeping only rows that carry all 8 rate columns. Extraction sometimes
/// drops the space after the currency pair, so numbers are matched anywhere
/// after `XXX/INR` rather than split on whitespace.
fn extract_currency_rates(text: &str) -> Vec<CurrencyRates> {
    let currency_re = Regex::new(r"([A-Z]{3})/INR").unwrap();
    let number_re = Regex::new(r"\d+(?:\.\d+)?").unwrap();

    let mut rates = Vec::new();
    for line in text.lines() {
        let Some(caps) = currency_re.captures(line) else {
            continue;
        };
        let after_pair = &line[caps.get(0).unwrap().end()..];
        let numbers: Vec<String> = number_re
            .find_iter(after_pair)
            .map(|m| m.as_str().to_string())
            .collect();

        if numbers.len() == 8 {
            rates.push(CurrencyRates {
                currency: caps[1].to_string(),
                rates: numbers,
            });
        } else {
            eprintln!(
                "skipping {} line with {} rate values (expected 8)",
                &caps[1],
                numbers.len()
            );
        }
    }
    rates
}
