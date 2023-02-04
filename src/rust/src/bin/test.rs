use std::{thread, time};

use interprog::TaskManager;
fn main() {
    let mut manager = TaskManager::new();
    manager.add_task("name".into(), None);
    manager.start();
    thread::sleep(time::Duration::from_millis(4000));
    manager.finish();
}
