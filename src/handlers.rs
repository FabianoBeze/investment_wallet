use axum::{
    extract::{State, Path, FromRequestParts},
    response::{IntoResponse, Redirect, Html},
    Form,
    http::{StatusCode, request::Parts},
};
use std::future::Future;
use axum_extra::extract::cookie::{CookieJar, Cookie};
use askama::Template;
use sqlx::MySqlPool;
use crate::models::{Investment, Claims, User, PortfolioHistoryEntry};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use bcrypt::verify;
use std::time::{SystemTime, UNIX_EPOCH};

fn jwt_secret() -> String {
    std::env::var("JWT_SECRET")
        .expect("JWT_SECRET não configurado")
}

pub struct AuthUser {
    pub id: i32,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = Redirect;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let headers = parts.headers.clone();

        async move {
            let jar = CookieJar::from_headers(&headers);

            let token = jar
                .get("jwt_token")
                .map(|c| c.value().to_string())
                .ok_or_else(|| Redirect::to("/login"))?;

            let secret = jwt_secret();

            let token_data = decode::<Claims>(
                &token,
                &DecodingKey::from_secret(secret.as_bytes()),
                &Validation::default(),
            )
            .map_err(|_| Redirect::to("/login"))?;

            Ok(AuthUser {
                id: token_data.claims.sub,
            })
        }
    }
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    error: Option<String>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    investments: Vec<Investment>,
    total_value: f64,
    history: Vec<PortfolioHistoryEntry>,
}

#[derive(serde::Deserialize)]
pub struct CreateInvestment {
    pub asset_name: String,
    pub amount: f64,
    pub current_price: f64,
    pub token_address: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}
#[derive(serde::Deserialize)]
pub struct RegisterPayload {
    pub username: String,
    pub password: String,
}
#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate {
    error: Option<String>,
}
pub async fn login_page_handler() -> impl IntoResponse {
    Html(LoginTemplate { error: None }.render().unwrap())
}
pub async fn register_page_handler() -> impl IntoResponse {
    Html(
        RegisterTemplate {
            error: None,
        }
        .render()
        .unwrap(),
    )
}

pub async fn login_post_handler(
    State(pool): State<MySqlPool>,
    jar: CookieJar,
    Form(payload): Form<LoginPayload>,
) -> impl IntoResponse {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(&pool)
    .await
    .unwrap();

    if let Some(user) = user {
        if verify(&payload.password, &user.password_hash).unwrap() {
            let exp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize
                + 86400;

            let claims = Claims {
                sub: user.id,
                exp,
            };

            let secret = jwt_secret();

            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(secret.as_bytes()),
            )
            .unwrap();

            let cookie = Cookie::build(("jwt_token", token))
                .http_only(true)
                .path("/")
                .build();

            return (jar.add(cookie), Redirect::to("/")).into_response();
        }
    }

    Html(
        LoginTemplate {
            error: Some("Credenciais inválidas".to_string()),
        }
        .render()
        .unwrap(),
    )
    .into_response()
}

pub async fn register_post_handler(
    State(pool): State<MySqlPool>,
    Form(payload): Form<RegisterPayload>,
) -> impl IntoResponse {
    

    let existing_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(&pool)
    .await
    .unwrap();

    if existing_user.is_some() {
        return Html(
            RegisterTemplate {
                error: Some("Usuário já existe".to_string()),
            }
            .render()
            .unwrap(),
        )
        .into_response();
    }

    let password_hash = bcrypt::hash(
        &payload.password,
        bcrypt::DEFAULT_COST,
    )
    .unwrap();
    

    sqlx::query(
        "INSERT INTO users (username, password_hash)
         VALUES (?, ?)"
    )
    .bind(&payload.username)
    .bind(password_hash)
    .execute(&pool)
    .await
    .unwrap();
    println!("Usuário cadastrado com sucesso!");

    Redirect::to("/login").into_response()
}

pub async fn home_handler(
    auth: AuthUser,
    State(pool): State<MySqlPool>,
) -> impl IntoResponse {
    let mut investments = sqlx::query_as::<_, Investment>(
        "SELECT * FROM investments WHERE user_id = ?"
    )
    .bind(auth.id)
    .fetch_all(&pool)
    .await
    .unwrap_or_else(|e| {
        eprintln!("Erro ao buscar investimentos: {}", e);
        Vec::new()
    });
    println!("Buscados {} investimentos para o usuário {}", investments.len(), auth.id);

    // Atualiza preços dos tokens Solana antes de exibir
    println!("Iniciando atualização de preços para {} investimentos do usuário {}", investments.len(), auth.id);
    for inv in &mut investments {
        println!("Processando investimento id={}, token_address={:?}", inv.id, inv.token_address);
        if let Some(address) = &inv.token_address {
            if address.is_empty() {
                continue; // Ignora tokens sem endereço preenchido
            }
            println!("Buscando preço para o endereço: {}", address);
            match crate::services::dex_screener::get_token_price(address).await {
                Ok(price) => {
                    println!("Preço obtido com sucesso: ${}", price);
                    inv.current_price = price;
                    // Atualiza também no banco de dados
                    let result = sqlx::query(
                        "UPDATE investments SET current_price = ? WHERE id = ?"
                    )
                    .bind(price)
                    .bind(inv.id)
                    .execute(&pool)
                    .await;
                    if let Err(e) = result {
                        eprintln!("Erro ao atualizar preço no banco: {:?}", e);
                    } else {
                        println!("Preço atualizado no banco para o investimento {}", inv.id);
                    }
                }
                Err(e) => {
                    eprintln!("Erro ao buscar preço para {}: {:?}", address, e);
                }
            }
        }
    }

    let total_value: f64 = investments
        .iter()
        .map(|inv| inv.amount * inv.current_price)
        .sum();
        
    // Salva o valor atual da carteira no histórico
    let _ = sqlx::query(
        "INSERT INTO portfolio_history (user_id, total_value) VALUES (?, ?)"
    )
    .bind(auth.id)
    .bind(total_value)
    .execute(&pool)
    .await;
    println!("Histórico salvo para o usuário {}: valor total ${}", auth.id, total_value);
    
    // Busca o histórico da carteira
    let history = sqlx::query_as!(PortfolioHistoryEntry,
        "SELECT id, user_id, recorded_at, total_value FROM portfolio_history WHERE user_id = ? ORDER BY recorded_at ASC",
        auth.id
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_else(|e| {
        eprintln!("Erro ao buscar histórico: {}", e);
        Vec::new()
    });
    println!("Total de registros no histórico: {}", history.len());

    match (IndexTemplate {
        investments,
        total_value,
        history,
    })
    .render()
    {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Erro ao renderizar template: {}", e);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Erro ao renderizar template",
            )
                .into_response()
        }
    }
}
pub async fn create_investment_handler(
    auth: AuthUser,
    State(pool): State<MySqlPool>,
    Form(payload): Form<CreateInvestment>,
) -> impl IntoResponse {
    let result = sqlx::query("INSERT INTO investments (user_id, asset_name, amount, current_price, token_address) VALUES (?, ?, ?, ?, ?)")
        .bind(auth.id)
        .bind(&payload.asset_name)
        .bind(payload.amount)
        .bind(payload.current_price)
        .bind(payload.token_address)
        .execute(&pool)
        .await;

    if let Err(e) = result {
        eprintln!("Erro ao inserir investimento: {:?}", e);
    }

    Redirect::to("/")
}

pub async fn delete_investment_handler(
    auth: AuthUser,
    State(pool): State<MySqlPool>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let _ = sqlx::query("DELETE FROM investments WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(auth.id)
        .execute(&pool)
        .await;

    Redirect::to("/")
}

pub async fn logout_handler(
    jar: CookieJar,
) -> impl IntoResponse {

    let cookie = Cookie::build(("jwt_token", ""))
        .path("/")
        .build();

    (jar.remove(cookie), Redirect::to("/login"))
}