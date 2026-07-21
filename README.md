# SBI TT Rate

Archives the State Bank of India's daily forex card rates — the TT buy/sell
rates used as reference rates under [Rule 26 of the Income Tax Rules, 1962](https://incometaxindia.gov.in/_layouts/15/dit/pages/viewer.aspx?grp=rule&cname=cmsid&cval=103120000000007372&searchfilter=)
for computing foreign income and capital gains. SBI publishes the rates as a
PDF each working day but offers no historical archive, so this repo keeps one.

A GitHub Action runs twice a day (16:00 and 22:00 IST), downloads
`FOREX_CARD_RATES.pdf` from SBI, and commits:

- **`pdf_files/<year>/<month>/<YYYY-MM-DD>.pdf`** — the original PDF, for verification
- **`csv_files/SBI_REFERENCE_RATES_<CCY>.csv`** — one CSV per currency, one row
  per publication, with columns `DATE, PDF FILE, TT BUY, TT SELL, BILL BUY,
  BILL SELL, FOREX TRAVEL CARD BUY, FOREX TRAVEL CARD SELL, CN BUY, CN SELL`

The CSV layout is identical to [sahilgupta/sbi-fx-ratekeeper](https://github.com/sahilgupta/sbi-fx-ratekeeper),
so files from the two repos are interchangeable. If a run fails (SBI down,
PDF unparseable), the workflow opens a GitHub issue; an unparseable PDF is
still committed under `pdf_files/unparsed/` for later re-parsing.

> **Note:** only the rates published for the ₹10–20 lakh transaction range are
> reference rates; they do not change with your transaction value.

## Running locally

```sh
cargo run --release   # downloads today's PDF, updates csv_files/ and pdf_files/
cargo test            # parser + storage tests against a bundled sample PDF
```

## Credits

Historical data (January 2020 – July 2026) was backfilled from
[sahilgupta/sbi-fx-ratekeeper](https://github.com/sahilgupta/sbi-fx-ratekeeper),
which in turn credits [skbly7/sbi-tt-rates-historical](https://github.com/skbly7/sbi-tt-rates-historical),
the forex rate card archives of Maneesh K. Singh & Co., and the Internet
Archive Wayback Machine for pre-Dec-2022 data.

## Structure

```
src/fetch.rs   HTTP download: retries, %PDF check
src/parse.rs   PDF text extraction, date/time + currency-rate parsing
src/store.rs   PDF archiving and per-currency CSV upsert (dedup by DATE, sorted)
src/main.rs    fetch → parse → store
```
