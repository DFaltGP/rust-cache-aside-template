use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    error::{ErrorInternalServerError, ErrorUnauthorized},
    web,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{Ready, ready},
    marker::PhantomData,
    rc::Rc,
};

pub struct AuthMiddleware<S> {
    service: Rc<S>,
}

pub struct AuthMiddlewareFactory<S> {
    pub _service: PhantomData<S>,
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddlewareFactory<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();

        Box::pin(async move {
            let pool = req
                .app_data::<web::Data<sqlx::PgPool>>()
                .ok_or_else(|| ErrorInternalServerError("DB Pool não encontrado"))?;

            let auth_header = req.headers().get("Authorization");
            let token = match auth_header {
                Some(header_value) => {
                    let parts = header_value
                        .to_str()
                        .unwrap_or("")
                        .split(' ')
                        .collect::<Vec<&str>>();
                    if parts.len() == 2 && parts[0] == "Bearer" {
                        parts[1]
                    } else {
                        return Err(ErrorUnauthorized("Formato de token inválido"));
                    }
                }
                None => return Err(ErrorUnauthorized("Token faltando")),
            };

            let claims = match crate::utils::decode_jwt(token) {
                Ok(c) => c,
                Err(_) => return Err(ErrorUnauthorized("Token inválido ou expirado")),
            };

            let schema_name = format!("tenant_{}", claims.tid.simple());
            let isolation_query = format!("SET search_path TO \"{}\"", schema_name);

            if let Err(e) = sqlx::query(&isolation_query).execute(pool.get_ref()).await {
                log::error!("Falha ao configurar search_path: {:?}", e);
                return Err(ErrorInternalServerError(
                    "Erro ao configurar isolamento dos dados",
                ));
            }

            srv.call(req).await
        })
    }
}
