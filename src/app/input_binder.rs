use std::sync::mpsc::Sender;

use inputbot::KeybdKey::*;

use super::app_event::AppEvent;

pub(crate) fn add_bindings(tx: Sender<AppEvent>) {
    //let tx1= tx.clone();
    //HKey.bind(move || {
    //    if LAltKey.is_pressed() {
    //        tx1.send(AppEvent::Left).unwrap();
    //    }
    //});

    //let tx2 = tx.clone();
    //LKey.bind(move || {
    //    if LAltKey.is_pressed() {
    //        tx2.send(AppEvent::Right).unwrap();
    //    }
    //});

    let tx3 = tx.clone();
    AKey.bind(move || {
        if LAltKey.is_pressed() {
            tx3.send(AppEvent::ListAll).unwrap();
        }
    });

    let tx4 = tx.clone();
    UKey.bind(move || {
        if LAltKey.is_pressed() {
            tx4.send(AppEvent::UpdateLayout).unwrap();
        }
    });
}
