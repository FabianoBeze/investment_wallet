use serde_json::Value;

/// Busca o preço atual de um token via API do DexScreener.
pub async fn get_token_price(
    token_address: &str,
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {

    let url = format!(
        "https://api.dexscreener.com/latest/dex/tokens/{}",
        token_address
    );

    let response = reqwest::get(&url).await?;
    let json: Value = response.json().await?;

    let price = json["pairs"]
        .get(0)
        .and_then(|pair| pair["priceUsd"].as_str())
        .unwrap_or("0")
        .parse::<f64>()?;

    Ok(price)
}