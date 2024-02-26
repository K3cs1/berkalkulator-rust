use slint::{SharedString, Weak};
use std::collections::HashMap;

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
    fn calculate_netto_ber(brutto_ber: f64) -> Result<String, String> {
        if brutto_ber <= ZERO {
            return Err("A megadott érték kisebb mint egy!".to_owned());
        }
        if brutto_ber > ONE_HUNDRED_MILLION {
            return Err("A megadott érték túl magas!".to_owned());
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
        let result: String = format!("Járulékok: \n\nNyugdíj-biztosítási járulék: {:.2} Ft\nPénzbeni Egészségbiztosítási járulék: {:.2} Ft\nTermészetbeni Egészségbiztosítási járulék: {:.2} Ft\nSZJA (személyi jövedelemadó): {:.2} Ft\nMunkaerő-piaci járulék: {:.2} Ft\n\nNettó havi bér: {:.2} Ft", calculated_jarulekok_map.get("nyugdij_bizt").unwrap(), calculated_jarulekok_map.get("penzbeni_egeszseg_bizt").unwrap(), calculated_jarulekok_map.get("term_egeszseg_bizt").unwrap(), calculated_jarulekok_map.get("szja").unwrap(), calculated_jarulekok_map.get("munkaero_piaci").unwrap(), netto_num);
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
        let berkalkulator = Berkalkulator::calculate_netto_ber(brutto_ber);
        match berkalkulator {
            Ok(response) => ui.set_results(response.into()),
            Err(e) => ui.set_results(e.into()),
        }
    });
    ui.run()
}
