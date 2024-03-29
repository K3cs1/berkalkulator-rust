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

const ONE: f64 = 1.00;
const ZERO: f64 = 0.00;
const ONE_HUNDRED_MILLION: f64 = 100000000.00;
const FIVE_THOUSAND: f64 = 5000.00;
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

struct Berkalkulator {    
}

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
            let error_msg = catalog.gettext("Given value to high!");
            warn!("{}", error_msg);
            return Err(error_msg.to_owned());
        }
        let jarulekok: Vec<Jarulek> = Self::init_jarulekok();
        let mut sum_of_jarulekok: f64 = ZERO;
        let mut calculated_jarulekok_map: HashMap<String, f64> = HashMap::new();
        for jarulek in jarulekok.iter() {
            match jarulek {
                Jarulek::NyugdijBizt(amount) => {
                    sum_of_jarulekok = sum_of_jarulekok + amount;
                    calculated_jarulekok_map.insert("nyugdij_bizt".to_owned(), brutto_ber * amount);
                }
                Jarulek::PenzbeniEgeszsegBizt(amount) => {
                    sum_of_jarulekok = sum_of_jarulekok + amount;
                    calculated_jarulekok_map.insert("penzbeni_egeszseg_bizt".to_owned(), brutto_ber * amount);
                }
                Jarulek::TermeszetbeniEgeszsegBizt(amount) => {
                    sum_of_jarulekok = sum_of_jarulekok + amount;
                    calculated_jarulekok_map.insert("term_egeszseg_bizt".to_owned(), brutto_ber * amount);
                }
                Jarulek::SZJA(amount) => {
                    let mut calculated_szja;
                    sum_of_jarulekok = sum_of_jarulekok + amount;
                    if friss_hazas == true {
                        calculated_szja = (brutto_ber * amount) - FIVE_THOUSAND;
                    } else {
                        calculated_szja = brutto_ber * amount;
                    }
                    if szja_mentes == true {
                        calculated_szja = ZERO;
                        sum_of_jarulekok = sum_of_jarulekok - amount;
                    }
                    calculated_jarulekok_map.insert("szja".to_owned(), calculated_szja);
                }
                Jarulek::MunkaeroPiaci(amount) => {
                    sum_of_jarulekok = sum_of_jarulekok + amount;
                    calculated_jarulekok_map.insert("munkaero_piaci".to_owned(), brutto_ber * amount);
                }
            }
        }
        let mut netto_num: f64 = brutto_ber * (ONE - sum_of_jarulekok);
        if friss_hazas == true {
            netto_num = netto_num - FIVE_THOUSAND;
        }

        let jarulekok_text = catalog.gettext("Contributions");
        let nyugdij_bizt_jarulek_text = catalog.gettext("Pension insurance contribution");
        let penzbeni_egeszsegbizt_jarulek_text = catalog.gettext("Cash Health Insurance contribution");
        let termeszetbeni_egeszsegbizt_jarulek_text = catalog.gettext("Health insurance contribution in kind");
        let szja_text = catalog.gettext("SJJA (personal income tax)");
        let munkaero_piaci_jarulek_text = catalog.gettext("Labor market contribution");
        let netto_havi_ber_text = catalog.gettext("Net monthly salary");
        let result: String = format!("{}: \n\n{}: {:.2} Ft\n{}: {:.2} Ft\n{}: {:.2} Ft\n{}: {:.2} Ft\n{}: {:.2} Ft\n\n{}: {:.2} Ft", 
        jarulekok_text, 
        nyugdij_bizt_jarulek_text, 
        calculated_jarulekok_map.get("nyugdij_bizt").unwrap(), 
        penzbeni_egeszsegbizt_jarulek_text, 
        calculated_jarulekok_map.get("penzbeni_egeszseg_bizt").unwrap(), 
        termeszetbeni_egeszsegbizt_jarulek_text, 
        calculated_jarulekok_map.get("term_egeszseg_bizt").unwrap(), 
        szja_text, 
        calculated_jarulekok_map.get("szja").unwrap(), 
        munkaero_piaci_jarulek_text, 
        calculated_jarulekok_map.get("munkaero_piaci").unwrap(), 
        netto_havi_ber_text, 
        netto_num);

        info!("{}", result);
        if csv_export == true {
            let res = Self::export_calculations_to_csv(&calculated_jarulekok_map, &netto_num);
            match res {
                Ok(()) => info!("export.csv created"),
                Err(message) => warn!("{}", message)
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

    fn export_calculations_to_csv(calculated_jarulekok_map: &HashMap<String, f64>, netto_num: &f64) -> Result<(), Box<dyn Error>> {
        let mut writer = Writer::from_path(CSV_PATH)?;
        writer.serialize(CsvRecord{
            labor_market: *calculated_jarulekok_map.get("munkaero_piaci").unwrap(),
            cash_health_insurance: *calculated_jarulekok_map.get("penzbeni_egeszseg_bizt").unwrap(),
            health_insurance: *calculated_jarulekok_map.get("term_egeszseg_bizt").unwrap(),
            sjja: *calculated_jarulekok_map.get("szja").unwrap(),
            pension_insurance: *calculated_jarulekok_map.get("nyugdij_bizt").unwrap(),
            net_monthly_salary: *netto_num
        })?;

        writer.flush()?;

        Ok(())
    }
}

fn main() -> Result<(), slint::PlatformError> {
    std::env::set_var("LANG", "hu");
    slint::init_translations!(concat!(env!("CARGO_MANIFEST_DIR"), "/i18n/"));
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let ui: AppWindow = AppWindow::new()?;
    let ui_handle: Weak<AppWindow> = ui.as_weak();
    ui.on_divide_income(
        move |string: SharedString, friss_hazas: bool, szja_mentes: bool, csv_export: bool| {
            let ui: AppWindow = ui_handle.unwrap();
            let brutto_ber = string.trim().parse();
            let mut brutto_ber_num: f64 = ZERO;
            match brutto_ber {
                Ok(response) => {
                    brutto_ber_num = response;
                }
                Err(e) => {
                    warn!("{}", e.to_string());
                    ui.set_results(e.to_string().into())
                }
            }

            let locale = get_locale().unwrap_or_else(|| String::from("hu-HU"));
            info!("{}", locale);

            let mo_file_path = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/i18n/hu/LC_MESSAGES/berkalkulator-rust.mo"
            );
            let file = File::open(mo_file_path).expect("could not open the catalog");
            let catalog = Catalog::parse(file).expect("could not parse the catalog");

            let berkalkulator = Berkalkulator::calculate_netto_ber(
                brutto_ber_num,
                friss_hazas,
                szja_mentes,
                csv_export,
                catalog,
            );
            match berkalkulator {
                Ok(response) => ui.set_results(response.into()),
                Err(e) => {
                    warn!("Error during calculation");
                    ui.set_results(e.into())
                }
            }
        },
    );
    ui.run()
}
