use super::config::EnvConfig;

pub fn verify_account_template(first_name: &String, token: &String, env: &EnvConfig) -> String {
    let verify_account_url = format!(
        "{}/api/user/verify-account?token={}",
        env.app_base_url, token
    );

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
