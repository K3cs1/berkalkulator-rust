use gettext::Catalog;
use slint::{SharedString, Weak};
use std::collections::HashMap;
use std::fs::File;

slint::include_modules!();

const ONE: f64 = 1.00;
const ZERO: f64 = 0.00;
const ONE_HUNDRED_MILLION: f64 = 100000000.00;

enum Jarulek {
    NyugdijBizt(f64),
    PenzbeniEgeszsegBizt(f64),
    TermeszetbeniEgeszsegBizt(f64),
    SZJA(f64),
    MunkaeroPiaci(f64),
}

struct Berkalkulator {}

impl Berkalkulator {
    fn calculate_netto_ber(brutto_ber: f64, catalog: Catalog) -> Result<String, String> {
        if brutto_ber <= ZERO {
            let error_msg = catalog.gettext("A megadott érték kisebb mint egy!");
            return Err(error_msg.to_owned());
        }
        if brutto_ber > ONE_HUNDRED_MILLION {
            let error_msg = catalog.gettext("A megadott érték túl magas!");
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
                    sum_of_jarulekok = sum_of_jarulekok + amount;
                    calculated_jarulekok_map.insert("szja", brutto_ber * amount);
                }
                Jarulek::MunkaeroPiaci(amount) => {
                    sum_of_jarulekok = sum_of_jarulekok + amount;
                    calculated_jarulekok_map.insert("munkaero_piaci", brutto_ber * amount);
                }
            }
        }
        let netto_num: f64 = brutto_ber * (ONE - sum_of_jarulekok);
        let jarulekok_text = catalog.gettext("Járulékok");
        let nyugdij_bizt_jarulek_text = catalog.gettext("Nyugdíj-biztosítási járulék");
        let penzbeni_egeszsegbizt_jarulek_text =
            catalog.gettext("Pénzbeni Egészségbiztosítási járulék");
        let termeszetbeni_egeszsegbizt_jarulek_text =
            catalog.gettext("Természetbeni Egészségbiztosítási járulék");
        let szja_text = catalog.gettext("SZJA (személyi jövedelemadó)");
        let munkaero_piaci_jarulek_text = catalog.gettext("Munkaerő-piaci járulék");
        let netto_havi_ber_text = catalog.gettext("Nettó havi bér");
        let result: String = format!("{}: \n\n{}: {:.2} Ft\n{}: {:.2} Ft\n{}: {:.2} Ft\n{}: {:.2} Ft\n{}: {:.2} Ft\n\n{}: {:.2} Ft", jarulekok_text, nyugdij_bizt_jarulek_text, calculated_jarulekok_map.get("nyugdij_bizt").unwrap(), penzbeni_egeszsegbizt_jarulek_text, calculated_jarulekok_map.get("penzbeni_egeszseg_bizt").unwrap(), termeszetbeni_egeszsegbizt_jarulek_text, calculated_jarulekok_map.get("term_egeszseg_bizt").unwrap(), szja_text, calculated_jarulekok_map.get("szja").unwrap(), munkaero_piaci_jarulek_text, calculated_jarulekok_map.get("munkaero_piaci").unwrap(), netto_havi_ber_text, netto_num);
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
    slint::init_translations!(concat!(env!("CARGO_MANIFEST_DIR"), "/i18n/"));


    let ui: AppWindow = AppWindow::new()?;
    let ui_handle: Weak<AppWindow> = ui.as_weak();
    ui.on_divide_income(move |string: SharedString| {
        let ui: AppWindow = ui_handle.unwrap();
        let brutto_ber: f64 = string.trim().parse().unwrap();

    let mo_file_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/i18n/en/LC_MESSAGES/berkalkulator-rust.mo"
    );
    let file = File::open(mo_file_path).expect("could not open the catalog");
    let catalog = Catalog::parse(file).expect("could not parse the catalog");
    let berkalkulator = Berkalkulator::calculate_netto_ber(brutto_ber, catalog);
    match berkalkulator {
        Ok(response) => ui.set_results(response.into()),
        Err(e) => ui.set_results(e.into()),
    }
    });
    ui.run()
}
