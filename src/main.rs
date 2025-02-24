use gettext::Catalog;
use log::{info, warn};
use slint::{SharedString, Weak};
use std::collections::HashMap;
use std::fs::File;
use std::error::Error;
use csv::Writer;
use sys_locale::get_locale;
use serde::{Serialize, Deserialize};

slint::include_modules!();

const ONE: f64 = 1.0;
const ZERO: f64 = 0.0;
const ONE_HUNDRED_MILLION: f64 = 100_000_000.0;
const FIVE_THOUSAND: f64 = 5_000.0;
const CSV_PATH: &str = "export.csv";

enum Jarulek {
    NyugdijBizt(f64),
    PenzbeniEgeszsegBizt(f64),
    TermeszetbeniEgeszsegBizt(f64),
    SZJA(f64),
    MunkaeroPiaci(f64),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct CsvRecord {
    net_monthly_salary: f64,
    pension_insurance: f64,
    cash_health_insurance: f64,
    health_insurance: f64,
    sjja: f64,
    labor_market: f64,
}

struct Berkalkulator;

impl Berkalkulator {
    fn calculate_netto_ber(
        brutto_ber: f64,
        friss_hazas: bool,
        szja_mentes: bool,
        csv_export: bool,
        catalog: Catalog,
    ) -> Result<String, String> {
        if brutto_ber <= ZERO {
            let error_msg = catalog.gettext("Given value less than one!");
            warn!("{}", error_msg);
            return Err(error_msg.to_owned());
        }
        if brutto_ber > ONE_HUNDRED_MILLION {
            let error_msg = catalog.gettext("Given value too high!");
            warn!("{}", error_msg);
            return Err(error_msg.to_owned());
        }

        let jarulekok = Self::init_jarulekok();
        let mut sum_of_jarulekok = ZERO;
        let mut calculated_jarulekok_map: HashMap<&str, f64> = HashMap::new();

        for jarulek in jarulekok.iter() {
            match jarulek {
                Jarulek::SZJA(rate) => {
                    let mut contribution = brutto_ber * rate;
                    if friss_hazas {
                        contribution -= FIVE_THOUSAND;
                    }
                    if szja_mentes {
                        contribution = ZERO;
                        // Do not add the SZJA rate if tax-exempt
                    } else {
                        sum_of_jarulekok += rate;
                    }
                    calculated_jarulekok_map.insert("szja", contribution);
                }
                Jarulek::NyugdijBizt(rate) => {
                    sum_of_jarulekok += rate;
                    calculated_jarulekok_map.insert("nyugdij_bizt", brutto_ber * rate);
                }
                Jarulek::PenzbeniEgeszsegBizt(rate) => {
                    sum_of_jarulekok += rate;
                    calculated_jarulekok_map.insert("penzbeni_egeszseg_bizt", brutto_ber * rate);
                }
                Jarulek::TermeszetbeniEgeszsegBizt(rate) => {
                    sum_of_jarulekok += rate;
                    calculated_jarulekok_map.insert("term_egeszseg_bizt", brutto_ber * rate);
                }
                Jarulek::MunkaeroPiaci(rate) => {
                    sum_of_jarulekok += rate;
                    calculated_jarulekok_map.insert("munkaero_piaci", brutto_ber * rate);
                }
            }
        }

        let mut netto_num = brutto_ber * (ONE - sum_of_jarulekok);
        if friss_hazas {
            netto_num -= FIVE_THOUSAND;
        }

        let result = format!(
            "{}: \n\n{}: {:.2} Ft\n{}: {:.2} Ft\n{}: {:.2} Ft\n{}: {:.2} Ft\n{}: {:.2} Ft\n\n{}: {:.2} Ft",
            catalog.gettext("Contributions"),
            catalog.gettext("Pension insurance contribution"),
            calculated_jarulekok_map.get("nyugdij_bizt").unwrap(),
            catalog.gettext("Cash Health Insurance contribution"),
            calculated_jarulekok_map.get("penzbeni_egeszseg_bizt").unwrap(),
            catalog.gettext("Health insurance contribution in kind"),
            calculated_jarulekok_map.get("term_egeszseg_bizt").unwrap(),
            catalog.gettext("SJJA (personal income tax)"),
            calculated_jarulekok_map.get("szja").unwrap(),
            catalog.gettext("Labor market contribution"),
            calculated_jarulekok_map.get("munkaero_piaci").unwrap(),
            catalog.gettext("Net monthly salary"),
            netto_num
        );

        info!("{}", result);
        if csv_export {
            match Self::export_calculations_to_csv(&calculated_jarulekok_map, netto_num) {
                Ok(()) => info!("export.csv created"),
                Err(err) => warn!("{}", err),
            }
        }
        Ok(result)
    }

    fn init_jarulekok() -> Vec<Jarulek> {
        vec![
            Jarulek::NyugdijBizt(0.10),
            Jarulek::PenzbeniEgeszsegBizt(0.03),
            Jarulek::TermeszetbeniEgeszsegBizt(0.04),
            Jarulek::SZJA(0.15),
            Jarulek::MunkaeroPiaci(0.015),
        ]
    }

    fn export_calculations_to_csv(
        calculated_jarulekok_map: &HashMap<&str, f64>,
        netto_num: f64,
    ) -> Result<(), Box<dyn Error>> {
        let mut writer = Writer::from_path(CSV_PATH)?;
        let record = CsvRecord {
            pension_insurance: *calculated_jarulekok_map.get("nyugdij_bizt").unwrap(),
            cash_health_insurance: *calculated_jarulekok_map.get("penzbeni_egeszseg_bizt").unwrap(),
            health_insurance: *calculated_jarulekok_map.get("term_egeszseg_bizt").unwrap(),
            sjja: *calculated_jarulekok_map.get("szja").unwrap(),
            labor_market: *calculated_jarulekok_map.get("munkaero_piaci").unwrap(),
            net_monthly_salary: netto_num,
        };
        writer.serialize(record)?;
        writer.flush()?;
        Ok(())
    }
}

fn main() -> Result<(), slint::PlatformError> {
    unsafe {
        std::env::set_var("LANG", "hu");
        std::env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();
    slint::init_translations!(concat!(env!("CARGO_MANIFEST_DIR"), "/i18n/"));

    let ui: AppWindow = AppWindow::new()?;
    let ui_handle: Weak<AppWindow> = ui.as_weak();

    ui.on_divide_income(move |input: SharedString, friss_hazas: bool, szja_mentes: bool, csv_export: bool| {
        // Upgrade the weak reference to get a usable instance.
        let ui = match ui_handle.upgrade() {
            Some(ui) => ui,
            None => return, // If the UI is no longer available, exit the closure.
        };

        let brutto_ber_num: f64 = match input.trim().parse() {
            Ok(num) => num,
            Err(e) => {
                let err_msg = e.to_string();
                warn!("{}", err_msg);
                ui.set_results(err_msg.into());
                return;
            }
        };

        let locale = sys_locale::get_locale().unwrap_or_else(|| "hu-HU".to_string());
        info!("Locale: {}", locale);

        let mo_file_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/i18n/hu/LC_MESSAGES/berkalkulator-rust.mo"
        );
        let file = std::fs::File::open(mo_file_path)
            .expect("Could not open the catalog file");
        let catalog = gettext::Catalog::parse(file)
            .expect("Could not parse the catalog");

        match Berkalkulator::calculate_netto_ber(
            brutto_ber_num,
            friss_hazas,
            szja_mentes,
            csv_export,
            catalog,
        ) {
            Ok(result) => ui.set_results(result.into()),
            Err(err) => {
                warn!("Error during calculation: {}", err);
                ui.set_results(err.into());
            }
        }
    });
    ui.run()
}
