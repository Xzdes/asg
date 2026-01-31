//! Модуль `concurrency`
//!
//! Поддержка многопоточности в ASG:
//! - Concurrency (запуск нового потока)
//! - Демонстрация безопасного использования std::thread
//!
//! Гарантирует Send/Sync и отсутствие глобального состояния.

use std::thread;

use crate::{ASGError, ASGResult};

/// Запустить новый поток исполнения.
///
/// # Пример:
/// ```
/// use asg_lang::concurrency::spawn_thread;
/// spawn_thread("Hello from thread!".to_string());
/// ```
pub fn spawn_thread(message: String) -> ASGResult<()> {
    let handle = thread::spawn(move || {
        println!("Thread: {}", message);
    });

    handle
        .join()
        .map_err(|_| ASGError::Concurrency("Thread panicked!".into()))?;

    Ok(())
}

/// Проверить, можно ли безопасно запустить Concurrency.
///
/// На данном этапе всегда возвращает Ok(true).
pub fn check_concurrency_safety() -> ASGResult<bool> {
    Ok(true)
}
