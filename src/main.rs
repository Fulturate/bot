mod config;
mod handlers;
mod loader;
mod util;

use crate::loader::run;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    println!("Bot starting...");

    // let converter_ru = CurrencyConverter::new(OutputLanguage::Russian).unwrap();
    //
    // let converter_en = CurrencyConverter::new(OutputLanguage::English).unwrap();
    //
    // let text = "Convert 1 dollar and 10 pounds";
    // let results_en = converter_en.process_text(text).await;
    // if let Ok(result) = results_en {
    //     println!("---\n{:?}\n---", result);
    // }
    //
    // let text_ru = "Конвертируй 2 гривны и 5 евро";
    // let results_ru = converter_ru.process_text(text_ru).await;
    // if let Ok(result) = results_ru {
    //     println!("---\n{:?}\n---", result);
    // }

    match run().await {
        Ok(_) => println!("Bot stopped"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
