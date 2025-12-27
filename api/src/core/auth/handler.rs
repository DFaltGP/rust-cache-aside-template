use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::auth::models::LoginRequest,
    utils::{create_jwt, hash_password, verify_password},
};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub nome_empresa: String,
    pub cnpj: String,
    pub nome_usuario: String,
    pub email: String,
    pub senha: String,
}

#[derive(sqlx::FromRow)]
pub struct UserLoginRecord {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub password_hash: String,
    pub role: String,
}

pub async fn signup(pool: web::Data<PgPool>, req: web::Json<RegisterRequest>) -> impl Responder {
    let password_hash = match hash_password(&req.senha) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().body("Erro ao processar senha"),
    };

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().body("Erro no banco de dados"),
    };

    let schema_name = format!("tenant_{}", Uuid::new_v4().simple());

    let tenant_insert = sqlx::query!(
        r#"
      INSERT INTO public.tenants (name, cnpj, schema_name)
      VALUES ($1, $2, $3)
      RETURNING id"#,
        req.nome_empresa,
        req.cnpj,
        schema_name
    )
    .fetch_one(&mut *tx)
    .await;

    let tenant_id = match tenant_insert {
        Ok(record) => record.id,
        Err(e) => {
            log::error!("Erro ao criar tenant: {:?}", e);
            return HttpResponse::BadRequest().body("Erro ao criar empresa (CNPJ duplicado?)");
        }
    };

    let user_insert = sqlx::query!(
        r#"
            INSERT INTO public.users (tenant_id, name, email, password_hash, role)
            VALUES ($1, $2, $3, $4, 'admin_dono')
            "#,
        tenant_id,
        req.nome_usuario,
        req.email,
        password_hash
    )
    .execute(&mut *tx)
    .await;

    if let Err(e) = user_insert {
        log::error!("Erro ao criar usuário: {:?}", e);
        return HttpResponse::BadRequest().body("Email já cadastrado?");
    }

    let create_schema_query = format!("CREATE SCHEMA \"{}\"", schema_name);
    if let Err(e) = sqlx::query(&create_schema_query).execute(&mut *tx).await {
        log::error!("Falha fatal ao criar schema: {:?}", e);
        return HttpResponse::InternalServerError().body("Falha na infraestrutura do Tenant");
    }

    if let Err(_) = tx.commit().await {
        return HttpResponse::InternalServerError().body("Falha ao confirmar transação");
    }

    HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Empresa criada com sucesso!",
        "tenant_id": tenant_id
    }))
}

pub async fn login(pool: web::Data<PgPool>, req: web::Json<LoginRequest>) -> impl Responder {
    let user_record = match sqlx::query_as!(
        UserLoginRecord,
        r#"SELECT id, tenant_id, password_hash, role FROM public.users WHERE email = $1"#,
        req.email
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(record)) => record,
        Ok(None) | Err(_) => {
            return HttpResponse::Unauthorized().body("Credenciais Inválidas");
        }
    };

    if !verify_password(&req.senha, &user_record.password_hash) {
        return HttpResponse::Unauthorized().body("Credenciais Inválidas");
    }

    let token = match create_jwt(user_record.id, user_record.tenant_id, user_record.role) {
        Ok(t) => t,
        Err(e) => {
            log::error!("Erro ao gerar JWT: {:?}", e);
            return HttpResponse::InternalServerError().body("Falha na autenticação");
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "token": token,
        "expires_in_days": 7
    }))
}
