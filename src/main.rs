use slint::{SharedString, Weak};

slint::include_modules!();

const NYUGDIJ_BIZT_JARULEK: f64 = 0.10;
const PENZBENI_EGESZSEG_BIZT_JARULEK: f64 = 0.03;
const TERMESZETBENI_EGESZSEG_BIZT_JARULEK: f64 = 0.04;
const SZJA: f64 = 0.15;
const MUNKAERO_PIACI_JARULEK: f64 = 0.015;
const ONE: f64 = 1.00;

fn main() -> Result<(), slint::PlatformError> {
    let ui: AppWindow = AppWindow::new()?;
    let ui_handle: Weak<AppWindow> = ui.as_weak();
    ui.on_divide_income(move |string: SharedString| {
        let ui: AppWindow = ui_handle.unwrap();
        let brutto_num: f64 = string.trim().parse().unwrap();
        let sum_of_jarulek: f64 = NYUGDIJ_BIZT_JARULEK
            + PENZBENI_EGESZSEG_BIZT_JARULEK
            + TERMESZETBENI_EGESZSEG_BIZT_JARULEK
            + SZJA
            + MUNKAERO_PIACI_JARULEK;
        let netto_num: f64 = brutto_num * ( ONE - sum_of_jarulek);

        let nyugdij_bizt: f64 = brutto_num * NYUGDIJ_BIZT_JARULEK;
        let penzbeni_egeszseg_bizt: f64 = brutto_num * PENZBENI_EGESZSEG_BIZT_JARULEK;
        let term_egeszseg_bizt: f64 = brutto_num * TERMESZETBENI_EGESZSEG_BIZT_JARULEK;
        let szja: f64 = brutto_num * SZJA;
        let munkaero_piaci: f64 = brutto_num * MUNKAERO_PIACI_JARULEK;
        let result: String = format!("Járulékok: \nNyugdíj-biztosítási járulék: {:.2} Ft\nPénzbeni Egészségbiztosítási járulék: {:.2} Ft\nTermészetbeni Egészségbiztosítási járulék: {:.2} Ft\nSZJA (személyi jövedelemadó): {:.2} Ft\nMunkaerő-piaci járulék: {:.2} Ft\n\nNettó bér: {:.2} Ft", nyugdij_bizt, penzbeni_egeszseg_bizt, term_egeszseg_bizt, szja, munkaero_piaci, netto_num);
        ui.set_results(result.into());
    });
    ui.run()
}

