use std::path::Path;
use std::process::ExitCode;

use sbi_tt_rate::{fetch, parse, store};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> anyhow::Result<()> {
    let root = Path::new(".");

    let pdf_bytes = fetch::download_rates_pdf()?;
    println!("downloaded rates PDF ({} bytes)", pdf_bytes.len());

    let sheet = match parse::parse_pdf(&pdf_bytes) {
        Ok(sheet) => sheet,
        Err(e) => {
            // Keep the PDF so it gets committed and can be re-parsed later.
            let path = store::save_unparsed_pdf(&pdf_bytes, root)?;
            eprintln!("saved unparseable PDF to {}", path.display());
            return Err(e);
        }
    };

    let pdf_path = store::save_pdf(&pdf_bytes, sheet.published_at, root)?;
    store::update_csvs(&sheet, root)?;

    println!(
        "saved {} and rates for {} currencies (published {})",
        pdf_path.display(),
        sheet.rates.len(),
        sheet.published_at
    );
    Ok(())
}
