use std::env;

pub fn verify_account_template(first_name: &String, token: &String) -> String {
    let app_base_url = env::var("APP_BASE_URL").expect("APP_BASE_URL is not set in .env file");
    let verify_account_url = format!("{app_base_url}/api/user/verify-account?token={token}");

    format!(
        r#"
        <html>
            <body>
                <p>Hi, {first_name}</p>
                <p>Welcome to money transfer,</p>
                <p>We are delighted to have you.</p>
                <p>Please verify your email by clicking on the link below:</p>
                <a href="{verify_account_url}">Click here to verify your account</a>
            </body>
        </html>
        "#
    )
}
