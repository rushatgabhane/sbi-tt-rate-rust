use std::thread;
use std::time::Duration;

use anyhow::{bail, Context, Result};

const RATE_PDF_URL: &str = "https://sbi.bank.in/documents/16012/1400784/FOREX_CARD_RATES.pdf";

const ATTEMPTS: u32 = 3;
const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) \
     Chrome/126.0.0.0 Safari/537.36";

/// Download the daily rates PDF, retrying with backoff. Guarantees the
/// returned bytes look like a PDF.
pub fn download_rates_pdf() -> Result<Vec<u8>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .build()
        .context("failed to build HTTP client")?;

    let mut last_error = None;
    for attempt in 1..=ATTEMPTS {
        match try_download(&client, RATE_PDF_URL) {
            Ok(bytes) => return Ok(bytes),
            Err(e) => {
                eprintln!("attempt {attempt}/{ATTEMPTS} failed: {e:#}");
                last_error = Some(e);
                thread::sleep(Duration::from_secs(3 * u64::from(attempt)));
            }
        }
    }

    Err(last_error.unwrap()).context("unable to download a valid PDF from SBI")
}

fn try_download(client: &reqwest::blocking::Client, url: &str) -> Result<Vec<u8>> {
    let response = client.get(url).send()?.error_for_status()?;
    let bytes = response.bytes()?.to_vec();

    if !bytes.starts_with(b"%PDF") {
        bail!("response is not a PDF ({} bytes)", bytes.len());
    }
    Ok(bytes)
}
