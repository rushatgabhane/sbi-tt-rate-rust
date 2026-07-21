use chrono::NaiveDate;
use sbi_tt_rate::{parse, store, RateSheet};

fn fixture_sheet() -> RateSheet {
    let bytes = include_bytes!("fixtures/sample.pdf");
    parse::parse_pdf(bytes).expect("fixture PDF should parse")
}

#[test]
fn parses_fixture_pdf() {
    let sheet = fixture_sheet();

    assert_eq!(
        sheet.published_at,
        NaiveDate::from_ymd_opt(2025, 12, 31)
            .unwrap()
            .and_hms_opt(9, 18, 0)
            .unwrap()
    );

    // The sheet lists 31 currency rows; every kept row must have all 8 rates.
    assert!(sheet.rates.len() >= 25, "got {} rows", sheet.rates.len());
    for row in &sheet.rates {
        assert_eq!(row.rates.len(), 8, "{} row incomplete", row.currency);
    }

    let usd = sheet
        .rates
        .iter()
        .find(|r| r.currency == "USD")
        .expect("USD row present");
    assert_eq!(usd.rates[0], "89.47"); // TT BUY
    assert_eq!(usd.rates[1], "90.32"); // TT SELL
}

#[test]
fn rejects_text_without_reference_marker() {
    assert!(parse::parse_text("Date 31-12-2025\nTime 9:18 AM\nUSD/INR 1 2 3 4 5 6 7 8").is_err());
}

#[test]
fn csv_upsert_is_idempotent() {
    let dir = std::env::temp_dir().join(format!("sbi-tt-rate-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let sheet = fixture_sheet();
    store::update_csvs(&sheet, &dir).unwrap();
    store::update_csvs(&sheet, &dir).unwrap();

    let usd_csv =
        std::fs::read_to_string(dir.join("csv_files/SBI_REFERENCE_RATES_USD.csv")).unwrap();
    let lines: Vec<&str> = usd_csv.lines().collect();
    assert_eq!(lines.len(), 2, "header + exactly one row:\n{usd_csv}");
    assert!(lines[0].starts_with("DATE,PDF FILE,TT BUY,TT SELL"));
    assert!(lines[1].starts_with("2025-12-31 09:18,"));
    assert!(lines[1].contains("89.47"));

    let _ = std::fs::remove_dir_all(&dir);
}
