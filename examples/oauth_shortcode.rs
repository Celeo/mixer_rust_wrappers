use mixer_wrappers::oauth::{check_shortcode, get_shortcode, ShortcodeStatus};
use std::{thread, time::Duration};

fn main() {
    let resp = get_shortcode("CLIENT_ID_HERE", "CLIENT_SECRET_HERE", &[]).unwrap();
    println!(
        "Code is {}; go to https://mixer.com/go to enter\n\n",
        resp.code
    );
    loop {
        let status = check_shortcode(&resp.handle);
        println!("Status: {:?}", status);
        let code: String = match status {
            ShortcodeStatus::UserGrantedAccess(ref c) => c.to_owned(),
            ShortcodeStatus::UserDeniedAccess => break,
            ShortcodeStatus::HandleInvalid => break,
            _ => {
                thread::sleep(Duration::from_secs(3));
                continue;
            }
        };
        println!("Code: {}", code);
        break;
    }
}
