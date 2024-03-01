use gettext::Catalog;
use gettext_macros::{i18n, init_i18n};
use log::{info, warn};
use slint::{SharedString, Weak};
use std::collections::HashMap;
use std::fs::File;
use sys_locale::get_locale;

slint::include_modules!();

const ONE: f64 = 1.00;
const ZERO: f64 = 0.00;
const ONE_HUNDRED_MILLION: f64 = 100000000.00;
const FIVE_THOUSAND: f64 = 5000.00;

enum Jarulek {
    NyugdijBizt(f64),
    PenzbeniEgeszsegBizt(f64),
    TermeszetbeniEgeszsegBizt(f64),
    SZJA(f64),
    MunkaeroPiaci(f64),
}

struct Berkalkulator {}

impl Berkalkulator {
    fn calculate_netto_ber(
        brutto_ber: f64,
        friss_hazas: bool,
        szja_mentes: bool,
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
        let mut calculated_jarulekok_map: HashMap<&str, f64> = HashMap::new();
        for jarulek in jarulekok.iter() {
            match jarulek {
                Jarulek::NyugdijBizt(amount) => {
                    sum_of_jarulekok = sum_of_jarulekok + amount;
                    calculated_jarulekok_map.insert("nyugdij_bizt", brutto_ber * amount);
                }
                Jarulek::PenzbeniEgeszsegBizt(amount) => {
                    sum_of_jarulekok = sum_of_jarulekok + amount;
                    calculated_jarulekok_map.insert("penzbeni_egeszseg_bizt", brutto_ber * amount);
                }
                Jarulek::TermeszetbeniEgeszsegBizt(amount) => {
                    sum_of_jarulekok = sum_of_jarulekok + amount;
                    calculated_jarulekok_map.insert("term_egeszseg_bizt", brutto_ber * amount);
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
                    calculated_jarulekok_map.insert("szja", calculated_szja);
                }
                Jarulek::MunkaeroPiaci(amount) => {
                    sum_of_jarulekok = sum_of_jarulekok + amount;
                    calculated_jarulekok_map.insert("munkaero_piaci", brutto_ber * amount);
                }
            }
        }
        let mut netto_num: f64 = brutto_ber * (ONE - sum_of_jarulekok);
        if friss_hazas == true {
            netto_num = netto_num - FIVE_THOUSAND;
        }

        let jarulekok_text = i18n!(catalog, "Contributions");
        let nyugdij_bizt_jarulek_text = i18n!(catalog, "Pension insurance contribution");
        let penzbeni_egeszsegbizt_jarulek_text = i18n!(catalog, "Cash Health Insurance contribution");
        let termeszetbeni_egeszsegbizt_jarulek_text = i18n!(catalog, "Health insurance contribution in kind");
        let szja_text = i18n!(catalog, "SJJA (personal income tax)");
        let munkaero_piaci_jarulek_text = i18n!(catalog, "Labor market contribution");
        let netto_havi_ber_text = i18n!(catalog, "Net monthly salary");
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
}

fn main() -> Result<(), slint::PlatformError> {
    init_i18n!("berkalkulator-rust", en, hu);
    slint::init_translations!(concat!(env!("CARGO_MANIFEST_DIR"), "/i18n/"));

    let ui: AppWindow = AppWindow::new()?;
    let ui_handle: Weak<AppWindow> = ui.as_weak();
    ui.on_divide_income(
        move |string: SharedString, friss_hazas: bool, szja_mentes: bool| {
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
