use std::{thread, time};

use interprog::TaskManager;
fn main() {
    let mut manager = TaskManager::new();
    manager.add_task("Logging in", None);
    manager.start();
    thread::sleep(time::Duration::from_millis(1000));
    manager.finish();
    let classes = vec![
        "Bible 8 - Mr. Delke",
        "Band II - Mr. Ryan",
        "PLTW Space & Electricity - Mr. Fairweather",
    ];
    for class in &classes {
        manager.add_task(&format!("Scraping {class}"), Some(4));
    }
    for class in &classes {
        manager.start_task(&format!("Scraping {class}"));
        thread::sleep(time::Duration::from_millis(1000));
        // manager.add_task(&format!("Scraping {class}"), None);
        for _ in 0..4 {
            manager.increment(1, false);
            thread::sleep(time::Duration::from_millis(300));
        }
        manager.finish()
    }
}
