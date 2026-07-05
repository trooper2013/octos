use uuid::Uuid;

/// Evaluates intent schemas, prints console layout frames, and queries approval tokens.
pub async fn render_dynamic_widget(intent: &str, payload: &str, interactive: bool) -> Option<String> {
    if intent == "approve_payment" {
        println!("\n+--------------------------------------------------------+");
        println!("|           OCTOS DYNAMIC WIDGET: PAYMENT APPROVAL       |");
        println!("+--------------------------------------------------------+");
        println!("| Intent:  {:<45} |", intent);
        println!("| Details: {:<45} |", payload);
        println!("+--------------------------------------------------------+");
        println!("| [!] ACTION REQUIRED: Confirm payment audit of $5000    |");

        let approved = if interactive {
            let join_handle = tokio::task::spawn_blocking(|| {
                use std::io::{self, Write};
                print!("| [?] Enter 'yes'/'y' to approve, or any key to decline: ");
                let _ = io::stdout().flush();
                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_ok() {
                    let trimmed = input.trim().to_lowercase();
                    trimmed == "yes" || trimmed == "y"
                } else {
                    false
                }
            });
            join_handle.await.unwrap_or(false)
        } else {
            println!("| Simulating biometric / human pin confirmation...       |");
            true
        };

        if approved {
            let verification_token = format!(
                "TOKEN-VERIFY-{}",
                Uuid::new_v4().to_string().split('-').next().unwrap().to_uppercase()
            );
            println!("| Status:  APPROVED                                      |");
            println!("| Generated Token: {:<37} |", verification_token);
            println!("+--------------------------------------------------------+\n");
            Some(verification_token)
        } else {
            println!("| Status:  DECLINED                                      |");
            println!("+--------------------------------------------------------+\n");
            None
        }
    } else if intent == "select_photo" {
        println!("\n+--------------------------------------------------------+");
        println!("|           OCTOS DYNAMIC WIDGET: PHOTO SELECTION       |");
        println!("+--------------------------------------------------------+");
        println!("| Intent:  {:<45} |", intent);
        println!("| Details: {:<45} |", payload);
        println!("+--------------------------------------------------------+");
        println!("| Simulating user photo selection...                     |");
        let token = "PHOTO-CONFIRM-9988".to_string();
        println!("| Selected Image Token: {:<32} |", token);
        println!("+--------------------------------------------------------+\n");
        Some(token)
    } else {
        None
    }
}
