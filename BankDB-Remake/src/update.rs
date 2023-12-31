use crossterm::event::Event as CrosstermEvent;
use std::sync::{Arc, Mutex};
use tui_input::backend::crossterm::EventHandler;
use sqlx::{Row, Pool, Postgres, FromRow};
use rust_decimal::Decimal;
use bcrypt::verify;
use anyhow::Result;

use crate::model::{App, InputMode, TimeoutType, Client, AccountType, Transaction};
use crate::event::Event;

pub async fn update(app: &mut Arc<Mutex<App>>, pool: &Pool<Postgres>, event: Event) -> Result<()> {
    match event {
        Event::Quit => {
            app.lock().unwrap().should_quit = true;
            Ok(())
        },
        Event::TimeoutStep(timeout_type) => {
            app.lock().unwrap().update_timeout_counter(timeout_type);
            Ok(())
        },
        Event::TryLogin => {
            if app.lock().unwrap().failed_logins == 3 {
                return Ok(());
            }

            let username: String = app.lock().unwrap().input.0.value().to_string();
            let password: String = app.lock().unwrap().input.1.value().to_string();

            if let Some(res) = sqlx::query("SELECT * FROM clients WHERE username = $1")
                .bind(&username)
                .fetch_optional(pool)
                .await? {
                    let password_hash: String = res.try_get("password")?;

                    if verify(&password, &password_hash).unwrap_or_else(|error| panic!("{}", error)) {
                        //todo!("[ LOGIN SUCCESSFUL ] Name: {res_name}, Password: {password}");
                        app.lock().unwrap().active_user = {
                            let balance: Decimal = res.try_get("balance")?;
                            let account_type: String = res.try_get("account_type")?;
                            let last_transaction: Option<String> = res.try_get("last_transaction")?;
                            Some(Client {
                                account_number: res.try_get("account_number")?,
                                username,
                                name: res.try_get("name")?,
                                ci: res.try_get("ci")?,
                                balance,
                                account_type: {
                                    if account_type == "current" {
                                        AccountType::Current
                                    } else {
                                        AccountType::Debit
                                    }
                                },
                                last_transaction: {
                                    if last_transaction.is_some() {
                                        Some(Transaction::from_row(&sqlx::query(
                                            "SELECT * FROM transactions WHERE username = $1")
                                                .bind(&last_transaction.unwrap())
                                                .fetch_one(pool)
                                                .await?)?
                                            )
                                    } else {
                                        None
                                    }
                                },    
                                suspended: res.try_get("suspended")?,
                            })
                        };
                        todo!("login successful, but not yet implemented");
                        return Ok(());
                    }
                }
            let mut app_lock = app.lock().unwrap();
            app_lock.failed_logins += 1;
            
            if app_lock.failed_logins == 3 {
                app_lock.add_timeout(30, 1000, TimeoutType::Login);
            }
            Ok(())
        },
        Event::SwitchInput => {
            let mut app_lock = app.lock().unwrap();
            if let InputMode::Editing(field) = app_lock.input_mode {
                if field == 0 { app_lock.input_mode = InputMode::Editing(1) }
                else { app_lock.input_mode = InputMode::Editing(0) }
            }
            Ok(())
        }
        Event::Key(key_event) => {
            let mut app_lock = app.lock().unwrap();
            if let InputMode::Editing(field) = app_lock.input_mode {
                if field == 0 { app_lock.input.0.handle_event(&CrosstermEvent::Key(key_event)); }
                else { app_lock.input.1.handle_event(&CrosstermEvent::Key(key_event)); }
            }
            Ok(())
        },
        _ => { Ok(()) }
    }
}